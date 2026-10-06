use std::{sync::Arc, time::Duration};

use serenity::{
    all::{Cache, ChannelId, Context as SerenityContext, CreateMessage, GuildId, UserId},
    http::Http,
    prelude::Mutex,
};
use songbird::{Call, Event, Songbird, TrackEvent, error::JoinError};
use tracing::{debug, error, info};

use crate::{
    data::{Data, GuildStates},
    handlers::track_end::TrackEndNotifier,
    utils::response::{announce, ok_embed},
};

/// The voice channel a user is currently in, from the cache
pub fn user_voice_channel(cache: &Cache, guild_id: GuildId, user_id: UserId) -> Option<ChannelId> {
    cache
        .guild(guild_id)?
        .voice_states
        .get(&user_id)?
        .channel_id
}

/// Whether anyone else is in the bot's voice channel. Other bots are skipped
/// when the cache has their member info. `None` when the cache doesn't (yet)
/// show the bot in a voice channel.
pub fn has_listeners(cache: &Cache, guild_id: GuildId) -> Option<bool> {
    let bot_id = cache.current_user().id;
    let guild = cache.guild(guild_id)?;
    let bot_channel = guild.voice_states.get(&bot_id)?.channel_id?;

    Some(guild.voice_states.values().any(|voice_state| {
        let is_bot = voice_state
            .member
            .as_ref()
            .is_some_and(|member| member.user.bot);

        voice_state.channel_id == Some(bot_channel) && voice_state.user_id != bot_id && !is_bot
    }))
}

/// Join (or move to) a voice channel and set up the per-session handlers
pub async fn connect(
    ctx: &SerenityContext,
    data: &Data,
    manager: Arc<Songbird>,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<Arc<Mutex<Call>>, Box<JoinError>> {
    info!(
        "Attempting to join voice channel {} in guild {}",
        channel_id, guild_id
    );

    let call = manager.join(guild_id, channel_id).await.map_err(Box::new)?;

    data.guilds.reset_voice_session(guild_id);

    {
        let mut handler = call.lock().await;

        handler.remove_all_global_events();

        handler.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                voice: VoiceHandles::new(ctx, manager.clone(), data),
                call: call.clone(),
                guild_id,
            },
        );
    } // lock released before any HTTP requests

    info!(
        "Successfully joined voice channel {} in guild {}",
        channel_id, guild_id
    );

    Ok(call)
}

/// What background voice tasks need to post messages and leave a guild
#[derive(Clone)]
pub struct VoiceHandles {
    pub http: Arc<Http>,
    pub cache: Arc<Cache>,
    pub manager: Arc<Songbird>,
    pub guilds: GuildStates,
    pub timeout_minutes: u64,
}

impl VoiceHandles {
    pub fn new(ctx: &SerenityContext, manager: Arc<Songbird>, data: &Data) -> Self {
        Self {
            http: ctx.http.clone(),
            cache: ctx.cache.clone(),
            manager,
            guilds: data.guilds.clone(),
            timeout_minutes: data.auto_disconnect_minutes,
        }
    }

    /// Leave the voice channel after the configured timeout, unless the bot is
    /// no longer idle (something is queued and someone is listening) by then.
    /// Replaces any timer already running for the guild.
    pub fn start_disconnect_timer(&self, guild_id: GuildId) {
        info!(
            "Starting auto-disconnect timer for {} minutes in guild {}",
            self.timeout_minutes, guild_id
        );

        let voice = self.clone();

        let task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(voice.timeout_minutes * 60)).await;

            voice
                .guilds
                .with(guild_id, |state| state.disconnect_timer = None);

            let Some(call) = voice.manager.get(guild_id) else {
                return;
            };

            // Safety net: double-check the bot is still idle
            let queue_is_empty = call.lock().await.queue().is_empty();
            let listeners = has_listeners(&voice.cache, guild_id).unwrap_or(false);

            if !queue_is_empty && listeners {
                debug!(
                    "Auto-disconnect cancelled - no longer idle in guild {}",
                    guild_id
                );
                return;
            }

            info!(
                "Auto-disconnect timer expired, leaving voice channel in guild {}",
                guild_id
            );

            if let Err(err) = voice.manager.remove(guild_id).await {
                error!("Failed to auto-disconnect from guild {}: {}", guild_id, err);
                return;
            }

            let embed = ok_embed(format!(
                "Left voice channel after {} minutes of inactivity!",
                voice.timeout_minutes
            ));
            announce(
                &voice.http,
                &voice.guilds,
                guild_id,
                CreateMessage::new().embed(embed),
            )
            .await;
        });

        self.guilds
            .replace_disconnect_timer(guild_id, task.abort_handle());
    }
}
