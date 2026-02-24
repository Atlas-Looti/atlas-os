import { Hono } from "hono";

/**
 * Maps user-friendly chain aliases → Alchemy network slug.
 * Slug is used to build the RPC URL: https://{slug}.g.alchemy.com/v2/KEY
 *
 * Special URL patterns:
 *   - "starknet" template: https://{slug}.g.alchemy.com/starknet/version/rpc/v0_10/KEY
 */
type UrlTemplate = "v2" | "starknet";

interface ChainDef {
    slug: string;
    template?: UrlTemplate; // defaults to "v2"
}

const CHAINS: Record<string, ChainDef> = {
    // ── Ethereum ──────────────────────────────────────────────
    "eth": { slug: "eth-mainnet" },
    "eth-mainnet": { slug: "eth-mainnet" },
    "ethereum": { slug: "eth-mainnet" },
    "eth-sepolia": { slug: "eth-sepolia" },
    "eth-holesky": { slug: "eth-holesky" },
    "eth-hoodi": { slug: "eth-hoodi" },

    // ── Arbitrum ──────────────────────────────────────────────
    "arb": { slug: "arb-mainnet" },
    "arbitrum": { slug: "arb-mainnet" },
    "arb-mainnet": { slug: "arb-mainnet" },
    "arb-sepolia": { slug: "arb-sepolia" },
    "arbnova": { slug: "arbnova-mainnet" },
    "arbnova-mainnet": { slug: "arbnova-mainnet" },
    "arbitrum-nova": { slug: "arbnova-mainnet" },

    // ── Base ──────────────────────────────────────────────────
    "base": { slug: "base-mainnet" },
    "base-mainnet": { slug: "base-mainnet" },
    "base-sepolia": { slug: "base-sepolia" },

    // ── OP / Optimism ─────────────────────────────────────────
    "op": { slug: "opt-mainnet" },
    "optimism": { slug: "opt-mainnet" },
    "opt-mainnet": { slug: "opt-mainnet" },
    "opt-sepolia": { slug: "opt-sepolia" },

    // ── Polygon ───────────────────────────────────────────────
    "polygon": { slug: "polygon-mainnet" },
    "matic": { slug: "polygon-mainnet" },
    "polygon-mainnet": { slug: "polygon-mainnet" },
    "polygon-amoy": { slug: "polygon-amoy" },
    "polygonzkevm": { slug: "polygonzkevm-mainnet" },
    "polygon-zkevm": { slug: "polygonzkevm-mainnet" },
    "polygonzkevm-mainnet": { slug: "polygonzkevm-mainnet" },
    "polygonzkevm-cardona": { slug: "polygonzkevm-cardona" },

    // ── Avalanche ─────────────────────────────────────────────
    "avax": { slug: "avax-mainnet" },
    "avalanche": { slug: "avax-mainnet" },
    "avax-mainnet": { slug: "avax-mainnet" },
    "avax-fuji": { slug: "avax-fuji" },

    // ── BNB / BSC ─────────────────────────────────────────────
    "bsc": { slug: "bnb-mainnet" },
    "bnb": { slug: "bnb-mainnet" },
    "bnb-mainnet": { slug: "bnb-mainnet" },
    "bnb-testnet": { slug: "bnb-testnet" },
    "opbnb": { slug: "opbnb-mainnet" },
    "opbnb-mainnet": { slug: "opbnb-mainnet" },
    "opbnb-testnet": { slug: "opbnb-testnet" },

    // ── Solana ────────────────────────────────────────────────
    "solana": { slug: "solana-mainnet" },
    "solana-mainnet": { slug: "solana-mainnet" },
    "solana-devnet": { slug: "solana-devnet" },

    // ── Starknet (special URL) ────────────────────────────────
    "starknet": { slug: "starknet-mainnet", template: "starknet" },
    "starknet-mainnet": { slug: "starknet-mainnet", template: "starknet" },
    "starknet-sepolia": { slug: "starknet-sepolia", template: "starknet" },

    // ── ZKsync ────────────────────────────────────────────────
    "zksync": { slug: "zksync-mainnet" },
    "zksync-mainnet": { slug: "zksync-mainnet" },
    "zksync-sepolia": { slug: "zksync-sepolia" },

    // ── Linea ─────────────────────────────────────────────────
    "linea": { slug: "linea-mainnet" },
    "linea-mainnet": { slug: "linea-mainnet" },
    "linea-sepolia": { slug: "linea-sepolia" },

    // ── Blast ─────────────────────────────────────────────────
    "blast": { slug: "blast-mainnet" },
    "blast-mainnet": { slug: "blast-mainnet" },
    "blast-sepolia": { slug: "blast-sepolia" },

    // ── Mantle ────────────────────────────────────────────────
    "mantle": { slug: "mantle-mainnet" },
    "mantle-mainnet": { slug: "mantle-mainnet" },
    "mantle-sepolia": { slug: "mantle-sepolia" },

    // ── Scroll ────────────────────────────────────────────────
    "scroll": { slug: "scroll-mainnet" },
    "scroll-mainnet": { slug: "scroll-mainnet" },
    "scroll-sepolia": { slug: "scroll-sepolia" },

    // ── Berachain ─────────────────────────────────────────────
    "bera": { slug: "berachain-mainnet" },
    "berachain": { slug: "berachain-mainnet" },
    "berachain-mainnet": { slug: "berachain-mainnet" },
    "berachain-bepolia": { slug: "berachain-bepolia" },

    // ── Celo ──────────────────────────────────────────────────
    "celo": { slug: "celo-mainnet" },
    "celo-mainnet": { slug: "celo-mainnet" },
    "celo-sepolia": { slug: "celo-sepolia" },

    // ── Unichain ──────────────────────────────────────────────
    "unichain": { slug: "unichain-mainnet" },
    "unichain-mainnet": { slug: "unichain-mainnet" },
    "unichain-sepolia": { slug: "unichain-sepolia" },

    // ── Base-family ───────────────────────────────────────────
    "world-chain": { slug: "worldchain-mainnet" },
    "worldchain": { slug: "worldchain-mainnet" },
    "worldchain-mainnet": { slug: "worldchain-mainnet" },
    "worldchain-sepolia": { slug: "worldchain-sepolia" },

    // ── HyperEVM ──────────────────────────────────────────────
    "hyperevm": { slug: "hyperliquid-mainnet" },
    "hyperliquid": { slug: "hyperliquid-mainnet" },
    "hyperliquid-mainnet": { slug: "hyperliquid-mainnet" },
    "hyperliquid-testnet": { slug: "hyperliquid-testnet" },

    // ── Sonic ─────────────────────────────────────────────────
    "sonic": { slug: "sonic-mainnet" },
    "sonic-mainnet": { slug: "sonic-mainnet" },
    "sonic-testnet": { slug: "sonic-testnet" },
    "sonic-blaze": { slug: "sonic-blaze" },

    // ── Sei ───────────────────────────────────────────────────
    "sei": { slug: "sei-mainnet" },
    "sei-mainnet": { slug: "sei-mainnet" },
    "sei-testnet": { slug: "sei-testnet" },

    // ── Monad ─────────────────────────────────────────────────
    "monad": { slug: "monad-mainnet" },
    "monad-mainnet": { slug: "monad-mainnet" },
    "monad-testnet": { slug: "monad-testnet" },

    // ── Ink ───────────────────────────────────────────────────
    "ink": { slug: "ink-mainnet" },
    "ink-mainnet": { slug: "ink-mainnet" },
    "ink-sepolia": { slug: "ink-sepolia" },

    // ── Lens ──────────────────────────────────────────────────
    "lens": { slug: "lens-mainnet" },
    "lens-mainnet": { slug: "lens-mainnet" },
    "lens-sepolia": { slug: "lens-sepolia" },

    // ── Gnosis ────────────────────────────────────────────────
    "gnosis": { slug: "gnosis-mainnet" },
    "gnosis-mainnet": { slug: "gnosis-mainnet" },
    "gnosis-chiado": { slug: "gnosis-chiado" },

    // ── Metis ─────────────────────────────────────────────────
    "metis": { slug: "metis-mainnet" },
    "metis-mainnet": { slug: "metis-mainnet" },

    // ── Moonbeam ──────────────────────────────────────────────
    "moonbeam": { slug: "moonbeam-mainnet" },
    "moonbeam-mainnet": { slug: "moonbeam-mainnet" },

    // ── Zora ──────────────────────────────────────────────────
    "zora": { slug: "zora-mainnet" },
    "zora-mainnet": { slug: "zora-mainnet" },
    "zora-sepolia": { slug: "zora-sepolia" },

    // ── Mode ──────────────────────────────────────────────────
    "mode": { slug: "mode-mainnet" },
    "mode-mainnet": { slug: "mode-mainnet" },
    "mode-sepolia": { slug: "mode-sepolia" },

    // ── Astar ─────────────────────────────────────────────────
    "astar": { slug: "astar-mainnet" },
    "astar-mainnet": { slug: "astar-mainnet" },

    // ── ZetaChain ─────────────────────────────────────────────
    "zetachain": { slug: "zetachain-mainnet" },
    "zetachain-mainnet": { slug: "zetachain-mainnet" },
    "zetachain-testnet": { slug: "zetachain-testnet" },

    // ── Soneium ───────────────────────────────────────────────
    "soneium": { slug: "soneium-mainnet" },
    "soneium-mainnet": { slug: "soneium-mainnet" },
    "soneium-minato": { slug: "soneium-minato" },

    // ── Abstract ──────────────────────────────────────────────
    "abstract": { slug: "abstract-mainnet" },
    "abstract-mainnet": { slug: "abstract-mainnet" },
    "abstract-testnet": { slug: "abstract-testnet" },

    // ── Anime ─────────────────────────────────────────────────
    "anime": { slug: "anime-mainnet" },
    "anime-mainnet": { slug: "anime-mainnet" },
    "anime-sepolia": { slug: "anime-sepolia" },

    // ── ApeChain ──────────────────────────────────────────────
    "apechain": { slug: "apechain-mainnet" },
    "apechain-mainnet": { slug: "apechain-mainnet" },
    "apechain-curtis": { slug: "apechain-curtis" },

    // ── Aptos ─────────────────────────────────────────────────
    "aptos": { slug: "aptos-mainnet" },
    "aptos-mainnet": { slug: "aptos-mainnet" },
    "aptos-testnet": { slug: "aptos-testnet" },

    // ── Story ─────────────────────────────────────────────────
    "story": { slug: "story-mainnet" },
    "story-mainnet": { slug: "story-mainnet" },
    "story-aeneid": { slug: "story-aeneid" },

    // ── Superseed ─────────────────────────────────────────────
    "superseed": { slug: "superseed-mainnet" },
    "superseed-mainnet": { slug: "superseed-mainnet" },
    "superseed-sepolia": { slug: "superseed-sepolia" },

    // ── Flow ──────────────────────────────────────────────────
    "flow": { slug: "flow-mainnet" },
    "flow-mainnet": { slug: "flow-mainnet" },
    "flow-testnet": { slug: "flow-testnet" },

    // ── Frax ──────────────────────────────────────────────────
    "frax": { slug: "frax-mainnet" },
    "frax-mainnet": { slug: "frax-mainnet" },
    "frax-sepolia": { slug: "frax-sepolia" },

    // ── BOB ───────────────────────────────────────────────────
    "bob": { slug: "bob-mainnet" },
    "bob-mainnet": { slug: "bob-mainnet" },
    "bob-sepolia": { slug: "bob-sepolia" },

    // ── CrossFi ───────────────────────────────────────────────
    "crossfi": { slug: "crossfi-mainnet" },
    "crossfi-mainnet": { slug: "crossfi-mainnet" },
    "crossfi-testnet": { slug: "crossfi-testnet" },

    // ── Rootstock ─────────────────────────────────────────────
    "rootstock": { slug: "rootstock-mainnet" },
    "rootstock-mainnet": { slug: "rootstock-mainnet" },
    "rootstock-testnet": { slug: "rootstock-testnet" },

    // ── Shape ─────────────────────────────────────────────────
    "shape": { slug: "shape-mainnet" },
    "shape-mainnet": { slug: "shape-mainnet" },
    "shape-sepolia": { slug: "shape-sepolia" },

    // ── Botanix ───────────────────────────────────────────────
    "botanix": { slug: "botanix-mainnet" },
    "botanix-mainnet": { slug: "botanix-mainnet" },
    "botanix-testnet": { slug: "botanix-testnet" },

    // ── Degen ─────────────────────────────────────────────────
    "degen": { slug: "degen-mainnet" },
    "degen-mainnet": { slug: "degen-mainnet" },
    "degen-sepolia": { slug: "degen-sepolia" },

    // ── Bitcoin ───────────────────────────────────────────────
    "bitcoin": { slug: "bitcoin-mainnet" },
    "btc": { slug: "bitcoin-mainnet" },
    "bitcoin-mainnet": { slug: "bitcoin-mainnet" },
    "bitcoin-testnet": { slug: "bitcoin-testnet" },
    "bitcoin-signet": { slug: "bitcoin-signet" },

    // ── SUI ───────────────────────────────────────────────────
    "sui": { slug: "sui-mainnet" },
    "sui-mainnet": { slug: "sui-mainnet" },
    "sui-testnet": { slug: "sui-testnet" },

    // ── Unichain / Syndicate / Misc ────────────────────────────
    "ronin": { slug: "ronin-mainnet" },
    "ronin-mainnet": { slug: "ronin-mainnet" },
    "ronin-saigon": { slug: "ronin-saigon" },
    "boba": { slug: "boba-mainnet" },
    "boba-mainnet": { slug: "boba-mainnet" },
    "boba-sepolia": { slug: "boba-sepolia" },
    "megaeth": { slug: "megaeth-mainnet" },
    "megaeth-mainnet": { slug: "megaeth-mainnet" },
    "megaeth-testnet": { slug: "megaeth-testnet" },
    "polynomial": { slug: "polynomial-mainnet" },
    "polynomial-mainnet": { slug: "polynomial-mainnet" },
    "polynomial-sepolia": { slug: "polynomial-sepolia" },
    "tron": { slug: "tron-mainnet" },
    "tron-mainnet": { slug: "tron-mainnet" },
    "tron-testnet": { slug: "tron-testnet" },
    "clankermon": { slug: "clankermon-mainnet" },
    "clankermon-mainnet": { slug: "clankermon-mainnet" },
    "humanity": { slug: "humanity-mainnet" },
    "humanity-mainnet": { slug: "humanity-mainnet" },
    "humanity-testnet": { slug: "humanity-testnet" },
    "galactica": { slug: "galactica-mainnet" },
    "galactica-mainnet": { slug: "galactica-mainnet" },
    "galactica-cassiopeia": { slug: "galactica-cassiopeia" },
    "scroll-race": { slug: "race-mainnet" },
    "race": { slug: "race-mainnet" },
    "race-mainnet": { slug: "race-mainnet" },
    "race-sepolia": { slug: "race-sepolia" },
} as const;

