use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex},
};

use reqwest::Client as HttpClient;
use serenity::model::prelude::GuildId;
use tokio::task::AbortHandle;

/// Pending auto-disconnect timers, keyed by guild
pub type DisconnectTimers = Arc<StdMutex<HashMap<GuildId, AbortHandle>>>;

/// State shared by every command and event handler
pub struct Data {
    pub http_client: HttpClient,
    pub disconnect_timers: DisconnectTimers,
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
