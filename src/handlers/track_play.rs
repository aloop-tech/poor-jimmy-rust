use std::sync::Arc;

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::{colour::Color, prelude::GuildId},
};
use songbird::{Event, EventContext, EventHandler};
use tracing::info;

use crate::{
    components::music_buttons::create_music_buttons, data::GuildStates, utils::response::announce,
};

pub struct TrackPlayHandler {
    pub http: Arc<Http>,
    pub guilds: GuildStates,
    pub guild_id: GuildId,
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

        info!("Now playing: '{}' in guild {}", self.title, self.guild_id);

        let embed = CreateEmbed::new()
            .description(format!("**Now playing:** {}", self.title.clone()))
            .image(self.thumbnail.clone())
            .color(Color::DARK_GREEN);

        let message = CreateMessage::new()
            .embed(embed)
            .components(create_music_buttons());

        announce(&self.http, &self.guilds, self.guild_id, message).await;

        None
    }
}
