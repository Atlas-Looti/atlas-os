import { Hono } from "hono";
import { DEMO_POOLS, poolSummary } from "./_data.ts";

/**
 * Token data route
 * GET /atlas-os/dex/tokens/:network/:address
 * Returns token info + top pools containing that token
 */
const tokens = new Hono();

tokens.get("/:network/:address", (ctx) => {
    const network = ctx.req.param("network").toLowerCase();
    const address = ctx.req.param("address").toLowerCase();

    // Find pools where this token appears as base or quote
    const containing = DEMO_POOLS.filter(
        (p) =>
            p.network === network &&
            (p.base_token.address.toLowerCase() === address ||
                p.quote_token.address.toLowerCase() === address)
    );

    if (containing.length === 0) {
        return ctx.json(
            {
                error: `Token not found: ${address} on ${network}.`,
                hint: "Only demo tokens available. GET /atlas-os/dex/pools/:network for a list of pools.",
            },
            404
        );
    }

    // Use the pool where it's base token as price source (prefer that)
    const asBase = containing.find((p) => p.base_token.address.toLowerCase() === address);
    const token = asBase ? asBase.base_token : containing[0]!.quote_token;
    const priceUsd = asBase ? asBase.base_token_price_usd : containing[0]!.quote_token_price_usd;
    const priceChange = asBase ? asBase.price_change_percentage : null;

    return ctx.json({
        data: {
            address,
            network,
            symbol: token.symbol,
            name: token.name,
            image_url: token.image_url,
            price_usd: priceUsd,
            price_change_percentage: priceChange,
            top_pools: containing.slice(0, 5).map(poolSummary),
        },
        _demo: true,
    });
});

export { tokens };
