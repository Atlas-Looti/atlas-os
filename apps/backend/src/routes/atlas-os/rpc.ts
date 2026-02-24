import { Hono } from "hono";

/** Alchemy network slugs per chain alias */
const CHAIN_MAP: Record<string, string> = {
    // Ethereum
    eth: "eth-mainnet",
    ethereum: "eth-mainnet",
    "eth-mainnet": "eth-mainnet",
    // Arbitrum
    arb: "arb-mainnet",
    arbitrum: "arb-mainnet",
    "arb-mainnet": "arb-mainnet",
    // Base
    base: "base-mainnet",
    "base-mainnet": "base-mainnet",
    // Optimism
    op: "opt-mainnet",
    optimism: "opt-mainnet",
    "opt-mainnet": "opt-mainnet",
    // Polygon
    polygon: "polygon-mainnet",
    matic: "polygon-mainnet",
    "polygon-mainnet": "polygon-mainnet",
    // Avalanche
    avax: "avax-mainnet",
    avalanche: "avax-mainnet",
    "avax-mainnet": "avax-mainnet",
};

if (!process.env["ALCHEMY_API_KEY"]) {
    throw new Error("ALCHEMY_API_KEY is required");
}
const ALCHEMY_KEY = process.env["ALCHEMY_API_KEY"];

const rpc = new Hono();

/**
 * GET /atlas-os/rpc — list supported chains
 */
rpc.get("/", (ctx) => {
    return ctx.json({ chains: Object.keys(CHAIN_MAP) });
});

/**
 * POST /atlas-os/rpc/:chain
 *
 * Proxies JSON-RPC requests to Alchemy for the given chain.
 * Auth: atl_xxx API key (checked by apiKeyAuth middleware).
 *
 * Example (Atlas CLI):
 *   POST /atlas-os/rpc/eth
 *   Authorization: Bearer atl_xxxxxxxx
 *   Content-Type: application/json
 *   { "jsonrpc": "2.0", "method": "eth_blockNumber", "id": 1 }
 */
rpc.post("/:chain", async (ctx) => {
    const chain = ctx.req.param("chain").toLowerCase();
    const network = CHAIN_MAP[chain];

    if (!network) {
        return ctx.json(
            { error: `Unknown chain: "${chain}". Supported: ${Object.keys(CHAIN_MAP).join(", ")}` },
            400
        );
    }

    const alchemyUrl = `https://${network}.g.alchemy.com/v2/${ALCHEMY_KEY}`;
    const body = await ctx.req.text();

    const upstream = await fetch(alchemyUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body,
    });

    const data = await upstream.json();

    if (!upstream.ok) {
        ctx.status(upstream.status as Parameters<typeof ctx.status>[0]);
    }

    return ctx.json(data);
});

export { rpc };
