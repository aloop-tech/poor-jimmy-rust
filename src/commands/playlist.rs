use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::CreateEmbed,
    client::Context,
    model::colour::Color,
};
use songbird::input::YoutubeDl;
use tracing::error;
use url::Url;

use crate::utils::{
    response::respond_to_followup,
    track_utils::{enqueue_track, enqueue_track_list},
    type_map::get_http_client,
};

use rustypipe::{client::RustyPipe, model::VideoCodec};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let rp = RustyPipe::new();

    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer play-url command: {}", err);
        return;
    }

    let mut response_embed = CreateEmbed::default();

    let command_value = command.data.options.first();

    let resolved_value = match command_value {
        Some(data) => &data.value,
        _ => {
            response_embed = response_embed
                .description("Please provide a URL to play!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;

            return;
        }
    };

    let url = match resolved_value {
        CommandDataOptionValue::String(value) => value.clone(),
        _ => {
            response_embed = response_embed
                .description("Please provide a valid URL!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;

            return;
        }
    };

    let http_client = get_http_client(&ctx).await;

    let id_playlist = get_id_playlist(url.clone());

    let mut urls: Vec<String> = Vec::new();

    if !is_valid_youtube_url(url.clone()) || id_playlist.is_err() {
        response_embed = response_embed
            .description("Please provide a valid **/watch** Youtube URL")
            .color(Color::DARK_RED);

        respond_to_followup(command, &ctx.http, response_embed, false).await;

        return;
    }

    let mut playlist = rp
        .query()
        .playlist(id_playlist.unwrap())
        .await
        .expect("Unable to access playlist");

    playlist.videos.extend_limit(rp.query(), 50).await.unwrap();

    playlist
        .videos
        .items
        .iter()
        .for_each(|v| urls.push(create_link_youtbe(v.id.to_string())));

    for url in urls.iter() {
        let source = YoutubeDl::new(http_client.clone(), url.clone());
        enqueue_track_list(&ctx, command, source.into()).await;
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("playlist")
        .description("Play the audio from a Youtube Playlist URL")
        .add_option(
            serenity::builder::CreateCommandOption::new(
                CommandOptionType::String,
                "url",
                "A Youtube Playlist URL",
            )
            .required(true),
        )
}

fn is_valid_youtube_url(url: String) -> bool {
    (url.contains("youtube.com") && (url.contains("/watch"))) || url.contains("youtu.be")
}

fn get_id_playlist(url: String) -> Result<String, String> {
    let url = Url::parse(url.as_str()).expect("Erro ao parsear link");

    let playlist_id = url
        .query_pairs()
        .find(|(k, _)| k == "list")
        .map(|(_, v)| v.into_owned()); // transforma Cow<str> em String

    if let Some(id) = playlist_id {
        return Result::Ok(id);
    }

    Result::Err("Não foi possivel resolver link".to_string())
}

fn create_link_youtbe(id: String) -> String {
    return format!("https://www.youtube.com/watch?v={}", id);
}
