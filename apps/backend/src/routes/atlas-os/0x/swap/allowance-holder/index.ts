import { Hono } from "hono";
import { price } from "./price.ts";
import { quote } from "./quote.ts";

/**
 * /atlas-os/0x/swap/allowance-holder/*
 *
 * Routes:
 *   GET /price  → getPrice (Allowance Holder) — indicative price, no taker required
 *   GET /quote  → getQuote (Allowance Holder) — firm quote with transaction calldata
 */
const allowanceHolder = new Hono();
allowanceHolder.route("/price", price);
allowanceHolder.route("/quote", quote);

export { allowanceHolder };
