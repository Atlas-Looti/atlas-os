/**
 * _data.ts — Shared demo data for /atlas-os/dex/*
 * Single source of truth; imported by all sub-route files.
 */

export const DEMO_NETWORKS = [
    { id: "eth", name: "Ethereum", coingecko_asset_platform_id: "ethereum", icon: "🔷" },
    { id: "base", name: "Base", coingecko_asset_platform_id: "base", icon: "🔵" },
    { id: "arb", name: "Arbitrum", coingecko_asset_platform_id: "arbitrum-one", icon: "🟦" },
    { id: "polygon", name: "Polygon PoS", coingecko_asset_platform_id: "polygon-pos", icon: "🟣" },
    { id: "solana", name: "Solana", coingecko_asset_platform_id: "solana", icon: "🟢" },
    { id: "bsc", name: "BNB Smart Chain", coingecko_asset_platform_id: "binance-smart-chain", icon: "🟡" },
    { id: "avax", name: "Avalanche", coingecko_asset_platform_id: "avalanche", icon: "🔴" },
    { id: "op", name: "Optimism", coingecko_asset_platform_id: "optimistic-ethereum", icon: "⭕" },
] as const;

export interface DemoPool {
    id: string;
    network: string;
    address: string;
    name: string;
    dex: { id: string; name: string };
    base_token: {
        address: string;
        symbol: string;
        name: string;
        image_url: string | null;
    };
    quote_token: {
        address: string;
        symbol: string;
        name: string;
        image_url: string | null;
    };
    base_token_price_usd: string;
    base_token_price_native_currency: string;
    quote_token_price_usd: string;
    price_change_percentage: { m5: string; h1: string; h6: string; h24: string };
    transactions: {
        m5: { buys: number; sells: number; buyers: number; sellers: number };
        h1: { buys: number; sells: number; buyers: number; sellers: number };
        h24: { buys: number; sells: number; buyers: number; sellers: number };
    };
    volume_usd: { m5: string; h1: string; h6: string; h24: string };
    reserve_in_usd: string;
    fdv_usd: string;
    market_cap_usd: string | null;
    pool_created_at: string;
    trending_score: number;
}

