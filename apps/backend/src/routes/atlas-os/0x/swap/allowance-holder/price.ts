import { Hono } from "hono";
import { buildUpstreamUrl, proxySwap, missingParam, SWAP_PARAMS } from "../../_proxy.ts";

const price = new Hono();

/**
 * GET /atlas-os/0x/swap/allowance-holder/price
 *
 * Indicative price using Allowance Holder to set allowances.
 * Required: chainId, buyToken, sellToken, sellAmount
 * Optional: taker, txOrigin, recipient, swapFeeRecipient, …
 *
 * Platform fee is auto-injected if ZERO_EX_FEE_RECIPIENT is set.
 */
price.get("/", async (ctx) => {
    const required = ["chainId", "buyToken", "sellToken", "sellAmount"] as const;
    const params = Object.fromEntries(
        SWAP_PARAMS.map((p) => [p, ctx.req.query(p)]).filter(([, v]) => v !== undefined),
    ) as Record<string, string>;

    const missing = missingParam(params as never, required);
    if (missing) return ctx.json({ error: `'${missing}' is required` }, 400);

    try {
        const url = buildUpstreamUrl("/swap/allowance-holder/price", params as never);
        const res = await proxySwap(url);
        return ctx.json(await res.json(), res.status as 200 | 400 | 403 | 422 | 500);
    } catch (err) {
        const msg = err instanceof Error ? err.message : "Upstream error";
        return ctx.json({ error: msg }, 503);
    }
});

export { price };
