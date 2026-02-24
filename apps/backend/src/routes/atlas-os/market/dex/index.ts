import { Hono } from "hono";
import { networks } from "./networks.ts";
import { trending } from "./trending.ts";
import { pools } from "./pools.ts";
import { tokens } from "./tokens.ts";
import { search } from "./search.ts";

/**
 * /atlas-os/dex — DEX Market Data (Demo)
 *
 * Auth: Atlas API key (atl_xxx) via apiKeyAuth middleware (set upstream in index.ts)
 *
 * Route map:
 *   GET /atlas-os/dex/networks                     → networks.ts
 *   GET /atlas-os/dex/trending                     → trending.ts
 *   GET /atlas-os/dex/trending/:network            → trending.ts
 *   GET /atlas-os/dex/pools/:network               → pools.ts
 *   GET /atlas-os/dex/pools/:network/:address      → pools.ts
 *   GET /atlas-os/dex/tokens/:network/:address     → tokens.ts
 *   GET /atlas-os/dex/search?q=&network=           → search.ts
 */
const dex = new Hono();

dex.route("/networks", networks);
dex.route("/trending", trending);
dex.route("/pools", pools);
dex.route("/tokens", tokens);
dex.route("/search", search);

export { dex };
