//! CoinGecko API client — Pro/Demo API with automatic retry on rate-limit.
//!
//! Supports both Pro API (x-cg-pro-api-key) and Demo API (x-cg-demo-api-key).
//! Rate-limit aware with exponential backoff on 429 responses.
#![allow(clippy::too_many_arguments)]

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// CoinGecko API tier — determines base URL and auth header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoinGeckoTier {
    /// Demo (free) API: api.coingecko.com/api/v3
    Demo,
    /// Pro (paid) API: pro-api.coingecko.com/api/v3
    Pro,
}

/// CoinGecko HTTP client with rate-limit handling.
#[derive(Clone)]
pub struct CoinGeckoClient {
    http: Client,
    api_key: String,
    tier: CoinGeckoTier,
}

// ── Response Types ──────────────────────────────────────────────────

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PingResponse {
    pub gecko_says: String,
}

/// Coin market data from /coins/markets
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CoinMarket {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub image: Option<String>,
    pub current_price: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<u32>,
    pub fully_diluted_valuation: Option<f64>,
    pub total_volume: Option<f64>,
    pub high_24h: Option<f64>,
    pub low_24h: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub market_cap_change_24h: Option<f64>,
    pub market_cap_change_percentage_24h: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub ath: Option<f64>,
    pub ath_change_percentage: Option<f64>,
    pub ath_date: Option<String>,
    pub atl: Option<f64>,
    pub atl_change_percentage: Option<f64>,
    pub atl_date: Option<String>,
    pub last_updated: Option<String>,
    pub sparkline_in_7d: Option<SparklineData>,
    pub price_change_percentage_1h_in_currency: Option<f64>,
    pub price_change_percentage_24h_in_currency: Option<f64>,
    pub price_change_percentage_7d_in_currency: Option<f64>,
    pub price_change_percentage_14d_in_currency: Option<f64>,
    pub price_change_percentage_30d_in_currency: Option<f64>,
    pub price_change_percentage_200d_in_currency: Option<f64>,
    pub price_change_percentage_1y_in_currency: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SparklineData {
    pub price: Vec<f64>,
}

/// Detailed coin data from /coins/{id}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CoinDetail {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub web_slug: Option<String>,
    pub categories: Option<Vec<String>>,
    pub description: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
    pub image: Option<serde_json::Value>,
    pub genesis_date: Option<String>,
    pub market_cap_rank: Option<u32>,
    pub market_data: Option<CoinMarketData>,
    pub community_data: Option<serde_json::Value>,
    pub developer_data: Option<serde_json::Value>,
    pub tickers: Option<Vec<serde_json::Value>>,
    pub last_updated: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CoinMarketData {
    pub current_price: Option<serde_json::Value>,
    pub market_cap: Option<serde_json::Value>,
    pub total_volume: Option<serde_json::Value>,
    pub high_24h: Option<serde_json::Value>,
    pub low_24h: Option<serde_json::Value>,
    pub price_change_24h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub price_change_percentage_7d: Option<f64>,
    pub price_change_percentage_14d: Option<f64>,
    pub price_change_percentage_30d: Option<f64>,
    pub price_change_percentage_60d: Option<f64>,
    pub price_change_percentage_200d: Option<f64>,
    pub price_change_percentage_1y: Option<f64>,
    pub market_cap_change_24h: Option<f64>,
    pub market_cap_change_percentage_24h: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub ath: Option<serde_json::Value>,
    pub atl: Option<serde_json::Value>,
    pub last_updated: Option<String>,
}

/// Historical chart data: [[timestamp, value], ...]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MarketChartData {
    pub prices: Vec<[f64; 2]>,
    pub market_caps: Vec<[f64; 2]>,
    pub total_volumes: Vec<[f64; 2]>,
}

/// OHLC data: [[timestamp, open, high, low, close], ...]
pub type OhlcData = Vec<[f64; 5]>;

