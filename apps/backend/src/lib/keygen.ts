import { createHash, randomBytes } from "crypto";

/** Generate a new Atlas API key: atl_<32 hex bytes> */
export function generateApiKey(): string {
    const raw = randomBytes(32).toString("hex");
    return `atl_${raw}`;
}

/** Return the first 8 chars of the key after the prefix (atl_xxxxxxxx...) */
export function keyPrefix(key: string): string {
    return key.slice(0, 12); // "atl_" + 8 chars
}

/** SHA-256 hash of the raw key for safe storage */
export function hashKey(key: string): string {
    return createHash("sha256").update(key).digest("hex");
}
