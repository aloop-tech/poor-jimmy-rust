use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex},
};

use reqwest::Client as HttpClient;
use serenity::model::prelude::GuildId;
use tokio::task::AbortHandle;
use tracing::warn;

/// Pending auto-disconnect timers, keyed by guild
pub type DisconnectTimers = Arc<StdMutex<HashMap<GuildId, AbortHandle>>>;

/// State shared by every command and event handler
pub struct Data {
    pub http_client: HttpClient,
    pub disconnect_timers: DisconnectTimers,
    /// Minutes to wait in an empty queue before leaving the voice channel
    pub auto_disconnect_minutes: u64,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub fn cancel_disconnect_timer(
    timers: &StdMutex<HashMap<GuildId, AbortHandle>>,
    guild_id: GuildId,
) {
    if let Some(handle) = timers.lock().unwrap().remove(&guild_id) {
        handle.abort();
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
}
