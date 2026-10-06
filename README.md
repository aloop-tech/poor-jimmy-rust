# Poor Jimmy 🎶

Poor Jimmy is a feature-rich Discord music bot written in Rust. This project is a re-write of the [existing Poor Jimmy written with TypeScript](https://github.com/andrewmloop/poor-jimmy). The bot utilizes modern Rust libraries including Serenity and Poise for Discord API interactions, Songbird for high-quality audio playback, and Tokio for asynchronous runtime.

## Dependencies

**Core Libraries**
- [Rust 2024 Edition](https://www.rust-lang.org/learn)
- [Serenity v0.12.5](https://docs.rs/serenity/latest/serenity/) - Discord API wrapper
- [Poise v0.6](https://docs.rs/poise/latest/poise/) - Slash command framework
- [Songbird v0.5.0](https://docs.rs/songbird/latest/songbird/) - Audio playback (from the [beerpsi-forks `davey` branch](https://github.com/beerpsi-forks/songbird/tree/davey) until DAVE voice encryption lands in a release)
- [Tokio v1.50](https://tokio.rs/) - Async runtime

**Additional Dependencies**
- `reqwest` - HTTP client for audio streams (rustls only, no OpenSSL)
- `serde` & `serde_json` - Serialization
- `tracing` & `tracing-subscriber` - Logging
- `symphonia` - Audio codecs, including the mkv demuxer YouTube's webm/opus audio needs (unused directly, but must stay a dependency)
- `fastrand` - Queue shuffling

**External Tools**
- [`yt-dlp`](https://github.com/yt-dlp/yt-dlp) - YouTube audio extraction, installed with its `default` extra for the YouTube challenge solver
- [`deno`](https://deno.com/) - JavaScript runtime yt-dlp uses to solve YouTube's playback challenges

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) - Rust toolchain (edition 2024)
- [Docker](https://www.docker.com/get-started) - For containerization (optional)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - YouTube audio extraction tool
- [deno](https://deno.com/) - JavaScript runtime used by yt-dlp for YouTube
- Discord Bot Token - Create a bot at [Discord Developer Portal](https://discord.com/developers/applications)

### Configuration

1. **Clone the repository:**
   ```bash
   git clone https://github.com/andrewmloop/poor-jimmy-rust.git
   cd poor-jimmy-rust
   ```

2. **Create a `.env` file** in the project root:
   ```bash
   DISCORD_TOKEN=your_discord_bot_token_here
   ```

3. **Ensure yt-dlp and deno are installed** (the Docker image already includes both):
   ```bash
   # macOS
   brew install yt-dlp deno

   # Linux
   pip install "yt-dlp[default]"
   curl -fsSL https://deno.land/install.sh | sh

   # Or download from https://github.com/yt-dlp/yt-dlp and https://deno.com
   ```

### Running Locally (Native)

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Run the bot:**
   ```bash
   cargo run --release
   ```

   Or run the binary directly:
   ```bash
   ./target/release/poor-jimmy
   ```

### Running with Docker

1. **Build the Docker image:**
   ```bash
   docker build -t poor-jimmy .
   ```

2. **Run the container:**
   ```bash
   docker run --env-file .env poor-jimmy
   ```

## Usage

Join a voice channel and use `/play <song title or link>`. Poor Jimmy joins your channel automatically. Links can be single videos or playlists (up to 50 songs). Use `/help` in Discord for every command, including the queue tools (`/list`, `/remove`, `/move`, `/shuffle`) and playback controls.

Poor Jimmy posts its automatic messages ("Now playing", "Queue has ended", …) in the channel where someone last used one of its commands or buttons. It pauses when everyone leaves the voice channel, resumes if someone returns, and leaves after `AUTO_DISCONNECT_MINUTES` of an empty queue or empty channel.

Commands only work in servers, not DMs.

## Development

Before pushing, run the same checks CI runs on every pull request and push to `main` (`.github/workflows/check.yml`):

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Pushes to `main` build and publish the Docker image (`.github/workflows/deploy.yml`). It also rebuilds every Monday so the image picks up the latest yt-dlp.

## Deployment

Poor Jimmy can be deployed to various platforms using Docker. Here are some common deployment scenarios:

### Docker Hub Deployment

1. **Build and tag the image:**
   ```bash
   docker build -t poor-jimmy .
   docker tag poor-jimmy <username>/<repository>:<version>
   ```

2. **Push to Docker Hub:**
   ```bash
   docker push <username>/<repository>:<version>
   ```

3. **Pull and run on target machine:**
   ```bash
   docker pull <username>/<repository>:<version>
   docker run --env-file .env <username>/<repository>:<version>
   ```

### Raspberry Pi Deployment

Poor Jimmy runs efficiently on Raspberry Pi devices:

1. **Build for ARM architecture (if building from x86):**
   ```bash
   docker buildx build --platform linux/arm64 -t poor-jimmy .
   ```

2. **Transfer and run on Raspberry Pi:**
   ```bash
   # On Raspberry Pi with Docker installed
   docker pull <username>/<repository>:<version>
   docker run -d --restart unless-stopped --env-file .env <username>/<repository>
   ```

### Cloud Platform Deployment (Heroku, AWS, etc.)

1. **Build for x86_64:**
   ```bash
   docker build --platform linux/amd64 -t poor-jimmy .
   ```

2. **Deploy to platform** (example for Heroku):
   ```bash
   docker tag poor-jimmy registry.heroku.com/<app-name>/worker
   docker push registry.heroku.com/<app-name>/worker
   heroku container:release worker --app <app-name>
   ```

### Environment Variables

Required environment variables:
- `DISCORD_TOKEN` - Your Discord bot token

Optional environment variables:
- `RUST_LOG` - Set logging level (e.g., `info`, `debug`, `warn`)
- `AUTO_DISCONNECT_MINUTES` - Minutes to wait before leaving the voice channel once the queue has ended or everyone has left (e.g. `10`, defaults to 5 minutes). Read once at startup.

## Bot Permissions

Poor Jimmy doesn't need any privileged gateway intents. When inviting it to your Discord server, ensure it has the following permissions:

- **Voice Permissions:**
  - Connect
  - Speak
  - Use Voice Activity

- **Text Permissions:**
  - Send Messages
  - Embed Links
  - Read Message History
  - Use Slash Commands


## Troubleshooting

**Bot doesn't join voice channel:**
- Ensure you're in a voice channel when using `/play` or `/join`
- Check that the bot has "Connect" and "Speak" permissions

**Music doesn't play:**
- Verify `yt-dlp` and `deno` are installed and accessible
- Check that the link is valid and from a site yt-dlp supports
- YouTube changes often: the bot owner can run `/damnit-jimmy` to update yt-dlp, and the Docker image rebuilds weekly with the latest version
- Ensure the bot has proper voice permissions

**Commands not showing up:**
- Commands are server-only, so they won't appear in DMs with the bot
- Discord may take up to an hour to register slash commands globally
- Try kicking and re-inviting the bot
- Check that the bot has "Use Slash Commands" permission

## License

This project is a learning exercise. Please feel free to copy or fork as you please.
