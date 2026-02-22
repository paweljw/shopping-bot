# Build stage
FROM rust:1.89-alpine AS builder

# Install build dependencies including OpenSSL and SQLite
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    sqlite-dev \
    sqlite-static

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies (only ca-certificates for HTTPS)
RUN apk add --no-cache ca-certificates

# Create data directory
RUN mkdir -p /data

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/shopping-bot /app/shopping-bot

# Set the data directory as a volume
VOLUME ["/data"]

EXPOSE 8080

CMD ["/app/shopping-bot"]