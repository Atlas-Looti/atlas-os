/**
 * Shared proxy helper for 0x Swap API requests.
 *
 * ## Platform Monetization
 *
 * Atlas OS automatically injects platform fees into every swap request via:
 *   ZERO_EX_FEE_RECIPIENT  — wallet address to receive the platform fee
 *   ZERO_EX_FEE_BPS        — fee in basis points (default: 25 = 0.25%)
 *   ZERO_EX_SURPLUS_RECIPIENT — wallet to receive positive slippage (optional)
 *
 * Behavior:
 * - If a caller already provides `swapFeeRecipient`, Atlas fee is appended
 *   as a second comma-separated recipient (multi-fee feature of 0x v2).
 * - If no fee env vars are set, requests pass through unmodified.
 *
 * Reference: https://0x.org/docs/0x-swap-api/guides/monetize-your-app-using-swap
 */

const ZERO_EX_BASE = "https://api.0x.org";
const ZERO_EX_VERSION = "v2";

export const SWAP_PARAMS = [
    "chainId",
    "buyToken",
    "sellToken",
    "sellAmount",
    "taker",
    "txOrigin",
    "recipient",
    "swapFeeRecipient",
    "swapFeeBps",
    "swapFeeToken",
    "tradeSurplusRecipient",
    "tradeSurplusMaxBps",
    "gasPrice",
    "slippageBps",
    "excludedSources",
    "sellEntireBalance",
] as const;

export type SwapParam = (typeof SWAP_PARAMS)[number];

/**
 * Build upstream URL for a 0x swap endpoint, injecting platform fee params.
 */
export function buildUpstreamUrl(
    path: string,
    callerParams: Partial<Record<SwapParam, string>>,
): URL {
    const url = new URL(`${ZERO_EX_BASE}${path}`);

    // Forward all caller params
    for (const [key, val] of Object.entries(callerParams)) {
        if (val !== undefined) url.searchParams.set(key, val);
    }

    // ── Platform fee injection ────────────────────────────────────────────────
    const feeRecipient = process.env["ZERO_EX_FEE_RECIPIENT"];
    const feeBps = process.env["ZERO_EX_FEE_BPS"] ?? "25"; // 0.25% default

    if (feeRecipient) {
        const existing = url.searchParams.get("swapFeeRecipient");
        const existingBps = url.searchParams.get("swapFeeBps");

        if (existing) {
            // Append as second recipient (multi-fee, comma-separated)
            url.searchParams.set("swapFeeRecipient", `${existing},${feeRecipient}`);
            url.searchParams.set("swapFeeBps", `${existingBps ?? feeBps},${feeBps}`);
        } else {
            url.searchParams.set("swapFeeRecipient", feeRecipient);
            url.searchParams.set("swapFeeBps", feeBps);
        }
    }

    // ── Trade surplus injection ───────────────────────────────────────────────
    const surplusRecipient = process.env["ZERO_EX_SURPLUS_RECIPIENT"];
    if (surplusRecipient && !url.searchParams.has("tradeSurplusRecipient")) {
        url.searchParams.set("tradeSurplusRecipient", surplusRecipient);
    }

    return url;
}

/**
 * Execute a proxied 0x swap request and return the raw Response.
 */
export async function proxySwap(upstreamUrl: URL): Promise<Response> {
    const apiKey = process.env["ZERO_EX_API_KEY"];
    if (!apiKey) throw new Error("ZERO_EX_API_KEY not configured");

    return fetch(upstreamUrl.toString(), {
        headers: {
            "0x-api-key": apiKey,
            "0x-version": ZERO_EX_VERSION,
        },
    });
}

/** Validate required query params; returns first missing name or null. */
export function missingParam(
    params: Partial<Record<SwapParam, string>>,
    required: readonly SwapParam[],
): SwapParam | null {
    for (const p of required) {
        if (!params[p]) return p;
    }
    return null;
}            