export const DEMO_POOLS: DemoPool[] = [
    {
        id: "eth_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
        network: "eth",
        address: "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
        name: "WETH / USDC 0.05%",
        dex: { id: "uniswap_v3", name: "Uniswap V3" },
        base_token: {
            address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            symbol: "WETH",
            name: "Wrapped Ether",
            image_url: "https://assets.coingecko.com/coins/images/2518/small/weth.png",
        },
        quote_token: {
            address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            symbol: "USDC",
            name: "USD Coin",
            image_url: "https://assets.coingecko.com/coins/images/6319/small/usdc.png",
        },
        base_token_price_usd: "3241.58",
        base_token_price_native_currency: "1.0",
        quote_token_price_usd: "0.9998",
        price_change_percentage: { m5: "0.12", h1: "-1.44", h6: "2.31", h24: "3.87" },
        transactions: {
            m5: { buys: 12, sells: 8, buyers: 10, sellers: 7 },
            h1: { buys: 142, sells: 98, buyers: 121, sellers: 84 },
            h24: { buys: 3842, sells: 2891, buyers: 2104, sellers: 1849 },
        },
        volume_usd: { m5: "412840.22", h1: "7284019.44", h6: "41820491.83", h24: "536545444.90" },
        reserve_in_usd: "163988541.38",
        fdv_usd: "389291204812.00",
        market_cap_usd: "389291204812.00",
        pool_created_at: "2021-05-04T10:00:00Z",
        trending_score: 98.4,
    },
    {
        id: "eth_0xcbcdf9626bc03e24f779434178a73a0b4bad62ed",
        network: "eth",
        address: "0xcbcdf9626bc03e24f779434178a73a0b4bad62ed",
        name: "WBTC / WETH 0.3%",
        dex: { id: "uniswap_v3", name: "Uniswap V3" },
        base_token: {
            address: "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599",
            symbol: "WBTC",
            name: "Wrapped Bitcoin",
            image_url: "https://assets.coingecko.com/coins/images/7598/small/wrapped_bitcoin_wbtc.png",
        },
        quote_token: {
            address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            symbol: "WETH",
            name: "Wrapped Ether",
            image_url: "https://assets.coingecko.com/coins/images/2518/small/weth.png",
        },
        base_token_price_usd: "97541.22",
        base_token_price_native_currency: "30.09",
        quote_token_price_usd: "3241.58",
        price_change_percentage: { m5: "-0.05", h1: "0.78", h6: "-0.44", h24: "1.12" },
        transactions: {
            m5: { buys: 3, sells: 4, buyers: 3, sellers: 4 },
            h1: { buys: 48, sells: 39, buyers: 41, sellers: 33 },
            h24: { buys: 1241, sells: 982, buyers: 874, sellers: 743 },
        },
        volume_usd: { m5: "84219.11", h1: "1842904.22", h6: "9842104.77", h24: "74182904.33" },
        reserve_in_usd: "48291742.55",
        fdv_usd: "1932841049102.00",
        market_cap_usd: "1932841049102.00",
        pool_created_at: "2021-05-05T14:22:00Z",
        trending_score: 91.2,
    },
    {
        id: "base_0x4c36388be6f416a29c8d8eee81c771ce6be14b18",
        network: "base",
        address: "0x4c36388be6f416a29c8d8eee81c771ce6be14b18",
        name: "VIRTUAL / WETH",
        dex: { id: "aerodrome", name: "Aerodrome" },
        base_token: {
            address: "0x0b3e328455c4059eeb9e3f84b5543f74e24e7e1b",
            symbol: "VIRTUAL",
            name: "Virtuals Protocol",
            image_url: null,
        },
        quote_token: {
            address: "0x4200000000000000000000000000000000000006",
            symbol: "WETH",
            name: "Wrapped Ether",
            image_url: "https://assets.coingecko.com/coins/images/2518/small/weth.png",
        },
        base_token_price_usd: "1.8841",
        base_token_price_native_currency: "0.000581",
        quote_token_price_usd: "3241.58",
        price_change_percentage: { m5: "2.41", h1: "8.92", h6: "14.22", h24: "31.44" },
        transactions: {
            m5: { buys: 48, sells: 19, buyers: 42, sellers: 14 },
            h1: { buys: 492, sells: 231, buyers: 384, sellers: 194 },
            h24: { buys: 8421, sells: 4892, buyers: 4912, sellers: 3241 },
        },
        volume_usd: { m5: "89421.33", h1: "1284019.44", h6: "8120491.83", h24: "42841044.90" },
        reserve_in_usd: "9182041.22",
        fdv_usd: "1884100000.00",
        market_cap_usd: null,
        pool_created_at: "2024-08-19T11:44:00Z",
        trending_score: 87.6,
    },
    {
        id: "solana_8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj",
        network: "solana",
        address: "8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj",
        name: "SOL / USDC",
        dex: { id: "raydium", name: "Raydium" },
        base_token: {
            address: "So11111111111111111111111111111111111111112",
            symbol: "SOL",
            name: "Solana",
            image_url: "https://assets.coingecko.com/coins/images/4128/small/solana.png",
        },
        quote_token: {
            address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            symbol: "USDC",
            name: "USD Coin",
            image_url: "https://assets.coingecko.com/coins/images/6319/small/usdc.png",
        },
        base_token_price_usd: "174.32",
        base_token_price_native_currency: "1.0",
        quote_token_price_usd: "0.9999",
        price_change_percentage: { m5: "0.33", h1: "1.22", h6: "-0.88", h24: "4.11" },
        transactions: {
            m5: { buys: 82, sells: 54, buyers: 71, sellers: 48 },
            h1: { buys: 912, sells: 714, buyers: 841, sellers: 622 },
            h24: { buys: 18241, sells: 14892, buyers: 12104, sellers: 10849 },
        },
        volume_usd: { m5: "234119.22", h1: "4912049.33", h6: "29184022.77", h24: "218491044.11" },
        reserve_in_usd: "87412041.99",
        fdv_usd: "68412094102.00",
        market_cap_usd: "68412094102.00",
        pool_created_at: "2022-10-20T00:00:00Z",
        trending_score: 95.1,
    },
    {
        id: "base_0x6921b130d297cc43754afba22e5eac0fbf8db75b",
        network: "base",
        address: "0x6921b130d297cc43754afba22e5eac0fbf8db75b",
        name: "DEGEN / WETH",
        dex: { id: "uniswap_v3", name: "Uniswap V3" },
        base_token: {
            address: "0x4ed4e862860bed51a9570b96d89af5e1b0efefed",
            symbol: "DEGEN",
            name: "Degen",
            image_url: null,
        },
        quote_token: {
            address: "0x4200000000000000000000000000000000000006",
            symbol: "WETH",
            name: "Wrapped Ether",
            image_url: "https://assets.coingecko.com/coins/images/2518/small/weth.png",
        },
        base_token_price_usd: "0.008241",
        base_token_price_native_currency: "0.00000254",
        quote_token_price_usd: "3241.58",
        price_change_percentage: { m5: "-0.82", h1: "-3.11", h6: "-5.44", h24: "12.88" },
        transactions: {
            m5: { buys: 22, sells: 31, buyers: 18, sellers: 27 },
            h1: { buys: 241, sells: 312, buyers: 194, sellers: 272 },
            h24: { buys: 4821, sells: 5912, buyers: 2914, sellers: 3841 },
        },
        volume_usd: { m5: "12841.00", h1: "284019.44", h6: "1820491.83", h24: "14841044.90" },
        reserve_in_usd: "3182041.44",
        fdv_usd: "824100000.00",
        market_cap_usd: null,
        pool_created_at: "2024-01-15T08:00:00Z",
        trending_score: 72.3,
    },
];

/** Compact pool summary for list endpoints */
export function poolSummary(pool: DemoPool) {
    return {
        id: pool.id,
        network: pool.network,
        address: pool.address,
        name: pool.name,
        dex: pool.dex,
        base_token: pool.base_token,
        quote_token: pool.quote_token,
        base_token_price_usd: pool.base_token_price_usd,
        price_change_percentage: pool.price_change_percentage,
        volume_usd: { h1: pool.volume_usd.h1, h24: pool.volume_usd.h24 },
        reserve_in_usd: pool.reserve_in_usd,
        trending_score: pool.trending_score,
    };
}
