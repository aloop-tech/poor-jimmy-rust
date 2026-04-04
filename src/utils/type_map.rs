use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex},
};

use reqwest::Client as HttpClient;
use serenity::{
    all::Context,
    model::prelude::GuildId,
    prelude::TypeMapKey,
};
use tokio::task::AbortHandle;

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

pub struct DisconnectTimerKey;

impl TypeMapKey for DisconnectTimerKey {
    type Value = Arc<StdMutex<HashMap<GuildId, AbortHandle>>>;
}

pub async fn get_http_client(ctx: &Context) -> HttpClient {
    let data = ctx.data.read().await;
    data.get::<HttpKey>()
        .cloned()
        .expect("Guaranteed to exist in the typemap.")
}

pub async fn get_disconnect_timers(
    ctx: &Context,
) -> Arc<StdMutex<HashMap<GuildId, AbortHandle>>> {
    let data = ctx.data.read().await;
    data.get::<DisconnectTimerKey>()
        .cloned()
        .expect("Guaranteed to exist in the typemap.")
}

pub fn cancel_disconnect_timer(
    timers: &StdMutex<HashMap<GuildId, AbortHandle>>,
    guild_id: GuildId,
) {
    if let Some(handle) = timers.lock().unwrap().remove(&guild_id) {
        handle.abort();
    }
}
