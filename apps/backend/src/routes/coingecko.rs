//! CoinGecko API routes — comprehensive DeFi/CEX market data.
//!
//! All endpoints are prefixed with `/api/coingecko/`.
//! Requires `COINGECKO_API_KEY` env var to be set.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::services::coingecko::CoinGeckoService;
use crate::state::AppState;

// ── Query Params ────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PriceQuery {
    /// Coin IDs, comma-separated (e.g. "bitcoin,ethereum")
    ids: String,
    /// Target currencies, comma-separated (default: "usd")
    vs_currencies: Option<String>,
    /// Include market cap
    include_market_cap: Option<bool>,
    /// Include 24hr volume
    include_24hr_vol: Option<bool>,
    /// Include 24hr change
    include_24hr_change: Option<bool>,
    /// Include last updated timestamp
    include_last_updated_at: Option<bool>,
    /// Decimal precision (e.g. "2", "full")
    precision: Option<String>,
}

#[derive(Deserialize)]
pub struct TokenPriceQuery {
    /// Contract addresses, comma-separated
    contract_addresses: String,
    /// Target currencies (default: "usd")
    vs_currencies: Option<String>,
    include_market_cap: Option<bool>,
    include_24hr_vol: Option<bool>,
    include_24hr_change: Option<bool>,
    include_last_updated_at: Option<bool>,
}

#[derive(Deserialize)]
pub struct MarketsQuery {
    /// Target currency (default: "usd")
    vs_currency: Option<String>,
    /// Coin IDs filter, comma-separated
    ids: Option<String>,
    /// Category filter
    category: Option<String>,
    /// Sort order (e.g. "market_cap_desc", "volume_desc")
    order: Option<String>,
    /// Results per page (default: 100, max: 250)
    per_page: Option<u32>,
    /// Page number (default: 1)
    page: Option<u32>,
    /// Include sparkline data
    sparkline: Option<bool>,
    /// Price change percentage intervals (e.g. "1h,24h,7d")
    price_change_percentage: Option<String>,
}

#[derive(Deserialize)]
pub struct ChartQuery {
    /// Target currency (default: "usd")
    vs_currency: Option<String>,
    /// Number of days (e.g. "1", "7", "30", "365", "max")
    days: Option<String>,
    /// Data interval (e.g. "daily", "hourly")
    interval: Option<String>,
    /// Decimal precision
    precision: Option<String>,
}

#[derive(Deserialize)]
pub struct ChartRangeQuery {
    /// Target currency (default: "usd")
    vs_currency: Option<String>,
    /// UNIX timestamp start
    from: String,
    /// UNIX timestamp end
    to: String,
    /// Decimal precision
    precision: Option<String>,
}

#[derive(Deserialize)]
pub struct OhlcQuery {
    /// Target currency (default: "usd")
    vs_currency: Option<String>,
    /// Number of days (1, 7, 14, 30, 90, 180, 365, max)
    days: Option<String>,
    /// Decimal precision
    precision: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    /// Search query string
    q: String,
}

#[derive(Deserialize)]
pub struct CategoriesQuery {
    /// Sort order (e.g. "market_cap_desc", "market_cap_asc", "name_desc", "name_asc",
    /// "market_cap_change_24h_desc", "market_cap_change_24h_asc")
    order: Option<String>,
}

#[derive(Deserialize)]
pub struct ExchangesQuery {
    per_page: Option<u32>,
    page: Option<u32>,
}

#[derive(Deserialize)]
pub struct CoinsListQuery {
    /// Include platform contract addresses
    include_platform: Option<bool>,
}

#[derive(Deserialize)]
pub struct TopMoversQuery {
    /// Target currency (default: "usd")
    vs_currency: Option<String>,
    /// Duration: "1h", "24h", "7d", "14d", "30d", "60d", "1y"
    duration: Option<String>,
}

#[derive(Deserialize)]
pub struct TickersQuery {
    /// Filter by exchange IDs
    exchange_ids: Option<String>,
    /// Include exchange logo
    include_exchange_logo: Option<bool>,
    page: Option<u32>,
    order: Option<String>,
    depth: Option<bool>,
}

#[derive(Deserialize)]
pub struct OnchainNetworkQuery {
    page: Option<u32>,
    sort: Option<String>,
}

#[derive(Deserialize)]
pub struct OnchainPoolQuery {
    include: Option<String>,
}

