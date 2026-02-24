# ── Stage 1: Builder ──────────────────────────────────────────────────────────
FROM oven/bun:1-alpine AS builder
WORKDIR /app

COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

COPY tsconfig.json ./
COPY src ./src

RUN bun run build

# ── Stage 2: Runner ───────────────────────────────────────────────────────────
FROM oven/bun:1-alpine AS runner
WORKDIR /app

COPY --from=builder /app/server ./server

EXPOSE 3001

CMD ["./server"]
