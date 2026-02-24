import { Hono } from "hono";
import { swap } from "./swap/index.ts";

/**
 * /atlas-os/0x — 0x Protocol Proxy
 *
 * Auth: Atlas API key (atl_xxx) — set upstream by apiKeyAuth middleware
 *
 * Routes:
 *   GET /swap/allowance-holder/price  → getPrice (Allowance Holder)
 */
const zerox = new Hono();
zerox.route("/swap", swap);

export { zerox };
