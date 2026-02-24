import { Hono } from "hono";
import { usage } from "./usage.ts";

/**
 * /atlas-os/compute — Compute & Workflow Tracking
 *
 * Auth: Atlas API key (atl_xxx) — set upstream by apiKeyAuth middleware
 *
 * Routes:
 *   POST /atlas-os/compute/usage   → record event
 *   GET  /atlas-os/compute/usage   → list history
 */
const compute = new Hono();
compute.route("/usage", usage);

export { compute };
