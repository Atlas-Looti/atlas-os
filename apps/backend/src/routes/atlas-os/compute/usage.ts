import { Hono } from "hono";
import { db } from "../../../lib/db.ts";

/**
 * Compute Usage Routes
 *
 * POST /atlas-os/compute/usage   — record a CLI / workflow usage event
 * GET  /atlas-os/compute/usage   — list usage history for the authenticated key owner
 *
 * Auth: Atlas API key (atl_xxx) set in context by apiKeyAuth middleware upstream.
 * Context vars: userId (TEXT), apiKeyId (UUID)
 */

interface UsageBody {
    action: string;           // required — e.g. 'perp.trade', 'market.watch'
    workflow?: string;        // optional — e.g. 'trading', 'arbitrage'
    duration_ms?: number;
    status?: "success" | "error" | "pending";
    error_msg?: string;
    metadata?: Record<string, unknown>;
}

type Variables = { userId: string; apiKeyId: string };

const usage = new Hono<{ Variables: Variables }>();

// ── POST /atlas-os/compute/usage ─────────────────────────────────────────────

usage.post("/", async (ctx) => {
    const userId = ctx.get("userId") as string;
    const apiKeyId = ctx.get("apiKeyId") as string;

    let body: UsageBody;
    try {
        body = await ctx.req.json<UsageBody>();
    } catch {
        return ctx.json({ error: "Invalid JSON body" }, 400);
    }

    const { action, workflow, duration_ms, status = "success", error_msg, metadata = {} } = body;

    if (!action || typeof action !== "string") {
        return ctx.json({ error: "'action' is required (string)" }, 400);
    }

    if (status === "error" && !error_msg) {
        return ctx.json({ error: "'error_msg' is required when status is 'error'" }, 400);
    }

    const { rows } = await db.query<{ id: string; created_at: string }>(
        `INSERT INTO compute_usage
            (user_id, api_key_id, action, workflow, duration_ms, status, error_msg, metadata)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, created_at`,
        [
            userId,
            apiKeyId ?? null,
            action,
            workflow ?? null,
            duration_ms ?? null,
            status,
            error_msg ?? null,
            JSON.stringify(metadata),
        ]
    );

    return ctx.json({ data: rows[0] }, 201);
});

// ── GET /atlas-os/compute/usage ──────────────────────────────────────────────

usage.get("/", async (ctx) => {
    const userId = ctx.get("userId") as string;

    const action = ctx.req.query("action");
    const workflow = ctx.req.query("workflow");
    const status = ctx.req.query("status");
    const limit = Math.min(parseInt(ctx.req.query("limit") ?? "50", 10), 200);
    const offset = parseInt(ctx.req.query("offset") ?? "0", 10);

    const conditions: string[] = ["user_id = $1"];
    const params: unknown[] = [userId];
    let idx = 2;

    if (action) {
        conditions.push(`action = $${idx++}`);
        params.push(action);
    }
    if (workflow) {
        conditions.push(`workflow = $${idx++}`);
        params.push(workflow);
    }
    if (status) {
        conditions.push(`status = $${idx++}`);
        params.push(status);
    }

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
        meta: {
            total: countResult.rows[0]?.total ?? 0,
            limit,
            offset,
        },
    });
});

export { usage };
