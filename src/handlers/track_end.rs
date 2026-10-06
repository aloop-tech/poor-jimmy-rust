use std::sync::Arc;

use serenity::{async_trait, builder::CreateMessage, model::prelude::GuildId, prelude::Mutex};
use songbird::{Call, Event, EventContext, EventHandler as VoiceEventHandler};
use tracing::debug;

use crate::utils::{
    response::{announce, ok_embed},
    voice::VoiceHandles,
};

pub struct TrackEndNotifier {
    pub voice: VoiceHandles,
    pub call: Arc<Mutex<Call>>,
    pub guild_id: GuildId,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::Track(_) = ctx else {
            return None;
        };

        let is_queue_empty = {
            let handler = self.call.lock().await;
            handler.queue().current_queue().is_empty()
        };

        if !is_queue_empty {
            self.voice.guilds.cancel_idle_timer(self.guild_id);
            return None;
        }

        debug!("Queue ended in guild {}", self.guild_id);

        announce(
            &self.voice.http,
            &self.voice.guilds,
            self.guild_id,
            CreateMessage::new().embed(ok_embed("Queue has **ended!**")),
        )
        .await;

        self.voice.start_disconnect_timer(self.guild_id);

        None
    }
}
