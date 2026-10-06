use crate::{
    data::{Context, Error},
    utils::response::{ok_embed, reply},
};

/// Display directions on how to use Poor Jimmy's commands
#[poise::command(slash_command, guild_only)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    reply(ctx, ok_embed(get_help_text())).await
}

/// Get the help description text
pub fn get_help_text() -> String {
    String::from(
        "## 🎶 Poor Jimmy - Discord Music Bot 🎶

**Getting Started**
First, join a voice channel, then use `/join` to bring Poor Jimmy into your channel.

**Playing Music**
• `/play-title <title>` - Search and play a song by title
  Example: `/play-title never gonna give you up`

• `/play-url <url>` - Play a specific YouTube video or share link
  Example: `/play-url https://youtube.com/watch?v=...`

• `/search <query>` - Search YouTube and select from results
  Example: `/search lofi hip hop`

**Playback Controls**
• `/pause` - Pause the current song
• `/resume` - Resume playback
• `/skip` - Skip to the next song in queue
• `/loop` - Toggle looping for the current song
• `/now-playing` - Show current song with progress bar

**Queue Management**
• `/list` - View all songs in the queue
• `/clear` - Stop playback and clear the entire queue

**Other Commands**
• `/join` - Summon Poor Jimmy to your voice channel
• `/leave` - Remove Poor Jimmy from the voice channel
• `/ping` - Check if the bot is responsive
• `/damnit-jimmy` - Update Jimmy's dependencies (bot owner only)
• `/help` - Display this help message

**Tips**
- Use the interactive buttons that appear with songs for quick controls
- Queue up multiple songs - they'll play automatically
- Poor Jimmy must be in a voice channel to play music",
    )
}
