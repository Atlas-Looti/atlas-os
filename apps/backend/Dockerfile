# ── Stage 1: Build compiled binary ────────────────────────────────────────────
FROM oven/bun:1 AS builder
WORKDIR /app

COPY package.json bun.lock* ./
RUN bun install --frozen-lockfile

COPY tsconfig.json ./
COPY src ./src

RUN bun build --compile --bytecode --minify --sourcemap \
    --target=bun-linux-x64 --no-compile-autoload-dotenv \
    ./src/index.ts --outfile=server

# ── Stage 2: Minimal runtime ─────────────────────────────────────────────────
FROM debian:12-slim AS runner

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1001 atlas && useradd --uid 1001 --gid atlas --create-home atlas

COPY --from=builder --chown=atlas:atlas /app/server /app/server

USER atlas
WORKDIR /app
EXPOSE 3001

CMD ["./server"]