if (!process.env["ALCHEMY_API_KEY"]) {
    throw new Error("ALCHEMY_API_KEY is required");
}
const ALCHEMY_KEY = process.env["ALCHEMY_API_KEY"];

function buildAlchemyUrl(chain: ChainDef): string {
    switch (chain.template) {
        case "starknet":
            return `https://${chain.slug}.g.alchemy.com/starknet/version/rpc/v0_10/${ALCHEMY_KEY}`;
        default:
            return `https://${chain.slug}.g.alchemy.com/v2/${ALCHEMY_KEY}`;
    }
}

const rpc = new Hono();

/**
 * GET /atlas-os/rpc — list all supported chain aliases
 */
rpc.get("/", (ctx) => {
    const chains = [...new Set(Object.keys(CHAINS))].sort();
    return ctx.json({ chains });
});

/**
 * POST /atlas-os/rpc/:chain
 * Proxies JSON-RPC requests to Alchemy for the given chain.
 * Auth: atl_xxx API key (enforced by apiKeyAuth middleware upstream).
 */
rpc.post("/:chain", async (ctx) => {
    const alias = ctx.req.param("chain").toLowerCase();
    const chain = CHAINS[alias];

    if (!chain) {
        return ctx.json(
            {
                error: `Unknown chain: "${alias}". GET /atlas-os/rpc for full list.`,
            },
            400
        );
    }

    const url = buildAlchemyUrl(chain);
    const body = await ctx.req.text();

    const upstream = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body,
    });

    const data = await upstream.json();

    if (!upstream.ok) {
        ctx.status(upstream.status as Parameters<typeof ctx.status>[0]);
    }

    return ctx.json(data);
});

export { rpc };
