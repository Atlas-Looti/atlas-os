//! CoinGecko service — business logic with Redis caching.
#![allow(clippy::too_many_arguments)]

use std::time::Duration;

use crate::clients::cache::Cache;
use crate::clients::coingecko::*;

/// CoinGecko service with caching layer.
pub struct CoinGeckoService;

/// Cache TTLs specific to CoinGecko data.
pub struct CgCacheTtl;

impl CgCacheTtl {
    /// Coin prices — 20s (matches CG update frequency).
    pub const PRICE: Duration = Duration::from_secs(20);
    /// Market data (coins/markets) — 30s.
    pub const MARKETS: Duration = Duration::from_secs(30);
    /// Coin detail — 60s.
    pub const COIN_DETAIL: Duration = Duration::from_secs(60);
    /// Chart data — 60s.
    pub const CHART: Duration = Duration::from_secs(60);
    /// OHLC data — 60s.
    pub const OHLC: Duration = Duration::from_secs(60);
    /// Trending — 5min.
    pub const TRENDING: Duration = Duration::from_secs(300);
    /// Search results — 60s.
    pub const SEARCH: Duration = Duration::from_secs(60);
    /// Global data — 60s.
    pub const GLOBAL: Duration = Duration::from_secs(60);
    /// Categories — 5min.
    pub const CATEGORIES: Duration = Duration::from_secs(300);
    /// Exchanges — 5min.
    pub const EXCHANGES: Duration = Duration::from_secs(300);
    /// Currencies list — 1h.
    pub const CURRENCIES: Duration = Duration::from_secs(3600);
    /// Coins list (id map) — 1h.
    pub const COINS_LIST: Duration = Duration::from_secs(3600);
    /// On-chain pools — 30s.
    pub const ONCHAIN_POOLS: Duration = Duration::from_secs(30);
    /// On-chain networks — 1h.
    pub const ONCHAIN_NETWORKS: Duration = Duration::from_secs(3600);
}

impl CoinGeckoService {
    // ── Simple Price ────────────────────────────────────────────

    pub async fn simple_price(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        ids: &str,
        vs_currencies: &str,
        include_market_cap: bool,
        include_24hr_vol: bool,
        include_24hr_change: bool,
        include_last_updated_at: bool,
        precision: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let cache_key = Cache::key("cg:price", &[ids, vs_currencies]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<serde_json::Value>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client
            .simple_price(
                ids,
                vs_currencies,
                include_market_cap,
                include_24hr_vol,
                include_24hr_change,
                include_last_updated_at,
                precision,
            )
            .await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::PRICE).await;
        }

