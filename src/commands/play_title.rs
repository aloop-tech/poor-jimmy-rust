use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::CreateEmbed,
    client::Context,
    model::colour::Color,
};
use songbird::input::YoutubeDl;
use tracing::error;

use crate::utils::{
    response::respond_to_followup, track_utils::enqueue_track, type_map::get_http_client,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer play-title command: {}", err);
        return;
    }

    let mut response_embed = CreateEmbed::default();

    let command_value = command.data.options.first();

    let resolved_value = match command_value {
        Some(data) => &data.value,
        _ => {
            response_embed = response_embed
                .description("Please provide a title to search!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;

            return;
        }
    };

    let title = match resolved_value {
        CommandDataOptionValue::String(value) => value.clone(),
        _ => {
            response_embed = response_embed
                .description("Please provide a valid title!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;

            return;
        }
    };

    let http_client = get_http_client(ctx).await;

    // Get the audio source for the URL
    let source = YoutubeDl::new_search(http_client, title);

    enqueue_track(ctx, command, source.into()).await;
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("play-title")
        .description("Play the audio from a Youtube video searching by title")
        .add_option(
            serenity::builder::CreateCommandOption::new(
                CommandOptionType::String,
                "title",
                "A Youtube video title",
            )
            .required(true),
        )
}
