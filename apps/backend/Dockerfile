# Build Stage
FROM rust:1.80 as builder

WORKDIR /usr/src/atlas-os

# Copy workspace Cargo.toml
COPY Cargo.toml Cargo.lock ./

# Copy all crates (monorepo structure)
COPY crates crates
COPY apps apps

# Build the backend application
RUN cargo build -p backend --release

# Production Stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/atlas-os/target/release/backend /usr/local/bin/backend

CMD ["backend"]
