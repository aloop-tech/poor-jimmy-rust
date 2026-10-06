use std::sync::Arc;

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage, EditMessage},
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

        // Edit this session's player message if it's in the channel people are
        // using now; otherwise (first track, channel changed) post a new one
        let player_message = self
            .guilds
            .with(self.guild_id, |state| state.player_message)
            .filter(|(player_channel, _)| *player_channel == channel_id);

        if let Some((_, message_id)) = player_message {
            let edit = EditMessage::new()
                .embed(embed.clone())
                .components(create_music_buttons());

            match channel_id.edit_message(&self.http, message_id, edit).await {
                Ok(_) => return None,
                // Most likely someone deleted it
                Err(err) => debug!("Couldn't edit player message, posting a new one: {}", err),
            }
        }

        let message = CreateMessage::new()
            .embed(embed)
            .components(create_music_buttons());

        match channel_id.send_message(&self.http, message).await {
            Ok(message) => self.guilds.with(self.guild_id, |state| {
                state.player_message = Some((channel_id, message.id));
            }),
            Err(err) => error!(
                "Failed to send now playing message to channel {}: {}",
                channel_id, err
            ),
        }

        None
    }
}
