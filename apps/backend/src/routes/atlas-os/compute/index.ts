import { Hono } from "hono";
import { db } from "../../../lib/db.ts";
import { usage } from "./usage.ts";

/**
 * Ensures the compute_usage table exists.
 * Called once at server startup — no SQL migration file needed.
 */
export async function setupComputeUsage(): Promise<void> {
    await db.query(`
        CREATE TABLE IF NOT EXISTS compute_usage (
            id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id     TEXT        NOT NULL,
            api_key_id  UUID        REFERENCES api_keys(id) ON DELETE SET NULL,
            action      TEXT        NOT NULL,
            workflow    TEXT,
            duration_ms INTEGER,
            status      TEXT        NOT NULL DEFAULT 'success'
                                    CHECK (status IN ('success', 'error', 'pending')),
            error_msg   TEXT,
            metadata    JSONB       NOT NULL DEFAULT '{}',
            created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
        );

        DO $$ BEGIN
            IF EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'compute_usage' AND column_name = 'command'
            ) THEN
                ALTER TABLE compute_usage RENAME COLUMN command TO action;
            END IF;
        END $$;

        CREATE INDEX IF NOT EXISTS idx_compute_usage_user_id    ON compute_usage(user_id);
        CREATE INDEX IF NOT EXISTS idx_compute_usage_api_key_id ON compute_usage(api_key_id);
        CREATE INDEX IF NOT EXISTS idx_compute_usage_created_at ON compute_usage(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_compute_usage_action     ON compute_usage(action);
        CREATE INDEX IF NOT EXISTS idx_compute_usage_workflow   ON compute_usage(workflow);
    `);
    console.log("[compute] Table ready.");
}

/**
 * /atlas-os/compute — Compute & Workflow Tracking
 *
 * Auth: Atlas API key (atl_xxx) — set upstream by apiKeyAuth middleware
 *
 * Routes:
 *   POST /atlas-os/compute/usage   → record event
 *   GET  /atlas-os/compute/usage   → list history
 */
const compute = new Hono();
compute.route("/usage", usage);

export { compute };
