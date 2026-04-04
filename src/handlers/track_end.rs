use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
};

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::{colour::Color, prelude::ChannelId, prelude::GuildId},
    prelude::Mutex,
};
use songbird::{Call, Event, EventContext, EventHandler as VoiceEventHandler, Songbird};
use tokio::task::AbortHandle;
use tracing::{debug, error, info};

use crate::utils::type_map::cancel_disconnect_timer;

pub struct TrackEndNotifier {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
    pub call: Arc<Mutex<Call>>,
    pub guild_id: GuildId,
    pub manager: Arc<Songbird>,
    pub disconnect_timers: Arc<StdMutex<HashMap<GuildId, AbortHandle>>>,
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
            cancel_disconnect_timer(&self.disconnect_timers, self.guild_id);
            return None;
        }

        debug!("Queue ended in channel {}", self.channel_id);

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

        // Cancel any existing timer before starting a new one
        cancel_disconnect_timer(&self.disconnect_timers, self.guild_id);

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
        let timers_clone = self.disconnect_timers.clone();

        let join_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(timeout_minutes * 60)).await;

            // Safety net: double-check queue is still empty
            let is_still_empty = {
                let handler = call_clone.lock().await;
                handler.queue().current_queue().is_empty()
            };

            if !is_still_empty {
                debug!(
                    "Auto-disconnect cancelled - queue is no longer empty in guild {}",
                    guild_id
                );
                timers_clone.lock().unwrap().remove(&guild_id);
                return;
            }

            info!(
                "Auto-disconnect timer expired, leaving voice channel in guild {}",
                guild_id
            );

            timers_clone.lock().unwrap().remove(&guild_id);

            if let Err(err) = manager_clone.remove(guild_id).await {
                error!("Failed to auto-disconnect from guild {}: {}", guild_id, err);
            } else {
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
        });

        self.disconnect_timers
            .lock()
            .unwrap()
            .insert(self.guild_id, join_handle.abort_handle());

        None
    }
}
