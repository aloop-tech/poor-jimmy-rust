use serenity::{
    all::{Color, CommandInteraction, CreateEmbed},
    client::Context,
};
use tracing::error;

use crate::utils::{response::respond_to_followup, track_utils::TrackMetadata};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer list command: {}", err);
        return;
    }

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    if let Some(call) = manager.get(guild_id) {
        let current_queue = {
            let handler = call.lock().await;
            handler.queue().current_queue()
        }; // Release lock on handler

        if current_queue.is_empty() {
            let embed = CreateEmbed::new()
                .description("The queue is **empty!**")
                .color(Color::DARK_GREEN);
            respond_to_followup(command, &ctx.http, embed, false).await;

            return;
        }

        // Transform the Vec of TrackHandles into a Vec of titles
        let queue_titles: Vec<String> = current_queue
            .iter()
            .map(|track| track.data::<TrackMetadata>().title.clone())
            .collect();

        // Build the response description string.
        let response_description = format_queue_description(queue_titles);

        let embed = CreateEmbed::new()
            .description(response_description)
            .color(Color::DARK_GREEN);
        respond_to_followup(command, &ctx.http, embed, false).await;
    } else {
        let embed = CreateEmbed::new()
            .description(
                "Error listing queue! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);
        respond_to_followup(command, &ctx.http, embed, false).await;
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("list").description("Display the current queue of songs")
}

/// Discord rejects embeds whose description is longer than this
const MAX_DESCRIPTION_LEN: usize = 4096;

fn format_queue_description(list_of_titles: Vec<String>) -> String {
    let mut description = String::new();

    // Room to keep free for the "...and N more" line if the list gets cut off
    let overflow_reserve = format!("...and {} more", list_of_titles.len()).len();

    for (index, title) in list_of_titles.iter().enumerate() {
        let line = format!("**{}:** {}\n", index + 1, title);
        let is_last = index + 1 == list_of_titles.len();
        let limit = if is_last {
            MAX_DESCRIPTION_LEN
        } else {
            MAX_DESCRIPTION_LEN - overflow_reserve
        };

        if description.len() + line.len() > limit {
            description.push_str(&format!("...and {} more", list_of_titles.len() - index));
            break;
        }

        description.push_str(&line);
    }

    description
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_queue_description_empty() {
        let titles = vec![];
        let result = format_queue_description(titles);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_queue_description_single() {
        let titles = vec!["Song One".to_string()];
        let result = format_queue_description(titles);
        assert_eq!(result, "**1:** Song One\n");
    }

    #[test]
    fn test_format_queue_description_multiple() {
        let titles = vec![
            "First Song".to_string(),
            "Second Song".to_string(),
            "Third Song".to_string(),
        ];
        let result = format_queue_description(titles);
        assert_eq!(
            result,
            "**1:** First Song\n**2:** Second Song\n**3:** Third Song\n"
        );
    }

    #[test]
    fn test_format_queue_description_with_special_characters() {
        let titles = vec![
            "Song with emoji 🎵".to_string(),
            "Song with **markdown**".to_string(),
        ];
        let result = format_queue_description(titles);
        assert!(result.contains("🎵"));
        assert!(result.contains("**markdown**"));
    }

    #[test]
    fn test_format_queue_description_truncates_long_queue() {
        let titles: Vec<String> = (0..200).map(|i| format!("{:0>80}", i)).collect();
        let result = format_queue_description(titles);

        assert!(result.len() <= MAX_DESCRIPTION_LEN);
        assert!(result.starts_with("**1:** "));
        assert!(result.ends_with(" more"));

        // Every listed track plus the "more" count should add up to the whole queue
        let listed = result.lines().filter(|line| line.starts_with("**")).count();
        let more: usize = result
            .lines()
            .last()
            .unwrap()
            .trim_start_matches("...and ")
            .trim_end_matches(" more")
            .parse()
            .unwrap();
        assert_eq!(listed + more, 200);
    }
}
