# Use the official Rust image as a builder
FROM rust:1.84-slim-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/atlas-os

# Copy the entire workspace
COPY . .

# Build the backend in release mode
# Note: We build from the root because it's a workspace
RUN cargo build --release -p atlas-backend

# Use a slim runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /usr/src/atlas-os/target/release/atlas-server /app/atlas-server

# Create a default config directory (Atlas OS expects this)
RUN mkdir -p /root/.atlas-os

# Expose the default backend port
EXPOSE 3001

# Set environment variables
ENV PORT=3001
ENV RUST_LOG=info

# Run the backend
CMD ["./atlas-server"]
