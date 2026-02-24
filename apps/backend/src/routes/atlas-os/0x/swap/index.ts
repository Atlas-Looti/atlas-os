import { Hono } from "hono";
import { allowanceHolder } from "./allowance-holder/index.ts";
import { permit2 } from "./permit2/index.ts";
import { chains } from "./chains.ts";

/**
 * /atlas-os/0x/swap/*
 *
 * Routes:
 *   GET /chains                  → getChains
 *   /allowance-holder/price      → getPrice (Allowance Holder)
 *   /allowance-holder/quote      → getQuote (Allowance Holder)
 *   /permit2/price               → getPrice (Permit2)
 *   /permit2/quote               → getQuote (Permit2)
 */
const swap = new Hono();
swap.route("/chains", chains);
swap.route("/allowance-holder", allowanceHolder);
swap.route("/permit2", permit2);

export { swap };
