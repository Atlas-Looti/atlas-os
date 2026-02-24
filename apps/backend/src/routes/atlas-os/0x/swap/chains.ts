import { Hono } from "hono";

const ZERO_EX_BASE = "https://api.0x.org";
const ZERO_EX_VERSION = "v2";

const chains = new Hono();

/**
 * GET /atlas-os/0x/swap/chains
 *
 * Proxy for https://api.0x.org/swap/chains
 * Returns list of supported chains for swap: [{ chainId, chainName }]
 */
chains.get("/", async (ctx) => {
    const apiKey = process.env["ZERO_EX_API_KEY"];
    if (!apiKey) {
        return ctx.json({ error: "0x API key not configured" }, 503);
    }

    const res = await fetch(`${ZERO_EX_BASE}/swap/chains`, {
        headers: {
            "0x-api-key": apiKey,
            "0x-version": ZERO_EX_VERSION,
        },
    });

    const body = await res.json();
    return ctx.json(body, res.status as 200 | 500);
});

export { chains };
