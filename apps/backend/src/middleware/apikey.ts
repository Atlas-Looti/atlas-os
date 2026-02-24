import { db } from "../lib/db.ts";
import { hashKey } from "../lib/keygen.ts";
import type { Context, Next } from "hono";

/**
 * Atlas API Key middleware.
 * Verifies the `atl_xxx` key from Authorization header or X-API-Key header.
 * Used for machine-to-machine auth (CLI → RPC proxy).
 * Does NOT use Clerk — checks against the api_keys table via key_hash.
 */
export async function apiKeyAuth(ctx: Context, next: Next) {
    const header =
        ctx.req.header("X-API-Key") ??
        ctx.req.header("Authorization")?.replace(/^Bearer\s+/i, "");

    if (!header?.startsWith("atl_")) {
        return ctx.json({ error: "Missing or invalid API key" }, 401);
    }

    const keyHash = hashKey(header);

    const { rows } = await db.query<{ id: string; user_id: string }>(
        "SELECT id, user_id FROM api_keys WHERE key_hash = $1",
        [keyHash]
    );

    if (rows.length === 0) {
        return ctx.json({ error: "Invalid API key" }, 401);
    }

    const row = rows[0]!;
    ctx.set("userId", row.user_id);
    ctx.set("apiKeyId", row.id);
    await next();
}
