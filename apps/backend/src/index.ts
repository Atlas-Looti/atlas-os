import { Hono } from "hono";
import { corsMiddleware } from "./middleware/cors.ts";
import { errorHandler } from "./middleware/error.ts";
import { clerkAuth } from "./middleware/auth.ts";
import { apiKeyAuth } from "./middleware/apikey.ts";
import { health } from "./routes/health.ts";
import { keys } from "./routes/keys.ts";
import { rpc } from "./routes/atlas-os/rpc.ts";
import { runMigrations } from "./lib/migrate.ts";

// Run migrations before accepting traffic
await runMigrations();

const app = new Hono();

// Global middleware
app.use("*", corsMiddleware);
app.use("*", errorHandler);

// ── Public ────────────────────────────────────────────
app.route("/health", health);

// ── Dashboard management (Clerk JWT) ─────────────────
app.use("/keys/*", clerkAuth);
app.route("/keys", keys);

// ── Atlas OS — CLI / SDK (Atlas API key) ─────────────
const atlasOs = new Hono();
atlasOs.use("/rpc/*", apiKeyAuth);
atlasOs.route("/rpc", rpc);

app.route("/atlas-os", atlasOs);

// 404 fallback
app.notFound((ctx) => ctx.json({ error: "Not found" }, 404));

const port = parseInt(process.env["PORT"] ?? "3001", 10);
console.log(`[atlas-backend] Starting on http://localhost:${port}`);

export default {
    port,
    fetch: app.fetch,
};