#[derive(Deserialize)]
pub struct OnchainPoolOhlcvQuery {
    aggregate: Option<String>,
    before_timestamp: Option<String>,
    limit: Option<u32>,
    currency: Option<String>,
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct OnchainTradesQuery {
    trade_volume_in_usd_greater_than: Option<String>,
}

#[derive(Deserialize)]
pub struct OnchainSearchQuery {
    q: String,
    network: Option<String>,
    include: Option<String>,
    page: Option<u32>,
}

#[derive(Deserialize)]
pub struct OnchainTokenPoolsQuery {
    include: Option<String>,
    page: Option<u32>,
    sort: Option<String>,
}

// ── Helper ──────────────────────────────────────────────────────────

fn cg_not_configured() -> Json<Value> {
    Json(json!({
        "error": "CoinGecko API not configured. Set COINGECKO_API_KEY env var."
    }))
}

// ── Route Handlers ──────────────────────────────────────────────────

/// GET /api/coingecko/ping
async fn ping(State(state): State<Arc<AppState>>) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg.ping().await {
        Ok(resp) => Json(json!(resp)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/usage
async fn usage(State(state): State<Arc<AppState>>) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg.api_usage().await {
        Ok(resp) => Json(json!(resp)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/price?ids=bitcoin,ethereum&vs_currencies=usd&include_market_cap=true...
async fn simple_price(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PriceQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    let vs = q.vs_currencies.as_deref().unwrap_or("usd");
    let precision = q.precision.as_deref();

    match CoinGeckoService::simple_price(
        cg,
        state.cache.as_ref(),
        &q.ids,
        vs,
        q.include_market_cap.unwrap_or(true),
        q.include_24hr_vol.unwrap_or(true),
        q.include_24hr_change.unwrap_or(true),
        q.include_last_updated_at.unwrap_or(true),
        precision,
    )
    .await
    {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/token-price/:platform?contract_addresses=0x...&vs_currencies=usd
async fn token_price(
    State(state): State<Arc<AppState>>,
    Path(platform): Path<String>,
    Query(q): Query<TokenPriceQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    let vs = q.vs_currencies.as_deref().unwrap_or("usd");

    match CoinGeckoService::simple_token_price(
        cg,
        state.cache.as_ref(),
        &platform,
        &q.contract_addresses,
        vs,
        q.include_market_cap.unwrap_or(false),
        q.include_24hr_vol.unwrap_or(false),
        q.include_24hr_change.unwrap_or(false),
        q.include_last_updated_at.unwrap_or(false),
    )
    .await
    {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/currencies
async fn supported_currencies(State(state): State<Arc<AppState>>) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::supported_vs_currencies(cg, state.cache.as_ref()).await {
        Ok(data) => Json(json!({ "currencies": data })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/coins/list?include_platform=true
async fn coins_list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CoinsListQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::coins_list(
        cg,
        state.cache.as_ref(),
        q.include_platform.unwrap_or(false),
    )
    .await
    {
        Ok(data) => Json(json!({ "coins": data, "count": data.len() })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/coins/markets?vs_currency=usd&order=market_cap_desc&per_page=50
async fn coins_markets(
    State(state): State<Arc<AppState>>,
    Query(q): Query<MarketsQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    let vs = q.vs_currency.as_deref().unwrap_or("usd");
    let pcp = q.price_change_percentage.as_deref();

    match CoinGeckoService::coins_markets(
        cg,
        state.cache.as_ref(),
        vs,
        q.ids.as_deref(),
        q.category.as_deref(),
        q.order.as_deref(),
        q.per_page,
        q.page,
        q.sparkline.unwrap_or(false),
        pcp,
    )
    .await
    {
        Ok(data) => Json(json!({ "coins": data, "count": data.len() })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/coins/:id
async fn coin_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::coin_by_id(cg, state.cache.as_ref(), &id).await {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/coins/:id/tickers
async fn coin_tickers(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(q): Query<TickersQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg
        .coin_tickers(
            &id,
            q.exchange_ids.as_deref(),
            q.include_exchange_logo.unwrap_or(false),
            q.page,
            q.order.as_deref(),
            q.depth.unwrap_or(false),
        )
        .await
    {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/coins/:id/chart?vs_currency=usd&days=7
async fn coin_chart(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(q): Query<ChartQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    let vs = q.vs_currency.as_deref().unwrap_or("usd");
    let days = q.days.as_deref().unwrap_or("7");

    match CoinGeckoService::coin_market_chart(
        cg,
        state.cache.as_ref(),
        &id,
        vs,
        days,
        q.interval.as_deref(),
        q.precision.as_deref(),
    )
    .await
    {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/coins/:id/chart/range?from=1711356300&to=1711442700&vs_currency=usd
async fn coin_chart_range(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(q): Query<ChartRangeQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    let vs = q.vs_currency.as_deref().unwrap_or("usd");

    match cg
        .coin_market_chart_range(&id, vs, &q.from, &q.to, q.precision.as_deref())
        .await
    {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/coins/:id/ohlc?vs_currency=usd&days=7
async fn coin_ohlc(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(q): Query<OhlcQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    let vs = q.vs_currency.as_deref().unwrap_or("usd");
    let days = q.days.as_deref().unwrap_or("7");

    match CoinGeckoService::coin_ohlc(
        cg,
        state.cache.as_ref(),
        &id,
        vs,
        days,
        q.precision.as_deref(),
    )
    .await
    {
        Ok(data) => Json(json!({ "ohlc": data })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/trending
async fn trending(State(state): State<Arc<AppState>>) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::trending(cg, state.cache.as_ref()).await {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/search?q=bitcoin
async fn search(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SearchQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::search(cg, state.cache.as_ref(), &q.q).await {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/global
async fn global(State(state): State<Arc<AppState>>) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::global(cg, state.cache.as_ref()).await {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/global/defi
async fn global_defi(State(state): State<Arc<AppState>>) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::global_defi(cg, state.cache.as_ref()).await {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/categories?order=market_cap_desc
async fn categories(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CategoriesQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::categories(cg, state.cache.as_ref(), q.order.as_deref()).await {
        Ok(data) => Json(json!({ "categories": data, "count": data.len() })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/exchanges?per_page=50&page=1
async fn exchanges(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ExchangesQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::exchanges(cg, state.cache.as_ref(), q.per_page, q.page).await {
        Ok(data) => Json(json!({ "exchanges": data, "count": data.len() })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/exchanges/:id
async fn exchange_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg.exchange_by_id(&id).await {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/exchange-rates
async fn exchange_rates(State(state): State<Arc<AppState>>) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg.exchange_rates().await {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/top-movers?vs_currency=usd&duration=24h (Pro only)
async fn top_movers(
    State(state): State<Arc<AppState>>,
    Query(q): Query<TopMoversQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    let vs = q.vs_currency.as_deref().unwrap_or("usd");

    match cg.top_gainers_losers(vs, q.duration.as_deref()).await {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

// ── On-chain / GeckoTerminal Routes ─────────────────────────────────

/// GET /api/coingecko/onchain/networks
async fn onchain_networks(State(state): State<Arc<AppState>>) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::onchain_networks(cg, state.cache.as_ref()).await {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/trending-pools
/// GET /api/coingecko/onchain/trending-pools/:network
async fn onchain_trending_pools(
    State(state): State<Arc<AppState>>,
    network: Option<Path<String>>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    let net = network.as_ref().map(|p| p.0.as_str());

    match CoinGeckoService::onchain_trending_pools(cg, state.cache.as_ref(), net).await {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/new-pools
/// GET /api/coingecko/onchain/new-pools/:network
async fn onchain_new_pools(
    State(state): State<Arc<AppState>>,
    network: Option<Path<String>>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    let net = network.as_ref().map(|p| p.0.as_str());

    match CoinGeckoService::onchain_new_pools(cg, state.cache.as_ref(), net).await {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/pools/:network?page=1&sort=h24_tx_count_desc
async fn onchain_top_pools(
    State(state): State<Arc<AppState>>,
    Path(network): Path<String>,
    Query(q): Query<OnchainNetworkQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match CoinGeckoService::onchain_top_pools(
        cg,
        state.cache.as_ref(),
        &network,
        q.page,
        q.sort.as_deref(),
    )
    .await
    {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/pools/:network/:address
async fn onchain_pool_detail(
    State(state): State<Arc<AppState>>,
    Path((network, address)): Path<(String, String)>,
    Query(q): Query<OnchainPoolQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg.onchain_pool(&network, &address, q.include.as_deref()).await {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/pools/:network/:address/ohlcv/:timeframe
async fn onchain_pool_ohlcv(
    State(state): State<Arc<AppState>>,
    Path((network, address, timeframe)): Path<(String, String, String)>,
    Query(q): Query<OnchainPoolOhlcvQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg
        .onchain_pool_ohlcv(
            &network,
            &address,
            &timeframe,
            q.aggregate.as_deref(),
            q.before_timestamp.as_deref(),
            q.limit,
            q.currency.as_deref(),
            q.token.as_deref(),
        )
        .await
    {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/pools/:network/:address/trades
async fn onchain_pool_trades(
    State(state): State<Arc<AppState>>,
    Path((network, address)): Path<(String, String)>,
    Query(q): Query<OnchainTradesQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg
        .onchain_pool_trades(&network, &address, q.trade_volume_in_usd_greater_than.as_deref())
        .await
    {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/tokens/:network/:address/pools
async fn onchain_token_pools(
    State(state): State<Arc<AppState>>,
    Path((network, address)): Path<(String, String)>,
    Query(q): Query<OnchainTokenPoolsQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg
        .onchain_token_pools(
            &network,
            &address,
            q.include.as_deref(),
            q.page,
            q.sort.as_deref(),
        )
        .await
    {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/tokens/:network/:address/info
async fn onchain_token_info(
    State(state): State<Arc<AppState>>,
    Path((network, address)): Path<(String, String)>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg.onchain_token_info(&network, &address).await {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/dexes/:network
async fn onchain_dexes(
    State(state): State<Arc<AppState>>,
    Path(network): Path<String>,
    Query(q): Query<OnchainNetworkQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg.onchain_dexes(&network, q.page).await {
        Ok(data) => Json(data),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/dexes/:network/:dex/pools
async fn onchain_dex_pools(
    State(state): State<Arc<AppState>>,
    Path((network, dex)): Path<(String, String)>,
    Query(q): Query<OnchainTokenPoolsQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg
        .onchain_dex_pools(
            &network,
            &dex,
            q.include.as_deref(),
            q.page,
            q.sort.as_deref(),
        )
        .await
    {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /api/coingecko/onchain/search?q=weth&network=eth
async fn onchain_search(
    State(state): State<Arc<AppState>>,
    Query(q): Query<OnchainSearchQuery>,
) -> Json<Value> {
    let cg = match &state.coingecko {
        Some(c) => c,
        None => return cg_not_configured(),
    };

    match cg
        .onchain_search_pools(&q.q, q.network.as_deref(), q.include.as_deref(), q.page)
        .await
    {
        Ok(data) => Json(json!(data)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

// ── Router ──────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // System
        .route("/coingecko/ping", get(ping))
        .route("/coingecko/usage", get(usage))
        // Simple
        .route("/coingecko/price", get(simple_price))
        .route("/coingecko/token-price/{platform}", get(token_price))
        .route("/coingecko/currencies", get(supported_currencies))
        // Coins
        .route("/coingecko/coins/list", get(coins_list))
        .route("/coingecko/coins/markets", get(coins_markets))
        .route("/coingecko/coins/{id}", get(coin_detail))
        .route("/coingecko/coins/{id}/tickers", get(coin_tickers))
        .route("/coingecko/coins/{id}/chart", get(coin_chart))
        .route("/coingecko/coins/{id}/chart/range", get(coin_chart_range))
        .route("/coingecko/coins/{id}/ohlc", get(coin_ohlc))
        // Discovery
        .route("/coingecko/trending", get(trending))
        .route("/coingecko/search", get(search))
        .route("/coingecko/top-movers", get(top_movers))
        // Global
        .route("/coingecko/global", get(global))
        .route("/coingecko/global/defi", get(global_defi))
        // Reference
        .route("/coingecko/categories", get(categories))
        .route("/coingecko/exchanges", get(exchanges))
        .route("/coingecko/exchanges/{id}", get(exchange_detail))
        .route("/coingecko/exchange-rates", get(exchange_rates))
        // On-chain / GeckoTerminal
        .route("/coingecko/onchain/networks", get(onchain_networks))
        .route("/coingecko/onchain/trending-pools", get(onchain_trending_pools_all))
        .route("/coingecko/onchain/trending-pools/{network}", get(onchain_trending_pools_network))
        .route("/coingecko/onchain/new-pools", get(onchain_new_pools_all))
        .route("/coingecko/onchain/new-pools/{network}", get(onchain_new_pools_network))
        .route("/coingecko/onchain/pools/{network}", get(onchain_top_pools))
        .route("/coingecko/onchain/pools/{network}/{address}", get(onchain_pool_detail))
        .route("/coingecko/onchain/pools/{network}/{address}/ohlcv/{timeframe}", get(onchain_pool_ohlcv))
        .route("/coingecko/onchain/pools/{network}/{address}/trades", get(onchain_pool_trades))
        .route("/coingecko/onchain/tokens/{network}/{address}/pools", get(onchain_token_pools))
        .route("/coingecko/onchain/tokens/{network}/{address}/info", get(onchain_token_info))
        .route("/coingecko/onchain/dexes/{network}", get(onchain_dexes))
        .route("/coingecko/onchain/dexes/{network}/{dex}/pools", get(onchain_dex_pools))
        .route("/coingecko/onchain/search", get(onchain_search))
}

// Separate handlers for with/without network param (axum needs distinct handlers)

async fn onchain_trending_pools_all(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    onchain_trending_pools(State(state), None).await
}

async fn onchain_trending_pools_network(
    State(state): State<Arc<AppState>>,
    Path(network): Path<String>,
) -> Json<Value> {
    onchain_trending_pools(State(state), Some(Path(network))).await
}

async fn onchain_new_pools_all(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    onchain_new_pools(State(state), None).await
}

async fn onchain_new_pools_network(
    State(state): State<Arc<AppState>>,
    Path(network): Path<String>,
) -> Json<Value> {
    onchain_new_pools(State(state), Some(Path(network))).await
}
