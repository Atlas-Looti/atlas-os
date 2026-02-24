import Redis from "ioredis";

if (!process.env["REDIS_URL"]) {
    throw new Error("REDIS_URL is required");
}

export const redis = new Redis(process.env["REDIS_URL"], {
    maxRetriesPerRequest: 3,
    lazyConnect: false,
});

redis.on("connect", () => console.log("[redis] Connected"));
redis.on("error", (err: Error) => console.error("[redis] Error:", err.message));
