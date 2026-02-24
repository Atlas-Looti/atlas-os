import type { Context, Next } from "hono";

export async function errorHandler(ctx: Context, next: Next) {
    try {
        await next();
    } catch (err) {
        const message = err instanceof Error ? err.message : "Internal server error";
        console.error("[error]", message, err);
        ctx.status(500);
        return ctx.json({ error: message });
    }
}
