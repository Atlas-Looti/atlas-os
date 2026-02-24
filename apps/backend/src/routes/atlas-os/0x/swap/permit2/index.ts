import { Hono } from "hono";
import { price } from "./price.ts";
import { quote } from "./quote.ts";

/**
 * /atlas-os/0x/swap/permit2/*
 *
 * Routes:
 *   GET /price  → getPrice (Permit2) — indicative price, taker optional
 *   GET /quote  → getQuote (Permit2) — firm quote with transaction + permit2 EIP-712
 */
const permit2 = new Hono();
permit2.route("/price", price);
permit2.route("/quote", quote);

export { permit2 };
