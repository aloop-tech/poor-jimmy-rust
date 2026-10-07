use std::sync::Arc;

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::{colour::Color, prelude::GuildId},
};
use songbird::{Event, EventContext, EventHandler};
use tracing::{debug, error, info, warn};

use crate::{components::music_buttons::create_music_buttons, data::GuildStates};

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

        let Some(channel_id) = self.guilds.text_channel(self.guild_id) else {
            warn!(
                "No text channel known for guild {}, dropping now playing message",
                self.guild_id
            );
            return None;
        };

        let message = CreateMessage::new()
            .embed(embed)
            .components(create_music_buttons());

        let message = match channel_id.send_message(&self.http, message).await {
            Ok(message) => message,
            Err(err) => {
                error!(
                    "Failed to send now playing message to channel {}: {}",
                    channel_id, err
                );
                return None;
            }
        };

        // Replace the previous player (possibly in another channel), so the
        // only one is the new one at the bottom of the chat
        let previous = self.guilds.with(self.guild_id, |state| {
            state.player_message.replace((channel_id, message.id))
        });

        if let Some((old_channel, old_message)) = previous {
            // Most likely someone deleted it already
            if let Err(err) = old_channel.delete_message(&self.http, old_message).await {
                debug!("Couldn't delete old player message: {}", err);
            }
        }

        None
    }
}
