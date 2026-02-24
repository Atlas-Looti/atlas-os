import { Hono } from "hono";
import { db } from "../lib/db.ts";
import { redis } from "../lib/redis.ts";
import { generateApiKey, keyPrefix, hashKey } from "../lib/keygen.ts";

const keys = new Hono<{ Variables: { userId: string } }>();

const CACHE_TTL = 60;
const cacheKey = (userId: string) => `atlas:keys:${userId}`;

/**
 * GET /api/keys
 * List API keys for the authenticated user. Redis-cached 60s.
 */
keys.get("/", async (ctx) => {
    const userId = ctx.get("userId");

    const cached = await redis.get(cacheKey(userId));
    if (cached) {
        return ctx.json({ keys: JSON.parse(cached) as unknown[], cached: true });
    }

    const result = await db.query<{
        id: string;
        user_id: string;
        name: string;
        prefix: string;
        created_at: string;
    }>(
        "SELECT id, user_id, name, prefix, created_at FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
        [userId]
    );

    await redis.setex(cacheKey(userId), CACHE_TTL, JSON.stringify(result.rows));
    return ctx.json({ keys: result.rows, cached: false });
});

/**
 * POST /api/keys
 * Create a new API key for the authenticated user.
 * Body: { name: string }
 */
keys.post("/", async (ctx) => {
    const userId = ctx.get("userId");
    const body = await ctx.req.json<{ name?: string }>();
    const { name } = body;

    if (!name?.trim()) {
        return ctx.json({ error: "name is required" }, 400);
    }

    const rawKey = generateApiKey();
    const prefix = keyPrefix(rawKey);
    const keyHash = hashKey(rawKey);

    const result = await db.query<{
        id: string;
        user_id: string;
        name: string;
        prefix: string;
        created_at: string;
    }>(
        `INSERT INTO api_keys (user_id, name, prefix, key_hash)
     VALUES ($1, $2, $3, $4)
     RETURNING id, user_id, name, prefix, created_at`,
        [userId, name.trim(), prefix, keyHash]
    );

    const record = result.rows[0];
    if (!record) throw new Error("Failed to insert API key");

    await redis.del(cacheKey(userId));

    return ctx.json({ key: rawKey, record }, 201);
});

/**
 * DELETE /api/keys/:id
 * Revoke an API key — only if it belongs to the authenticated user.
 */
keys.delete("/:id", async (ctx) => {
    const userId = ctx.get("userId");
    const id = ctx.req.param("id");

    const existing = await db.query<{ id: string }>(
        "DELETE FROM api_keys WHERE id = $1 AND user_id = $2 RETURNING id",
        [id, userId]
    );

    if (existing.rows.length === 0) {
        return ctx.json({ error: "Key not found or not yours" }, 404);
    }

    await redis.del(cacheKey(userId));
    return ctx.json({ success: true, id });
});

/**
 * GET /keys/compute-usage
 * Fetch compute usage events for the authenticated user (Clerk JWT).
 * Used by the dashboard chart — does not require the raw API key.
 */
keys.get("/compute-usage", async (ctx) => {
    const userId = ctx.get("userId");
    const action = ctx.req.query("action");
    const workflow = ctx.req.query("workflow");
    const status = ctx.req.query("status");
    const limit = Math.min(parseInt(ctx.req.query("limit") ?? "200", 10), 500);
    const offset = parseInt(ctx.req.query("offset") ?? "0", 10);

    const conditions: string[] = ["user_id = $1"];
    const params: unknown[] = [userId];
    let idx = 2;

    if (action) { conditions.push(`action = $${idx++}`); params.push(action); }
    if (workflow) { conditions.push(`workflow = $${idx++}`); params.push(workflow); }
    if (status) { conditions.push(`status = $${idx++}`); params.push(status); }

    const where = conditions.join(" AND ");

    const [dataResult, countResult] = await Promise.all([
        db.query(
            `SELECT id, action, workflow, duration_ms, status, error_msg, metadata, created_at
             FROM compute_usage
             WHERE ${where}
             ORDER BY created_at DESC
             LIMIT $${idx++} OFFSET $${idx}`,
            [...params, limit, offset]
        ),
        db.query(
            `SELECT COUNT(*)::int AS total FROM compute_usage WHERE ${where}`,
            params
        ),
    ]);

    return ctx.json({
        data: dataResult.rows,
        meta: { total: countResult.rows[0]?.total ?? 0, limit, offset },
    });
});

export { keys };

