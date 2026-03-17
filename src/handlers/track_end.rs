use std::{env, sync::Arc, time::Duration};

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::{colour::Color, prelude::ChannelId, prelude::GuildId},
    prelude::Mutex,
};
use songbird::{Call, Event, EventContext, EventHandler as VoiceEventHandler, Songbird};
use tracing::{debug, error, info};

pub struct TrackEndNotifier {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
    pub call: Arc<Mutex<Call>>,
    pub guild_id: GuildId,
    pub manager: Arc<Songbird>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        // Continue only if this is a Track event
        let EventContext::Track(_) = ctx else {
            return None;
        };

        let is_queue_empty = {
            let handler = self.call.lock().await;
            handler.queue().current_queue().is_empty()
        }; // lock released before any I/O

        if is_queue_empty {
            debug!("Queue ended in channel {}", self.channel_id);
            // No songs left in the queue, notify the channel
            let embed = CreateEmbed::new()
                .description("Queue has **ended!**")
                .color(Color::DARK_GREEN);

            let message = CreateMessage::new().embed(embed);

            if let Err(err) = self.channel_id.send_message(&self.http, message).await {
                error!(
                    "Failed to send queue end notification to channel {}: {}",
                    self.channel_id, err
                );
            }

            // Start auto-disconnect timer
            let timeout_minutes = env::var("AUTO_DISCONNECT_MINUTES")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(5);

            info!(
                "Starting auto-disconnect timer for {} minutes in guild {}",
                timeout_minutes, self.guild_id
            );

            let call_clone = self.call.clone();
            let guild_id = self.guild_id;
            let manager_clone = self.manager.clone();
            let channel_id = self.channel_id;
            let http_clone = self.http.clone();

            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(timeout_minutes * 60)).await;

                // Check if queue is still empty
                let handler = call_clone.lock().await;
                let queue = handler.queue().current_queue();
                drop(handler); // Release lock before potentially leaving

                if queue.is_empty() {
                    info!(
                        "Auto-disconnect timer expired, leaving voice channel in guild {}",
                        guild_id
                    );

                    if let Err(err) = manager_clone.remove(guild_id).await {
                        error!("Failed to auto-disconnect from guild {}: {}", guild_id, err);
                    } else {
                        // Send disconnect notification
                        let embed = CreateEmbed::new()
                            .description(format!(
                                "Left voice channel after {} minutes of inactivity!",
                                timeout_minutes
                            ))
                            .color(Color::DARK_GREEN);

                        let message = CreateMessage::new().embed(embed);

                        if let Err(err) = channel_id.send_message(&http_clone, message).await {
                            error!(
                                "Failed to send auto-disconnect notification to channel {}: {}",
                                channel_id, err
                            );
                        }
                    }
                } else {
                    debug!(
                        "Auto-disconnect cancelled - queue is no longer empty in guild {}",
                        guild_id
                    );
                }
            });
        }

        None
    }
}
