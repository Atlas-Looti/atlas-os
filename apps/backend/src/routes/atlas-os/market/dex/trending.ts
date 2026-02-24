import { Hono } from "hono";
import { DEMO_POOLS, poolSummary } from "./_data.ts";

/**
 * Trending pools routes
 * GET /atlas-os/dex/trending           — across all networks (sorted by trending_score)
 * GET /atlas-os/dex/trending/:network  — filtered by network
 */
const trending = new Hono();

trending.get("/", (ctx) => {
    const sorted = [...DEMO_POOLS].sort((a, b) => b.trending_score - a.trending_score);
    return ctx.json({
        data: sorted.map(poolSummary),
        meta: { total: sorted.length, updated_at: new Date().toISOString() },
        _demo: true,
    });
});

trending.get("/:network", (ctx) => {
    const network = ctx.req.param("network").toLowerCase();
    const matching = DEMO_POOLS.filter((p) => p.network === network);

    if (matching.length === 0) {
        return ctx.json(
            { error: `No demo data for network: "${network}"` },
            404
        );
    }

    const sorted = [...matching].sort((a, b) => b.trending_score - a.trending_score);
    return ctx.json({
        data: sorted.map(poolSummary),
        meta: { network, total: sorted.length, updated_at: new Date().toISOString() },
        _demo: true,
    });
});

export { trending };
