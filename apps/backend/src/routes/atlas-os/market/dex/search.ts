import { Hono } from "hono";
import { DEMO_POOLS, poolSummary } from "./_data.ts";

/**
 * Search route
 * GET /atlas-os/dex/search?q=keyword&network=eth
 * Search pools by token symbol, token name, or pool name
 * Optional: filter by network
 */
const search = new Hono();

search.get("/", (ctx) => {
    const q = ctx.req.query("q")?.toLowerCase().trim();
    const network = ctx.req.query("network")?.toLowerCase().trim();

    if (!q || q.length < 2) {
        return ctx.json({ error: "Query param 'q' is required (min 2 chars)" }, 400);
    }

    let results = DEMO_POOLS.filter(
        (p) =>
            p.name.toLowerCase().includes(q) ||
            p.base_token.symbol.toLowerCase().includes(q) ||
            p.base_token.name.toLowerCase().includes(q) ||
            p.quote_token.symbol.toLowerCase().includes(q) ||
            p.quote_token.name.toLowerCase().includes(q) ||
            p.dex.name.toLowerCase().includes(q)
    );

    if (network) {
        results = results.filter((p) => p.network === network);
    }

    return ctx.json({
        data: results.map(poolSummary),
        meta: {
            query: q,
            network: network ?? "all",
            total: results.length,
        },
        _demo: true,
    });
});

export { search };
