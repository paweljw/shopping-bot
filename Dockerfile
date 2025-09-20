# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    sqlite

# Create a non-root user
RUN addgroup -g 1000 bot && \
    adduser -D -u 1000 -G bot bot

# Create data directory
RUN mkdir -p /data && chown bot:bot /data

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/shopping-bot /app/shopping-bot

# Switch to non-root user
USER bot

# Set the data directory as a volume
VOLUME ["/data"]

# Run the bot
CMD ["/app/shopping-bot"]