# BotRust 🎶

BoRust is a feature-rich Discord music bot written in Rust. The bot utilizes modern Rust libraries including Serenity for Discord API interactions, Songbird for high-quality audio playback, and Tokio for asynchronous runtime.

## Dependencies

**Core Libraries**

- [Rust 2024 Edition](https://www.rust-lang.org/learn)
- [Serenity v0.12.4](https://docs.rs/serenity/latest/serenity/) - Discord API wrapper
- [Songbird v0.5.0](https://docs.rs/songbird/latest/songbird/) - Audio playback
- [Tokio v1.47](https://tokio.rs/) - Async runtime

**Additional Dependencies**

- `reqwest` - HTTP client for YouTube-DL
- `serde` & `serde_json` - Serialization
- `tracing` & `tracing-subscriber` - Logging
- `symphonia` - Audio codec support

**External Tool**

- [`yt-dlp`](https://github.com/yt-dlp/yt-dlp) - YouTube audio extraction

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) - Rust toolchain (edition 2024)
- [Docker](https://www.docker.com/get-started) - For containerization (optional)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - YouTube audio extraction tool
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

3. **Ensure yt-dlp is installed:**

   ```bash
   # macOS
   brew install yt-dlp

   # Linux
   pip install yt-dlp

   # Or download from https://github.com/yt-dlp/yt-dlp
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
- `AUTO_DISCONNECT_MINUTES` - Set auto disconnect wait time (e.g. `10`, defaults to 5 minutes)

## Bot Permissions

When inviting Poor Jimmy to your Discord server, ensure it has the following permissions:

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

- Ensure you're in a voice channel when using `/join`
- Check that the bot has "Connect" and "Speak" permissions

**Music doesn't play:**

- Verify `yt-dlp` is installed and accessible
- Check that the YouTube URL is valid
- Ensure the bot has proper voice permissions

**Commands not showing up:**

- Discord may take up to an hour to register slash commands globally
- Try kicking and re-inviting the bot
- Check that the bot has "Use Slash Commands" permission

## License

This project is a learning exercise. Please feel free to copy or fork as you please.
