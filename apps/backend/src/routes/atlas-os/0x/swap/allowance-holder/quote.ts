import { Hono } from "hono";
import { buildUpstreamUrl, proxySwap, missingParam, SWAP_PARAMS } from "../../_proxy.ts";

const quote = new Hono();

/**
 * GET /atlas-os/0x/swap/allowance-holder/quote
 *
 * Firm quote using Allowance Holder. Response includes transaction calldata.
 * Required: chainId, buyToken, sellToken, sellAmount, taker
 *
 * Platform fee is auto-injected if ZERO_EX_FEE_RECIPIENT is set.
 */
quote.get("/", async (ctx) => {
    const required = ["chainId", "buyToken", "sellToken", "sellAmount", "taker"] as const;
    const params = Object.fromEntries(
        SWAP_PARAMS.map((p) => [p, ctx.req.query(p)]).filter(([, v]) => v !== undefined),
    ) as Record<string, string>;

    const missing = missingParam(params as never, required);
    if (missing) return ctx.json({ error: `'${missing}' is required` }, 400);

    try {
        const url = buildUpstreamUrl("/swap/allowance-holder/quote", params as never);
        const res = await proxySwap(url);
        return ctx.json(await res.json(), res.status as 200 | 400 | 403 | 422 | 500);
    } catch (err) {
        const msg = err instanceof Error ? err.message : "Upstream error";
        return ctx.json({ error: msg }, 503);
    }
});

export { quote };