/// Trending search response
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TrendingResponse {
    pub coins: Option<Vec<TrendingCoinItem>>,
    pub nfts: Option<Vec<serde_json::Value>>,
    pub categories: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TrendingCoinItem {
    pub item: TrendingCoin,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TrendingCoin {
    pub id: String,
    pub coin_id: Option<u64>,
    pub name: String,
    pub symbol: String,
    pub market_cap_rank: Option<u32>,
    pub thumb: Option<String>,
    pub small: Option<String>,
    pub large: Option<String>,
    pub slug: Option<String>,
    pub price_btc: Option<f64>,
    pub score: Option<u32>,
    pub data: Option<serde_json::Value>,
}

/// Search result
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchResponse {
    pub coins: Option<Vec<SearchCoin>>,
    pub exchanges: Option<Vec<SearchExchange>>,
    pub categories: Option<Vec<SearchCategory>>,
    pub nfts: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchCoin {
    pub id: String,
    pub name: String,
    pub api_symbol: Option<String>,
    pub symbol: String,
    pub market_cap_rank: Option<u32>,
    pub thumb: Option<String>,
    pub large: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchExchange {
    pub id: String,
    pub name: String,
    pub market_type: Option<String>,
    pub thumb: Option<String>,
    pub large: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchCategory {
    pub id: Option<u64>,
    pub name: String,
}

/// Global market data
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GlobalDataWrapper {
    pub data: GlobalData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GlobalData {
    pub active_cryptocurrencies: Option<u64>,
    pub upcoming_icos: Option<u64>,
    pub ongoing_icos: Option<u64>,
    pub ended_icos: Option<u64>,
    pub markets: Option<u64>,
    pub total_market_cap: Option<serde_json::Value>,
    pub total_volume: Option<serde_json::Value>,
    pub market_cap_percentage: Option<serde_json::Value>,
    pub market_cap_change_percentage_24h_usd: Option<f64>,
    pub updated_at: Option<u64>,
}

/// Global DeFi data
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GlobalDefiWrapper {
    pub data: GlobalDefiData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GlobalDefiData {
    pub defi_market_cap: Option<String>,
    pub eth_market_cap: Option<String>,
    pub defi_to_eth_ratio: Option<String>,
    pub trading_volume_24h: Option<String>,
    pub defi_dominance: Option<String>,
    pub top_coin_name: Option<String>,
    pub top_coin_defi_dominance: Option<f64>,
}

/// Coin category with market data
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CoinCategory {
    pub id: String,
    pub name: String,
    pub market_cap: Option<f64>,
    pub market_cap_change_24h: Option<f64>,
    pub content: Option<String>,
    pub top_3_coins: Option<Vec<String>>,
    pub volume_24h: Option<f64>,
    pub updated_at: Option<String>,
}

/// Exchange data
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Exchange {
    pub id: String,
    pub name: String,
    pub year_established: Option<u32>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub image: Option<String>,
    pub has_trading_incentive: Option<bool>,
    pub trust_score: Option<u32>,
    pub trust_score_rank: Option<u32>,
    pub trade_volume_24h_btc: Option<f64>,
    pub trade_volume_24h_btc_normalized: Option<f64>,
}

/// API usage / key info
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApiUsage {
    pub plan: Option<String>,
    pub rate_limit_request_per_minute: Option<u64>,
    pub monthly_call_credit: Option<u64>,
    pub current_total_monthly_calls: Option<u64>,
    pub current_remaining_monthly_calls: Option<u64>,
}

/// Coins list item (id map)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CoinListItem {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub platforms: Option<serde_json::Value>,
}

/// Top gainers/losers item
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TopMoverItem {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub image: Option<String>,
    pub market_cap_rank: Option<u32>,
    pub usd: Option<f64>,
    pub usd_24h_vol: Option<f64>,
    pub usd_24h_change: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TopMovers {
    pub top_gainers: Vec<TopMoverItem>,
    pub top_losers: Vec<TopMoverItem>,
}

// ── On-chain / GeckoTerminal Types ──────────────────────────────────

/// Trending pools
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OnchainPoolsResponse {
    pub data: Vec<OnchainPoolData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OnchainPoolData {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: serde_json::Value,
    pub relationships: Option<serde_json::Value>,
}

/// Supported networks
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OnchainNetworksResponse {
    pub data: Vec<OnchainNetworkData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OnchainNetworkData {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: OnchainNetworkAttributes,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OnchainNetworkAttributes {
    pub name: String,
    pub coingecko_asset_platform_id: Option<String>,
}

// ── Client Implementation ───────────────────────────────────────────

impl CoinGeckoClient {
    /// Create a new CoinGecko client.
    pub fn new(api_key: &str, tier: CoinGeckoTier) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build CoinGecko HTTP client");

        Self {
            http,
            api_key: api_key.to_string(),
            tier,
        }
    }

    /// Base URL for CoinGecko v3 API.
    fn base_url(&self) -> &str {
        match self.tier {
            CoinGeckoTier::Demo => "https://api.coingecko.com/api/v3",
            CoinGeckoTier::Pro => "https://pro-api.coingecko.com/api/v3",
        }
    }

    /// Base URL for on-chain (GeckoTerminal) API.
    fn onchain_base_url(&self) -> &str {
        match self.tier {
            CoinGeckoTier::Demo => "https://api.coingecko.com/api/v3/onchain",
            CoinGeckoTier::Pro => "https://pro-api.coingecko.com/api/v3/onchain",
        }
    }

    /// Auth header name.
    fn auth_header(&self) -> &str {
        match self.tier {
            CoinGeckoTier::Demo => "x-cg-demo-api-key",
            CoinGeckoTier::Pro => "x-cg-pro-api-key",
        }
    }

    /// Execute a GET request with retry on 429.
    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        query: &[(&str, &str)],
    ) -> anyhow::Result<T> {
        let mut retries = 0u32;
        let max_retries = 3;

        loop {
            let resp = self
                .http
                .get(url)
                .header(self.auth_header(), &self.api_key)
                .query(query)
                .send()
                .await?;

            if resp.status() == 429 {
                retries += 1;
                if retries > max_retries {
                    anyhow::bail!("CoinGecko rate limited after {max_retries} retries");
                }
                let wait = Duration::from_millis(1000 * 2u64.pow(retries - 1));
                warn!(
                    "CoinGecko 429 — retrying in {:?} (attempt {retries}/{max_retries})",
                    wait
                );
                tokio::time::sleep(wait).await;
                continue;
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("CoinGecko API error {status}: {body}");
            }

            let result: T = resp.json().await?;
            return Ok(result);
        }
    }

    // ── Ping ────────────────────────────────────────────────────

    /// Check API server status.
    pub async fn ping(&self) -> anyhow::Result<PingResponse> {
        let url = format!("{}/ping", self.base_url());
        self.get(&url, &[]).await
    }

    // ── API Usage (Pro only) ────────────────────────────────────

    /// Monitor API usage (rate limits, credits).
    pub async fn api_usage(&self) -> anyhow::Result<ApiUsage> {
        let url = format!("{}/key", self.base_url());
        self.get(&url, &[]).await
    }

    // ── Simple ──────────────────────────────────────────────────

    /// Get coin prices by IDs.
    pub async fn simple_price(
        &self,
        ids: &str,
        vs_currencies: &str,
        include_market_cap: bool,
        include_24hr_vol: bool,
        include_24hr_change: bool,
        include_last_updated_at: bool,
        precision: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/simple/price", self.base_url());
        let mut query = vec![
            ("ids", ids),
            ("vs_currencies", vs_currencies),
        ];

        let mc = include_market_cap.to_string();
        let vol = include_24hr_vol.to_string();
        let chg = include_24hr_change.to_string();
        let upd = include_last_updated_at.to_string();

        if include_market_cap {
            query.push(("include_market_cap", &mc));
        }
        if include_24hr_vol {
            query.push(("include_24hr_vol", &vol));
        }
        if include_24hr_change {
            query.push(("include_24hr_change", &chg));
        }
        if include_last_updated_at {
            query.push(("include_last_updated_at", &upd));
        }
        if let Some(p) = precision {
            query.push(("precision", p));
        }

        self.get(&url, &query).await
    }

    /// Get token prices by contract addresses.
    pub async fn simple_token_price(
        &self,
        platform: &str,
        contract_addresses: &str,
        vs_currencies: &str,
        include_market_cap: bool,
        include_24hr_vol: bool,
        include_24hr_change: bool,
        include_last_updated_at: bool,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/simple/token_price/{}", self.base_url(), platform);
        let mut query = vec![
            ("contract_addresses", contract_addresses),
            ("vs_currencies", vs_currencies),
        ];

        let mc = include_market_cap.to_string();
        let vol = include_24hr_vol.to_string();
        let chg = include_24hr_change.to_string();
        let upd = include_last_updated_at.to_string();

        if include_market_cap {
            query.push(("include_market_cap", &mc));
        }
        if include_24hr_vol {
            query.push(("include_24hr_vol", &vol));
        }
        if include_24hr_change {
            query.push(("include_24hr_change", &chg));
        }
        if include_last_updated_at {
            query.push(("include_last_updated_at", &upd));
        }

        self.get(&url, &query).await
    }

    /// Get supported vs_currencies.
    pub async fn supported_vs_currencies(&self) -> anyhow::Result<Vec<String>> {
        let url = format!("{}/simple/supported_vs_currencies", self.base_url());
        self.get(&url, &[]).await
    }

    // ── Coins ───────────────────────────────────────────────────

    /// List coins with market data.
    pub async fn coins_markets(
        &self,
        vs_currency: &str,
        ids: Option<&str>,
        category: Option<&str>,
        order: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
        sparkline: bool,
        price_change_percentage: Option<&str>,
    ) -> anyhow::Result<Vec<CoinMarket>> {
        let url = format!("{}/coins/markets", self.base_url());
        let mut query = vec![("vs_currency", vs_currency)];

        if let Some(v) = ids {
            query.push(("ids", v));
        }
        if let Some(v) = category {
            query.push(("category", v));
        }
        if let Some(v) = order {
            query.push(("order", v));
        }

        let pp = per_page.unwrap_or(100).to_string();
        let pg = page.unwrap_or(1).to_string();
        let sl = sparkline.to_string();

        query.push(("per_page", &pp));
        query.push(("page", &pg));
        query.push(("sparkline", &sl));

        if let Some(v) = price_change_percentage {
            query.push(("price_change_percentage", v));
        }

        self.get(&url, &query).await
    }

    /// Get coin data by ID.
    pub async fn coin_by_id(
        &self,
        id: &str,
        localization: bool,
        tickers: bool,
        market_data: bool,
        community_data: bool,
        developer_data: bool,
        sparkline: bool,
    ) -> anyhow::Result<CoinDetail> {
        let url = format!("{}/coins/{}", self.base_url(), id);
        let loc = localization.to_string();
        let tick = tickers.to_string();
        let md = market_data.to_string();
        let cd = community_data.to_string();
        let dd = developer_data.to_string();
        let sl = sparkline.to_string();

        let query = vec![
            ("localization", loc.as_str()),
            ("tickers", tick.as_str()),
            ("market_data", md.as_str()),
            ("community_data", cd.as_str()),
            ("developer_data", dd.as_str()),
            ("sparkline", sl.as_str()),
        ];

        self.get(&url, &query).await
    }

    /// Get coins list (id map).
    pub async fn coins_list(&self, include_platform: bool) -> anyhow::Result<Vec<CoinListItem>> {
        let url = format!("{}/coins/list", self.base_url());
        let ip = include_platform.to_string();
        let query = vec![("include_platform", ip.as_str())];
        self.get(&url, &query).await
    }

    /// Get historical chart data for a coin.
    pub async fn coin_market_chart(
        &self,
        id: &str,
        vs_currency: &str,
        days: &str,
        interval: Option<&str>,
        precision: Option<&str>,
    ) -> anyhow::Result<MarketChartData> {
        let url = format!("{}/coins/{}/market_chart", self.base_url(), id);
        let mut query = vec![
            ("vs_currency", vs_currency),
            ("days", days),
        ];
        if let Some(v) = interval {
            query.push(("interval", v));
        }
        if let Some(v) = precision {
            query.push(("precision", v));
        }
        self.get(&url, &query).await
    }

    /// Get OHLC chart data for a coin.
    pub async fn coin_ohlc(
        &self,
        id: &str,
        vs_currency: &str,
        days: &str,
        precision: Option<&str>,
    ) -> anyhow::Result<OhlcData> {
        let url = format!("{}/coins/{}/ohlc", self.base_url(), id);
        let mut query = vec![
            ("vs_currency", vs_currency),
            ("days", days),
        ];
        if let Some(v) = precision {
            query.push(("precision", v));
        }
        self.get(&url, &query).await
    }

    /// Get historical chart data within a time range.
    pub async fn coin_market_chart_range(
        &self,
        id: &str,
        vs_currency: &str,
        from: &str,
        to: &str,
        precision: Option<&str>,
    ) -> anyhow::Result<MarketChartData> {
        let url = format!("{}/coins/{}/market_chart/range", self.base_url(), id);
        let mut query = vec![
            ("vs_currency", vs_currency),
            ("from", from),
            ("to", to),
        ];
        if let Some(v) = precision {
            query.push(("precision", v));
        }
        self.get(&url, &query).await
    }

    /// Get coin tickers by ID.
    pub async fn coin_tickers(
        &self,
        id: &str,
        exchange_ids: Option<&str>,
        include_exchange_logo: bool,
        page: Option<u32>,
        order: Option<&str>,
        depth: bool,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/coins/{}/tickers", self.base_url(), id);
        let mut query = Vec::new();

        if let Some(v) = exchange_ids {
            query.push(("exchange_ids", v.to_string()));
        }
        let logo = include_exchange_logo.to_string();
        query.push(("include_exchange_logo", logo));

        if let Some(v) = page {
            query.push(("page", v.to_string()));
        }
        if let Some(v) = order {
            query.push(("order", v.to_string()));
        }
        let dp = depth.to_string();
        query.push(("depth", dp));

        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.get(&url, &q).await
    }

    // ── Trending ────────────────────────────────────────────────

    /// Get trending search coins, NFTs, and categories.
    pub async fn trending(&self) -> anyhow::Result<TrendingResponse> {
        let url = format!("{}/search/trending", self.base_url());
        self.get(&url, &[]).await
    }

    // ── Search ──────────────────────────────────────────────────

    /// Search for coins, categories, markets.
    pub async fn search(&self, query: &str) -> anyhow::Result<SearchResponse> {
        let url = format!("{}/search", self.base_url());
        self.get(&url, &[("query", query)]).await
    }

    // ── Global ──────────────────────────────────────────────────

    /// Get global crypto market data.
    pub async fn global(&self) -> anyhow::Result<GlobalDataWrapper> {
        let url = format!("{}/global", self.base_url());
        self.get(&url, &[]).await
    }

    /// Get global DeFi market data.
    pub async fn global_defi(&self) -> anyhow::Result<GlobalDefiWrapper> {
        let url = format!("{}/global/decentralized_finance_defi", self.base_url());
        self.get(&url, &[]).await
    }

    // ── Categories ──────────────────────────────────────────────

    /// Get coin categories with market data.
    pub async fn categories(
        &self,
        order: Option<&str>,
    ) -> anyhow::Result<Vec<CoinCategory>> {
        let url = format!("{}/coins/categories", self.base_url());
        let mut query = Vec::new();
        if let Some(v) = order {
            query.push(("order", v));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }

    // ── Exchanges ───────────────────────────────────────────────

    /// Get exchanges list with data.
    pub async fn exchanges(
        &self,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> anyhow::Result<Vec<Exchange>> {
        let url = format!("{}/exchanges", self.base_url());
        let pp = per_page.unwrap_or(100).to_string();
        let pg = page.unwrap_or(1).to_string();
        let query = vec![("per_page", pp.as_str()), ("page", pg.as_str())];
        self.get(&url, &query).await
    }

    /// Get exchange data by ID.
    pub async fn exchange_by_id(&self, id: &str) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/exchanges/{}", self.base_url(), id);
        self.get(&url, &[]).await
    }

    // ── Top Gainers/Losers (Pro) ────────────────────────────────

    /// Get top 30 coins with largest price gain/loss.
    pub async fn top_gainers_losers(
        &self,
        vs_currency: &str,
        duration: Option<&str>,
    ) -> anyhow::Result<TopMovers> {
        let url = format!("{}/coins/top_gainers_losers", self.base_url());
        let mut query = vec![("vs_currency", vs_currency)];
        if let Some(v) = duration {
            query.push(("duration", v));
        }
        self.get(&url, &query).await
    }

    // ── Exchange Rates ──────────────────────────────────────────

    /// Get BTC exchange rates with other currencies.
    pub async fn exchange_rates(&self) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/exchange_rates", self.base_url());
        self.get(&url, &[]).await
    }

    // ── On-chain / GeckoTerminal ────────────────────────────────

    /// Get supported networks (GeckoTerminal).
    pub async fn onchain_networks(&self) -> anyhow::Result<OnchainNetworksResponse> {
        let url = format!("{}/networks", self.onchain_base_url());
        self.get(&url, &[]).await
    }

    /// Get trending pools across all networks.
    pub async fn onchain_trending_pools(
        &self,
        network: Option<&str>,
        include: Option<&str>,
        page: Option<u32>,
    ) -> anyhow::Result<OnchainPoolsResponse> {
        let url = match network {
            Some(net) => format!("{}/networks/{}/trending_pools", self.onchain_base_url(), net),
            None => format!("{}/networks/trending_pools", self.onchain_base_url()),
        };
        let mut query = Vec::new();
        let pg;
        if let Some(v) = include {
            query.push(("include", v));
        }
        if let Some(v) = page {
            pg = v.to_string();
            query.push(("page", pg.as_str()));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }

    /// Get new pools across all networks or by network.
    pub async fn onchain_new_pools(
        &self,
        network: Option<&str>,
        include: Option<&str>,
        page: Option<u32>,
    ) -> anyhow::Result<OnchainPoolsResponse> {
        let url = match network {
            Some(net) => format!("{}/networks/{}/new_pools", self.onchain_base_url(), net),
            None => format!("{}/networks/new_pools", self.onchain_base_url()),
        };
        let mut query = Vec::new();
        let pg;
        if let Some(v) = include {
            query.push(("include", v));
        }
        if let Some(v) = page {
            pg = v.to_string();
            query.push(("page", pg.as_str()));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }

    /// Get top pools by network.
    pub async fn onchain_top_pools(
        &self,
        network: &str,
        include: Option<&str>,
        page: Option<u32>,
        sort: Option<&str>,
    ) -> anyhow::Result<OnchainPoolsResponse> {
        let url = format!("{}/networks/{}/pools", self.onchain_base_url(), network);
        let mut query = Vec::new();
        let pg;
        if let Some(v) = include {
            query.push(("include", v));
        }
        if let Some(v) = page {
            pg = v.to_string();
            query.push(("page", pg.as_str()));
        }
        if let Some(v) = sort {
            query.push(("sort", v));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }

    /// Get pool data by address.
    pub async fn onchain_pool(
        &self,
        network: &str,
        pool_address: &str,
        include: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!(
            "{}/networks/{}/pools/{}",
            self.onchain_base_url(),
            network,
            pool_address
        );
        let mut query = Vec::new();
        if let Some(v) = include {
            query.push(("include", v));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }

    /// Get top pools by token address.
    pub async fn onchain_token_pools(
        &self,
        network: &str,
        token_address: &str,
        include: Option<&str>,
        page: Option<u32>,
        sort: Option<&str>,
    ) -> anyhow::Result<OnchainPoolsResponse> {
        let url = format!(
            "{}/networks/{}/tokens/{}/pools",
            self.onchain_base_url(),
            network,
            token_address
        );
        let mut query = Vec::new();
        let pg;
        if let Some(v) = include {
            query.push(("include", v));
        }
        if let Some(v) = page {
            pg = v.to_string();
            query.push(("page", pg.as_str()));
        }
        if let Some(v) = sort {
            query.push(("sort", v));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }

    /// Get token info by address.
    pub async fn onchain_token_info(
        &self,
        network: &str,
        token_address: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!(
            "{}/networks/{}/tokens/{}/info",
            self.onchain_base_url(),
            network,
            token_address
        );
        self.get(&url, &[]).await
    }

    /// Search pools.
    pub async fn onchain_search_pools(
        &self,
        query_str: &str,
        network: Option<&str>,
        include: Option<&str>,
        page: Option<u32>,
    ) -> anyhow::Result<OnchainPoolsResponse> {
        let url = format!("{}/search/pools", self.onchain_base_url());
        let mut query = vec![("query", query_str)];
        if let Some(v) = network {
            query.push(("network", v));
        }
        if let Some(v) = include {
            query.push(("include", v));
        }
        let pg;
        if let Some(v) = page {
            pg = v.to_string();
            query.push(("page", pg.as_str()));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }

    /// Get pool OHLCV chart.
    pub async fn onchain_pool_ohlcv(
        &self,
        network: &str,
        pool_address: &str,
        timeframe: &str,
        aggregate: Option<&str>,
        before_timestamp: Option<&str>,
        limit: Option<u32>,
        currency: Option<&str>,
        token: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!(
            "{}/networks/{}/pools/{}/ohlcv/{}",
            self.onchain_base_url(),
            network,
            pool_address,
            timeframe
        );
        let mut query = Vec::new();
        if let Some(v) = aggregate {
            query.push(("aggregate", v.to_string()));
        }
        if let Some(v) = before_timestamp {
            query.push(("before_timestamp", v.to_string()));
        }
        let lim;
        if let Some(v) = limit {
            lim = v.to_string();
            query.push(("limit", lim));
        }
        if let Some(v) = currency {
            query.push(("currency", v.to_string()));
        }
        if let Some(v) = token {
            query.push(("token", v.to_string()));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.get(&url, &q).await
    }

    /// Get pool trades.
    pub async fn onchain_pool_trades(
        &self,
        network: &str,
        pool_address: &str,
        trade_volume_in_usd_greater_than: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!(
            "{}/networks/{}/pools/{}/trades",
            self.onchain_base_url(),
            network,
            pool_address
        );
        let mut query = Vec::new();
        if let Some(v) = trade_volume_in_usd_greater_than {
            query.push(("trade_volume_in_usd_greater_than", v));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }

    /// Get supported DEXes by network.
    pub async fn onchain_dexes(
        &self,
        network: &str,
        page: Option<u32>,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/networks/{}/dexes", self.onchain_base_url(), network);
        let mut query = Vec::new();
        let pg;
        if let Some(v) = page {
            pg = v.to_string();
            query.push(("page", pg.as_str()));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }

    /// Get top pools by DEX.
    pub async fn onchain_dex_pools(
        &self,
        network: &str,
        dex: &str,
        include: Option<&str>,
        page: Option<u32>,
        sort: Option<&str>,
    ) -> anyhow::Result<OnchainPoolsResponse> {
        let url = format!(
            "{}/networks/{}/dexes/{}/pools",
            self.onchain_base_url(),
            network,
            dex
        );
        let mut query = Vec::new();
        let pg;
        if let Some(v) = include {
            query.push(("include", v));
        }
        if let Some(v) = page {
            pg = v.to_string();
            query.push(("page", pg.as_str()));
        }
        if let Some(v) = sort {
            query.push(("sort", v));
        }
        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        self.get(&url, &q).await
    }
}