        Ok(result)
    }

    // ── Token Price ─────────────────────────────────────────────

    pub async fn simple_token_price(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        platform: &str,
        contract_addresses: &str,
        vs_currencies: &str,
        include_market_cap: bool,
        include_24hr_vol: bool,
        include_24hr_change: bool,
        include_last_updated_at: bool,
    ) -> anyhow::Result<serde_json::Value> {
        let cache_key = Cache::key("cg:token_price", &[platform, contract_addresses, vs_currencies]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<serde_json::Value>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client
            .simple_token_price(
                platform,
                contract_addresses,
                vs_currencies,
                include_market_cap,
                include_24hr_vol,
                include_24hr_change,
                include_last_updated_at,
            )
            .await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::PRICE).await;
        }

        Ok(result)
    }

    // ── Coins Markets ───────────────────────────────────────────

    pub async fn coins_markets(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        vs_currency: &str,
        ids: Option<&str>,
        category: Option<&str>,
        order: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
        sparkline: bool,
        price_change_percentage: Option<&str>,
    ) -> anyhow::Result<Vec<CoinMarket>> {
        let ids_str = ids.unwrap_or("_");
        let cat_str = category.unwrap_or("_");
        let pg = page.unwrap_or(1).to_string();
        let cache_key = Cache::key("cg:markets", &[vs_currency, ids_str, cat_str, &pg]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<Vec<CoinMarket>>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client
            .coins_markets(
                vs_currency,
                ids,
                category,
                order,
                per_page,
                page,
                sparkline,
                price_change_percentage,
            )
            .await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::MARKETS).await;
        }

        Ok(result)
    }

    // ── Coin Detail ─────────────────────────────────────────────

    pub async fn coin_by_id(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        id: &str,
    ) -> anyhow::Result<CoinDetail> {
        let cache_key = Cache::key("cg:coin", &[id]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<CoinDetail>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client
            .coin_by_id(id, false, false, true, false, false, false)
            .await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::COIN_DETAIL).await;
        }

        Ok(result)
    }

    // ── Chart Data ──────────────────────────────────────────────

    pub async fn coin_market_chart(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        id: &str,
        vs_currency: &str,
        days: &str,
        interval: Option<&str>,
        precision: Option<&str>,
    ) -> anyhow::Result<MarketChartData> {
        let int_str = interval.unwrap_or("_");
        let cache_key = Cache::key("cg:chart", &[id, vs_currency, days, int_str]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<MarketChartData>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client
            .coin_market_chart(id, vs_currency, days, interval, precision)
            .await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::CHART).await;
        }

        Ok(result)
    }

    // ── OHLC Data ───────────────────────────────────────────────

    pub async fn coin_ohlc(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        id: &str,
        vs_currency: &str,
        days: &str,
        precision: Option<&str>,
    ) -> anyhow::Result<OhlcData> {
        let cache_key = Cache::key("cg:ohlc", &[id, vs_currency, days]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<OhlcData>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.coin_ohlc(id, vs_currency, days, precision).await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::OHLC).await;
        }

        Ok(result)
    }

    // ── Trending ────────────────────────────────────────────────

    pub async fn trending(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
    ) -> anyhow::Result<TrendingResponse> {
        let cache_key = Cache::key("cg:trending", &[]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<TrendingResponse>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.trending().await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::TRENDING).await;
        }

        Ok(result)
    }

    // ── Search ──────────────────────────────────────────────────

    pub async fn search(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        query: &str,
    ) -> anyhow::Result<SearchResponse> {
        let cache_key = Cache::key("cg:search", &[query]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<SearchResponse>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.search(query).await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::SEARCH).await;
        }

        Ok(result)
    }

    // ── Global ──────────────────────────────────────────────────

    pub async fn global(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
    ) -> anyhow::Result<GlobalDataWrapper> {
        let cache_key = Cache::key("cg:global", &[]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<GlobalDataWrapper>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.global().await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::GLOBAL).await;
        }

        Ok(result)
    }

    pub async fn global_defi(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
    ) -> anyhow::Result<GlobalDefiWrapper> {
        let cache_key = Cache::key("cg:global_defi", &[]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<GlobalDefiWrapper>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.global_defi().await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::GLOBAL).await;
        }

        Ok(result)
    }

    // ── Categories ──────────────────────────────────────────────

    pub async fn categories(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        order: Option<&str>,
    ) -> anyhow::Result<Vec<CoinCategory>> {
        let ord = order.unwrap_or("market_cap_desc");
        let cache_key = Cache::key("cg:categories", &[ord]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<Vec<CoinCategory>>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.categories(order).await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::CATEGORIES).await;
        }

        Ok(result)
    }

    // ── Exchanges ───────────────────────────────────────────────

    pub async fn exchanges(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> anyhow::Result<Vec<Exchange>> {
        let pg = page.unwrap_or(1).to_string();
        let cache_key = Cache::key("cg:exchanges", &[&pg]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<Vec<Exchange>>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.exchanges(per_page, page).await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::EXCHANGES).await;
        }

        Ok(result)
    }

    // ── Supported Currencies ────────────────────────────────────

    pub async fn supported_vs_currencies(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
    ) -> anyhow::Result<Vec<String>> {
        let cache_key = Cache::key("cg:currencies", &[]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<Vec<String>>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.supported_vs_currencies().await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::CURRENCIES).await;
        }

        Ok(result)
    }

    // ── Coins List ──────────────────────────────────────────────

    pub async fn coins_list(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        include_platform: bool,
    ) -> anyhow::Result<Vec<CoinListItem>> {
        let ip = include_platform.to_string();
        let cache_key = Cache::key("cg:coins_list", &[&ip]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<Vec<CoinListItem>>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.coins_list(include_platform).await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::COINS_LIST).await;
        }

        Ok(result)
    }

    // ── On-chain ────────────────────────────────────────────────

    pub async fn onchain_trending_pools(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        network: Option<&str>,
    ) -> anyhow::Result<OnchainPoolsResponse> {
        let net = network.unwrap_or("all");
        let cache_key = Cache::key("cg:onchain_trending", &[net]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<OnchainPoolsResponse>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.onchain_trending_pools(network, None, None).await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::ONCHAIN_POOLS).await;
        }

        Ok(result)
    }

    pub async fn onchain_new_pools(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        network: Option<&str>,
    ) -> anyhow::Result<OnchainPoolsResponse> {
        let net = network.unwrap_or("all");
        let cache_key = Cache::key("cg:onchain_new", &[net]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<OnchainPoolsResponse>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.onchain_new_pools(network, None, None).await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::ONCHAIN_POOLS).await;
        }

        Ok(result)
    }

    pub async fn onchain_networks(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
    ) -> anyhow::Result<OnchainNetworksResponse> {
        let cache_key = Cache::key("cg:onchain_networks", &[]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<OnchainNetworksResponse>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.onchain_networks().await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::ONCHAIN_NETWORKS).await;
        }

        Ok(result)
    }

    pub async fn onchain_top_pools(
        client: &CoinGeckoClient,
        cache: Option<&Cache>,
        network: &str,
        page: Option<u32>,
        sort: Option<&str>,
    ) -> anyhow::Result<OnchainPoolsResponse> {
        let pg = page.unwrap_or(1).to_string();
        let srt = sort.unwrap_or("h24_tx_count_desc");
        let cache_key = Cache::key("cg:onchain_top", &[network, &pg, srt]);

        if let Some(c) = cache {
            if let Some(cached) = c.get::<OnchainPoolsResponse>(&cache_key).await {
                return Ok(cached);
            }
        }

        let result = client.onchain_top_pools(network, None, page, sort).await?;

        if let Some(c) = cache {
            let _ = c.set(&cache_key, &result, CgCacheTtl::ONCHAIN_POOLS).await;
        }

        Ok(result)
    }
}
