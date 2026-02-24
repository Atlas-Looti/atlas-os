import { Hono } from "hono";
import { DEMO_NETWORKS } from "./_data.ts";

/**
 * GET /atlas-os/dex/networks
 * List all supported demo networks
 */
const networks = new Hono();

networks.get("/", (ctx) => {
    return ctx.json({
        data: DEMO_NETWORKS,
        meta: { total: DEMO_NETWORKS.length },
        _demo: true,
    });
});

export { networks };
