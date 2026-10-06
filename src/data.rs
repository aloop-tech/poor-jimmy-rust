use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex},
};

use reqwest::Client as HttpClient;
use serenity::model::prelude::{ChannelId, GuildId, MessageId};
use tokio::task::AbortHandle;
use tracing::warn;

/// State shared by every command and event handler
pub struct Data {
    pub http_client: HttpClient,
    pub guilds: GuildStates,
    /// Minutes to wait in an empty queue or empty voice channel before leaving
    pub auto_disconnect_minutes: u64,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// What the bot remembers about each guild
#[derive(Default)]
pub struct GuildState {
    /// Where automatic messages (Now playing, Queue ended, …) go: the channel of
    /// the most recent command or button press
    pub text_channel: Option<ChannelId>,
    /// Pending auto-disconnect timer
    pub disconnect_timer: Option<AbortHandle>,
    /// Everyone else left the voice channel and the bot is waiting for them
    pub waiting_for_listeners: bool,
    /// Playback was paused because the channel emptied; resume when someone returns
    pub paused_for_empty_channel: bool,
    /// The "Now playing" message edited for each new track in this voice session
    pub player_message: Option<(ChannelId, MessageId)>,
}

/// Per-guild state, safe to share between tasks
#[derive(Clone, Default)]
pub struct GuildStates(Arc<StdMutex<HashMap<GuildId, GuildState>>>);

impl GuildStates {
    /// Run `f` with the guild's state, creating it if needed. Keep `f` short:
    /// it holds a lock shared by every guild.
    pub fn with<R>(&self, guild_id: GuildId, f: impl FnOnce(&mut GuildState) -> R) -> R {
        let mut guilds = self.0.lock().unwrap();
        f(guilds.entry(guild_id).or_default())
    }

    pub fn text_channel(&self, guild_id: GuildId) -> Option<ChannelId> {
        self.with(guild_id, |state| state.text_channel)
    }

    pub fn set_text_channel(&self, guild_id: GuildId, channel_id: ChannelId) {
        self.with(guild_id, |state| state.text_channel = Some(channel_id));
    }

    /// Start tracking a new disconnect timer, aborting any previous one
    pub fn replace_disconnect_timer(&self, guild_id: GuildId, timer: AbortHandle) {
        if let Some(old) = self.with(guild_id, |state| state.disconnect_timer.replace(timer)) {
            old.abort();
        }
    }

    /// Abort the disconnect timer unconditionally
    pub fn cancel_disconnect_timer(&self, guild_id: GuildId) {
        if let Some(timer) = self.with(guild_id, |state| state.disconnect_timer.take()) {
            timer.abort();
        }
    }

    /// Abort the disconnect timer because something is playing again, unless the
    /// timer is counting down an empty voice channel: music playing to nobody
    /// shouldn't keep the bot around.
    pub fn cancel_idle_timer(&self, guild_id: GuildId) {
        let timer = self.with(guild_id, |state| {
            if state.waiting_for_listeners {
                None
            } else {
                state.disconnect_timer.take()
            }
        });

        if let Some(timer) = timer {
            timer.abort();
        }
    }

    /// Forget everything about the current voice session, e.g. when (re)joining
    pub fn reset_voice_session(&self, guild_id: GuildId) {
        self.cancel_disconnect_timer(guild_id);
        self.with(guild_id, |state| {
            state.waiting_for_listeners = false;
            state.paused_for_empty_channel = false;
            state.player_message = None;
        });
    }
}

const DEFAULT_AUTO_DISCONNECT_MINUTES: u64 = 5;

/// Parse the AUTO_DISCONNECT_MINUTES setting, falling back to the default when
/// it's unset or not a whole number of minutes.
pub fn parse_auto_disconnect_minutes(value: Option<&str>) -> u64 {
    let Some(value) = value else {
        return DEFAULT_AUTO_DISCONNECT_MINUTES;
    };

    match value.trim().parse::<u64>() {
        Ok(minutes) => minutes,
        Err(_) => {
            warn!(
                "Invalid AUTO_DISCONNECT_MINUTES '{}', using default of {} minutes",
                value, DEFAULT_AUTO_DISCONNECT_MINUTES
            );
            DEFAULT_AUTO_DISCONNECT_MINUTES
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_disconnect_minutes_unset_uses_default() {
        assert_eq!(parse_auto_disconnect_minutes(None), 5);
    }

    #[test]
    fn test_auto_disconnect_minutes_valid() {
        assert_eq!(parse_auto_disconnect_minutes(Some("10")), 10);
        assert_eq!(parse_auto_disconnect_minutes(Some(" 1 ")), 1);
        assert_eq!(parse_auto_disconnect_minutes(Some("0")), 0);
    }

    #[test]
    fn test_auto_disconnect_minutes_invalid_uses_default() {
        assert_eq!(parse_auto_disconnect_minutes(Some("")), 5);
        assert_eq!(parse_auto_disconnect_minutes(Some("ten")), 5);
        assert_eq!(parse_auto_disconnect_minutes(Some("-3")), 5);
        assert_eq!(parse_auto_disconnect_minutes(Some("2.5")), 5);
    }

    #[tokio::test]
    async fn test_idle_timer_not_cancelled_while_waiting_for_listeners() {
        let guilds = GuildStates::default();
        let guild_id = GuildId::new(1);

        let task = tokio::spawn(std::future::pending::<()>());
        guilds.replace_disconnect_timer(guild_id, task.abort_handle());
        guilds.with(guild_id, |state| state.waiting_for_listeners = true);

        guilds.cancel_idle_timer(guild_id);
        assert!(guilds.with(guild_id, |state| state.disconnect_timer.is_some()));

        guilds.with(guild_id, |state| state.waiting_for_listeners = false);
        guilds.cancel_idle_timer(guild_id);
        assert!(guilds.with(guild_id, |state| state.disconnect_timer.is_none()));
        assert!(task.await.unwrap_err().is_cancelled());
    }

    #[tokio::test]
    async fn test_reset_voice_session_clears_flags_and_timer() {
        let guilds = GuildStates::default();
        let guild_id = GuildId::new(1);
        let channel_id = ChannelId::new(2);

        let task = tokio::spawn(std::future::pending::<()>());
        guilds.set_text_channel(guild_id, channel_id);
        guilds.replace_disconnect_timer(guild_id, task.abort_handle());
        guilds.with(guild_id, |state| {
            state.waiting_for_listeners = true;
            state.paused_for_empty_channel = true;
            state.player_message = Some((channel_id, MessageId::new(3)));
        });

        guilds.reset_voice_session(guild_id);

        guilds.with(guild_id, |state| {
            assert!(state.disconnect_timer.is_none());
            assert!(!state.waiting_for_listeners);
            assert!(!state.paused_for_empty_channel);
            assert!(state.player_message.is_none());
            // The announce channel is about the text side, so it survives
            assert_eq!(state.text_channel, Some(channel_id));
        });
        assert!(task.await.unwrap_err().is_cancelled());
    }
}
