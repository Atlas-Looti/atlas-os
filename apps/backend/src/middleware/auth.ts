import { verifyToken } from "@clerk/backend";
import type { Context, Next } from "hono";

if (!process.env["CLERK_SECRET_KEY"]) {
    throw new Error("CLERK_SECRET_KEY is required");
}

const SECRET_KEY = process.env["CLERK_SECRET_KEY"];

/**
 * Clerk JWT auth middleware.
 * Verifies the Bearer token in Authorization header using @clerk/backend verifyToken.
 * Stores the verified Clerk userId in ctx.var.userId via ctx.set("userId", ...).
 */
export async function clerkAuth(ctx: Context, next: Next) {
    const authHeader = ctx.req.header("Authorization");
    const token = authHeader?.startsWith("Bearer ")
        ? authHeader.slice(7)
        : undefined;

    if (!token) {
        return ctx.json({ error: "Unauthorized" }, 401);
    }

    try {
        const payload = await verifyToken(token, { secretKey: SECRET_KEY });
        ctx.set("userId", payload.sub);
        await next();
    } catch {
        return ctx.json({ error: "Invalid or expired token" }, 401);
    }
}
