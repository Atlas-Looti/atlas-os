import { Hono } from "hono";
import { DEMO_POOLS, poolSummary } from "./_data.ts";

/**
 * Pool data routes
 * GET /atlas-os/dex/pools/:network           — list all pools for a network
 * GET /atlas-os/dex/pools/:network/:address  — full pool detail by address
 */
const pools = new Hono();

pools.get("/:network", (ctx) => {
    const network = ctx.req.param("network").toLowerCase();
    const matching = DEMO_POOLS.filter((p) => p.network === network);

    if (matching.length === 0) {
        return ctx.json(
            { error: `No demo pools for network: "${network}"` },
            404
        );
    }

    return ctx.json({
        data: matching.map(poolSummary),
        meta: { network, total: matching.length, updated_at: new Date().toISOString() },
        _demo: true,
    });
});

pools.get("/:network/:address", (ctx) => {
    const network = ctx.req.param("network").toLowerCase();
    const address = ctx.req.param("address").toLowerCase();

    const pool = DEMO_POOLS.find(
        (p) => p.network === network && p.address.toLowerCase() === address
    );

    if (!pool) {
        return ctx.json(
            {
                error: `Pool not found: ${address} on ${network}.`,
                hint: "Only demo pools are available. GET /atlas-os/dex/pools/:network for the full list.",
            },
            404
        );
    }

    return ctx.json({ data: pool, _demo: true });
});

export { pools };
