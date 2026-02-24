import { db } from "./db.ts";

/**
 * Runs all migrations on startup.
 * Add new migrations as additional entries in the MIGRATIONS array.
 * Each migration only runs once (tracked by name in the schema_migrations table).
 */
const MIGRATIONS: { name: string; sql: string }[] = [
    {
        name: "001_api_keys",
        sql: `
      CREATE TABLE IF NOT EXISTS api_keys (
        id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id     TEXT NOT NULL,
        name        TEXT NOT NULL,
        prefix      TEXT NOT NULL,
        key_hash    TEXT NOT NULL,
        created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
      );
      CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys (user_id);
    `,
    },
];

export async function runMigrations() {
    // Ensure tracking table exists
    await db.query(`
    CREATE TABLE IF NOT EXISTS schema_migrations (
      name        TEXT PRIMARY KEY,
      applied_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
    )
  `);

    for (const migration of MIGRATIONS) {
        const { rows } = await db.query(
            "SELECT name FROM schema_migrations WHERE name = $1",
            [migration.name]
        );

        if (rows.length > 0) continue; // already applied

        console.log(`[migrate] Applying: ${migration.name}`);
        await db.query(migration.sql);
        await db.query(
            "INSERT INTO schema_migrations (name) VALUES ($1)",
            [migration.name]
        );
        console.log(`[migrate] Done: ${migration.name}`);
    }

    console.log("[migrate] All migrations up to date.");
}
