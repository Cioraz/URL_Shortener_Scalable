# ---- Builder Stage ----
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev build-base

# Set working directory
WORKDIR /app

# Add Cargo manifest first to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy source and env
COPY src ./src
COPY .env .env

# Add musl target and build statically
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

# ---- Runtime Stage ----
FROM alpine:latest

# Install minimal runtime dependencies (CA certs for HTTPS)
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN adduser -D appuser

WORKDIR /app

# Copy statically linked binary and env file
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/url_shortener .
COPY --from=builder /app/.env .env

# Use non-root user
USER appuser

# Expose the app port
EXPOSE 8000

# Run the binary
CMD ["./url_shortener"]
