import { Hono } from "hono";

const health = new Hono();

health.get("/", (ctx) => {
    return ctx.json({
        status: "ok",
        timestamp: new Date().toISOString(),
        service: "atlas-os-backend",
        version: "0.1.0",
    });
});

export { health };
