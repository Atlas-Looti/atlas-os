import { cors } from "hono/cors";

// CORS terbuka — auth security ditangani Clerk JWT & Atlas API key.
export const corsMiddleware = cors({
    origin: "*",
    allowMethods: ["GET", "POST", "DELETE", "PUT", "PATCH", "OPTIONS"],
    allowHeaders: ["Content-Type", "Authorization", "X-API-Key"],
});
