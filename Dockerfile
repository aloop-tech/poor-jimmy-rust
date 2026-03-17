### Builder Image ###
FROM rust:1.88-alpine AS builder

# Install build dependencies
RUN apk add --update --no-cache \
    alpine-sdk \
    pkgconfig \
    cmake \
    musl-dev \
    openssl \
    libressl-dev

WORKDIR /poor-jimmy

# Copy manifests first to cache dependency builds
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy the actual source code
COPY src ./src

# Touch source files to ensure cargo detects changes and rebuilds
RUN find src -type f -exec touch {} +

# Build the actual bot (dependencies are cached, only app code rebuilds)
RUN cargo build --release

### Final Image ###
# This final image is what is ultimately shipped. It just has the bot's binary
# and all the dependencies it needs. We leave behind all the build tools.
FROM alpine:latest

# Install only runtime dependencies
RUN apk add --update --no-cache \
    ffmpeg \
    python3 \
    py3-pip \
    libgcc \
    ca-certificates

# Install yt-dlp via pip for easier updates (gets latest version from PyPI)
RUN pip install --break-system-packages yt-dlp

# Set the working directory for where the binary will live
WORKDIR /bot

# Copy the release binary to our final image
COPY --from=builder /poor-jimmy/target/release/poor-jimmy ./

# Command to start the bot once the container starts
CMD [ "/bot/poor-jimmy" ]
