import pg from "pg";
const { Pool } = pg;

if (!process.env["DATABASE_URL"]) {
    throw new Error("DATABASE_URL is required");
}

export const db = new Pool({
    connectionString: process.env["DATABASE_URL"],
    ssl: { rejectUnauthorized: false },
    max: 10,
    idleTimeoutMillis: 30_000,
    connectionTimeoutMillis: 5_000,
});

db.on("error", (err: Error) => {
    console.error("[db] Unexpected pool error:", err.message);
});
