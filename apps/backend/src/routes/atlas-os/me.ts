import { Hono } from "hono";
import { createClerkClient } from "@clerk/backend";
import { db } from "../../lib/db.ts";

if (!process.env["CLERK_SECRET_KEY"]) {
    throw new Error("CLERK_SECRET_KEY not configured");
}

const clerk = createClerkClient({ secretKey: process.env["CLERK_SECRET_KEY"] });

const me = new Hono<{ Variables: { userId: string; apiKeyId: string } }>();

/**
 * GET /atlas-os/me
 * Returns the authenticated user's profile.
 * Auth: Atlas API key (atl_xxx) via apiKeyAuth middleware.
 *
 * Response:
 * {
 *   id: string           — Clerk user ID
 *   username: string | null
 *   email: string | null — primary email address
 *   name: string | null  — full name
 *   avatar: string | null — profile image URL
 *   api_key_id: string   — which key was used
 *   created_at: string   — account created at (ISO)
 * }
 */
me.get("/", async (ctx) => {
    const userId = ctx.get("userId");
    const apiKeyId = ctx.get("apiKeyId");

    const [user, keyMeta] = await Promise.all([
        clerk.users.getUser(userId),
        db.query<{ name: string; prefix: string; created_at: string }>(
            "SELECT name, prefix, created_at FROM api_keys WHERE id = $1",
            [apiKeyId]
        ),
    ]);

    const primaryEmail =
        user.emailAddresses.find((e) => e.id === user.primaryEmailAddressId)
            ?.emailAddress ?? null;

    const key = keyMeta.rows[0];

    return ctx.json({
        id: user.id,
        username: user.username,
        email: primaryEmail,
        name: [user.firstName, user.lastName].filter(Boolean).join(" ") || null,
        avatar: user.imageUrl,
        created_at: new Date(user.createdAt).toISOString(),
        api_key: key
            ? { id: apiKeyId, name: key.name, prefix: key.prefix, created_at: key.created_at }
            : null,
    });
});

export { me };
