use std::sync::Arc;

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::{colour::Color, prelude::ChannelId},
};
use songbird::{Event, EventContext, EventHandler};
use tracing::{error, info};

use crate::components::music_buttons::create_music_buttons;

pub struct TrackPlayHandler {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
    pub title: String,
    pub thumbnail: String,
}

#[async_trait]
impl EventHandler for TrackPlayHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        // Continue only if this is a Track event
        let EventContext::Track(_) = ctx else {
            return None;
        };

        info!("Now playing: '{}' in channel {}", self.title, self.channel_id);

        let embed = CreateEmbed::new()
            .description(format!("**Now playing:** {}", self.title.clone()))
            .image(self.thumbnail.clone())
            .color(Color::DARK_GREEN);

        let message = CreateMessage::new()
            .embed(embed)
            .components(create_music_buttons());

        if let Err(err) = self.channel_id.send_message(&self.http, message).await {
            error!("Failed to send now playing message to channel {}: {}", self.channel_id, err);
        }

        None
    }
}
