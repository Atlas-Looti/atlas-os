//! 0x Swap API v2 module — multi-chain DEX aggregator.
//!
//! Uses AllowanceHolder flow (recommended by 0x).
//! Supports 19+ EVM chains via chainId parameter.

use async_trait::async_trait;
use atlas_common::constants::{ATLAS_FEE_WALLET, BUILDER_FEE_BPS};
use atlas_common::error::{AtlasError, AtlasResult};
use atlas_common::traits::SwapModule;
use atlas_common::types::{Chain, Protocol, SwapQuote};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::info;

/// 0x API v2 base URL.
const ZEROX_API_BASE: &str = "https://api.0x.org";

/// AllowanceHolder address for Cancun hardfork chains (Ethereum, Arbitrum, Base, etc.)
pub const ALLOWANCE_HOLDER_CANCUN: &str = "0x0000000000001fF3684f28c67538d4D072C22734";

/// AllowanceHolder address for Shanghai hardfork chains (Mantle).
pub const ALLOWANCE_HOLDER_SHANGHAI: &str = "0x0000000000005E88410CcDFaDe4a5EfaE4b49562";

/// Permit2 address (universal across all chains).
pub const PERMIT2_ADDRESS: &str = "0x000000000022D473030F116dDEE9F6B43aC78BA3";

/// Native token placeholder address (works on all chains).
pub const NATIVE_TOKEN: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

// ── Chain ID Mapping ────────────────────────────────────────────────

/// Get the EVM chain ID for a given Chain enum.
pub fn chain_id(chain: &Chain) -> u64 {
    match chain {
        Chain::Ethereum => 1,
        Chain::Arbitrum => 42161,
        Chain::Base => 8453,
        Chain::HyperliquidL1 => 999, // Not supported by 0x
        Chain::Solana => 0,          // Not supported by 0x (non-EVM)
    }
}

/// Get the Chain enum from an EVM chain ID.
pub fn chain_from_id(id: u64) -> Option<Chain> {
    match id {
        1 => Some(Chain::Ethereum),
        42161 => Some(Chain::Arbitrum),
        8453 => Some(Chain::Base),
        _ => None,
    }
}

/// Check if a chain is supported by 0x.
pub fn is_supported(chain: &Chain) -> bool {
    !matches!(chain, Chain::Solana | Chain::HyperliquidL1)
}

// ── Response Types ──────────────────────────────────────────────────

/// 0x price/quote response (AllowanceHolder or Permit2).
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXQuoteResponse {
    /// Whether liquidity is available for this pair.
    pub liquidity_available: bool,

    /// Amount of buyToken to receive (in base units).
    #[serde(default)]
    pub buy_amount: Option<String>,

    /// Amount of sellToken to spend (in base units).
    #[serde(default)]
    pub sell_amount: Option<String>,

    /// Buy token contract address.
    #[serde(default)]
    pub buy_token: Option<String>,

    /// Sell token contract address.
    #[serde(default)]
    pub sell_token: Option<String>,

    /// The contract to set token allowance on (NOT the Settler).
    #[serde(default)]
    pub allowance_target: Option<String>,

    /// Minimum buy amount after slippage.
    #[serde(default)]
    pub min_buy_amount: Option<String>,

    /// Gas price in wei.
    #[serde(default)]
    pub gas_price: Option<String>,

    /// Block number the quote was sampled at.
    #[serde(default)]
    pub block_number: Option<String>,

    /// Route details (fills + tokens).
    #[serde(default)]
    pub route: Option<ZeroXRoute>,

    /// Fee breakdown.
    #[serde(default)]
    pub fees: Option<ZeroXFees>,

    /// Potential issues (allowance, balance, simulation).
    #[serde(default)]
    pub issues: Option<ZeroXIssues>,

    /// Transaction data (only in /quote responses, not /price).
    #[serde(default)]
    pub transaction: Option<ZeroXTransaction>,

    /// Token tax metadata.
    #[serde(default)]
    pub token_metadata: Option<serde_json::Value>,

    /// Total network fee.
    #[serde(default)]
    pub total_network_fee: Option<serde_json::Value>,

    /// Unique 0x request identifier.
    #[serde(default)]
    pub zid: Option<String>,
}

/// Route: how the swap is split across liquidity sources.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZeroXRoute {
    pub fills: Vec<ZeroXRouteFill>,
    pub tokens: Vec<ZeroXRouteToken>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXRouteFill {
    pub from: String,
    pub to: String,
    pub source: String,
    pub proportion_bps: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZeroXRouteToken {
    pub address: String,
    pub symbol: String,
}

/// Fee breakdown.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXFees {
    pub integrator_fee: Option<ZeroXFeeItem>,
    pub zero_ex_fee: Option<ZeroXFeeItem>,
    pub gas_fee: Option<ZeroXFeeItem>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZeroXFeeItem {
    pub amount: String,
    pub token: String,
    #[serde(rename = "type")]
    pub kind: String,
}

/// Issues that might prevent successful execution.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXIssues {
    pub allowance: Option<ZeroXAllowanceIssue>,
    pub balance: Option<ZeroXBalanceIssue>,
    pub simulation_incomplete: bool,
    #[serde(default)]
    pub invalid_sources_passed: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZeroXAllowanceIssue {
    pub actual: String,
    pub spender: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZeroXBalanceIssue {
    pub token: String,
    pub actual: String,
    pub expected: String,
}

/// Transaction data for submitting the swap on-chain.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXTransaction {
    /// Settler contract address — send the tx here.
    pub to: String,
    /// Encoded calldata.
    pub data: String,
    /// Estimated gas limit.
    pub gas: Option<serde_json::Value>,
    /// Gas price in wei.
    pub gas_price: String,
    /// ETH value to send (wei). Usually "0" for ERC20→ERC20.
    pub value: String,
}

/// Supported chains response.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZeroXChainsResponse {
    pub chains: Vec<ZeroXChainInfo>,
    pub zid: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXChainInfo {
    pub chain_id: u64,
    pub chain_name: String,
}

// ── Module ──────────────────────────────────────────────────────────

/// 0x Swap module — multi-chain DEX aggregator (API v2).
pub struct ZeroXModule {
    http: reqwest::Client,
    /// 0x API Key (from dashboard.0x.org).
    pub api_key: String,
    /// Atlas builder fee recipient address.
    pub fee_recipient: Option<String>,
    /// Atlas builder fee in bps (default: 1 bps = 0.01%).
    pub fee_bps: u16,
}

impl ZeroXModule {
    pub fn new(api_key: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");

        info!("0x module initialized (API v2, AllowanceHolder flow)");

        Self {
            http,
            api_key,
            fee_recipient: Some(ATLAS_FEE_WALLET.to_string()),
            fee_bps: BUILDER_FEE_BPS,
        }
    }

    /// Set the builder fee recipient and bps.
    pub fn with_fee(mut self, recipient: String, bps: u16) -> Self {
        self.fee_recipient = Some(recipient);
        self.fee_bps = bps;
        self
    }

    /// GET request with 0x v2 auth headers.
    async fn get(&self, url: &str, query: &[(&str, &str)]) -> AtlasResult<serde_json::Value> {
        let resp = self
            .http
            .get(url)
            .header("0x-api-key", &self.api_key)
            .header("0x-version", "v2")
            .query(query)
            .send()
            .await
            .map_err(|e| AtlasError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AtlasError::Protocol {
                protocol: "0x".into(),
                message: format!("0x API error {status}: {text}"),
            });
        }

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| AtlasError::Other(format!("Failed to parse 0x response: {e}")))
    }

    // ── Price (indicative, no commitment) ───────────────────────

    /// Get an indicative price for a swap (AllowanceHolder flow).
    /// Does NOT commit liquidity — use for display/comparison.
    pub async fn price(
        &self,
        chain: &Chain,
        sell_token: &str,
        buy_token: &str,
        sell_amount: &str,
        taker: Option<&str>,
        slippage_bps: Option<u32>,
    ) -> AtlasResult<ZeroXQuoteResponse> {
        if !is_supported(chain) {
            return Err(AtlasError::Protocol {
                protocol: "0x".into(),
                message: format!("Chain {chain} is not supported by 0x"),
            });
        }

        let cid = chain_id(chain).to_string();
        let url = format!("{ZEROX_API_BASE}/swap/allowance-holder/price");
        let mut query = vec![
            ("chainId", cid.as_str()),
            ("sellToken", sell_token),
            ("buyToken", buy_token),
            ("sellAmount", sell_amount),
        ];

        let slip;
        if let Some(s) = slippage_bps {
            slip = s.to_string();
            query.push(("slippageBps", &slip));
        }
        if let Some(t) = taker {
            query.push(("taker", t));
        }

        // Inject Atlas builder fee
        let bps;
        if let Some(ref recipient) = self.fee_recipient {
            bps = self.fee_bps.to_string();
            query.push(("swapFeeRecipient", recipient));
            query.push(("swapFeeBps", &bps));
        }

        let val = self.get(&url, &query).await?;
        serde_json::from_value(val)
            .map_err(|e| AtlasError::Other(format!("Failed to deserialize 0x price: {e}")))
    }

    // ── Quote (firm, commits liquidity) ─────────────────────────

    /// Get a firm quote for a swap (AllowanceHolder flow).
    /// Returns transaction data ready to submit on-chain.
    pub async fn firm_quote(
        &self,
        chain: &Chain,
        sell_token: &str,
        buy_token: &str,
        sell_amount: &str,
        taker: &str,
        slippage_bps: Option<u32>,
    ) -> AtlasResult<ZeroXQuoteResponse> {
        if !is_supported(chain) {
            return Err(AtlasError::Protocol {
                protocol: "0x".into(),
                message: format!("Chain {chain} is not supported by 0x"),
            });
        }

        let cid = chain_id(chain).to_string();
        let url = format!("{ZEROX_API_BASE}/swap/allowance-holder/quote");
        let mut query = vec![
            ("chainId", cid.as_str()),
            ("sellToken", sell_token),
            ("buyToken", buy_token),
            ("sellAmount", sell_amount),
            ("taker", taker),
        ];

        let slip;
        if let Some(s) = slippage_bps {
            slip = s.to_string();
            query.push(("slippageBps", &slip));
        }

        // Inject Atlas builder fee
        let bps;
        if let Some(ref recipient) = self.fee_recipient {
            bps = self.fee_bps.to_string();
            query.push(("swapFeeRecipient", recipient));
            query.push(("swapFeeBps", &bps));
        }

        let val = self.get(&url, &query).await?;
        serde_json::from_value(val)
            .map_err(|e| AtlasError::Other(format!("Failed to deserialize 0x quote: {e}")))
    }

    // ── Supported Chains ────────────────────────────────────────

    /// Get list of chains supported by 0x Swap API.
    pub async fn supported_chains(&self) -> AtlasResult<ZeroXChainsResponse> {
        let url = format!("{ZEROX_API_BASE}/swap/chains");
        let val = self.get(&url, &[]).await?;
        serde_json::from_value(val)
            .map_err(|e| AtlasError::Other(format!("Failed to deserialize 0x chains: {e}")))
    }

    // ── Liquidity Sources ───────────────────────────────────────

    /// Get available liquidity sources for a chain.
    pub async fn sources(&self, chain: &Chain) -> AtlasResult<serde_json::Value> {
        let cid = chain_id(chain).to_string();
        let url = format!("{ZEROX_API_BASE}/sources");
        self.get(&url, &[("chainId", &cid)]).await
    }
}

// ── SwapModule Trait Implementation ─────────────────────────────────

#[async_trait]
impl SwapModule for ZeroXModule {
    fn protocol(&self) -> Protocol {
        Protocol::ZeroX
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn quote(
        &self,
        sell_token: &str,
        buy_token: &str,
        amount: Decimal,
    ) -> AtlasResult<SwapQuote> {
        // Default to Ethereum if no chain context (trait doesn't pass chain)
        let chain = Chain::Ethereum;
        let sell_amount = amount.to_string();

        let resp = self
            .price(&chain, sell_token, buy_token, &sell_amount, None, Some(100))
            .await?;

        if !resp.liquidity_available {
            return Err(AtlasError::Protocol {
                protocol: "0x".into(),
                message: "No liquidity available for this pair".into(),
            });
        }

        let buy_amount = resp
            .buy_amount
            .as_deref()
            .unwrap_or("0")
            .parse::<Decimal>()
            .unwrap_or(Decimal::ZERO);

        // Compute effective price: buy_amount / sell_amount
        let price = if amount > Decimal::ZERO {
            buy_amount / amount
        } else {
            Decimal::ZERO
        };

        let estimated_gas = resp
            .transaction
            .as_ref()
            .and_then(|tx| {
                tx.gas
                    .as_ref()
                    .and_then(|g| g.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
            });

        let allowance_target = resp.allowance_target.clone();

        let tx_data = resp
            .transaction
            .as_ref()
            .map(|tx| tx.data.clone());

        Ok(SwapQuote {
            protocol: Protocol::ZeroX,
            chain,
            sell_token: sell_token.to_string(),
            buy_token: buy_token.to_string(),
            sell_amount: amount,
            buy_amount,
            estimated_gas,
            price,
            allowance_target,
            tx_data,
        })
    }

    async fn swap(&self, _quote: &SwapQuote) -> AtlasResult<String> {
        // On-chain execution requires signing + broadcasting via Alchemy/RPC
        // This will be implemented when backend RPC integration is ready
        Err(AtlasError::Other(
            "On-chain swap execution requires RPC integration (coming soon)".into(),
        ))
    }
}

// ── Trade Analytics Types ───────────────────────────────────────────

/// Trade Analytics response — completed swap trades.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradeAnalyticsResponse {
    /// Cursor for next page. `None` means no more pages.
    pub next_cursor: Option<String>,
    /// Completed trades.
    pub trades: Vec<SwapTrade>,
    /// Request identifier.
    pub zid: String,
}

/// A single completed swap trade from 0x Trade Analytics.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapTrade {
    /// App that initiated the trade.
    pub app_name: String,
    /// Block number.
    pub block_number: String,
    /// Token received by taker.
    pub buy_token: String,
    /// Amount received (formatted by token decimals).
    pub buy_amount: Option<String>,
    /// Chain ID.
    pub chain_id: u64,
    /// Chain name.
    pub chain_name: String,
    /// Fee breakdown.
    pub fees: SwapTradeFees,
    /// Gas consumed.
    pub gas_used: String,
    /// Protocol version (0xv4 or Settler).
    pub protocol_version: String,
    /// Token spent by taker.
    pub sell_token: String,
    /// Amount spent (formatted by token decimals).
    pub sell_amount: Option<String>,
    /// Slippage in bps.
    pub slippage_bps: Option<String>,
    /// Taker wallet address.
    pub taker: String,
    /// Block timestamp (unix seconds).
    pub timestamp: u64,
    /// Tokens involved in the trade.
    pub tokens: Vec<ZeroXRouteToken>,
    /// Transaction hash.
    pub transaction_hash: String,
    /// Trade volume in USD.
    pub volume_usd: Option<String>,
    /// 0x request ID that initiated this trade.
    pub zid: String,
    /// Service type ("swap").
    pub service: String,
}

/// Fee details for a completed trade.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapTradeFees {
    pub integrator_fee: Option<SwapTradeFeeItem>,
    pub zero_ex_fee: Option<SwapTradeFeeItem>,
}

/// Individual fee item (integrator or 0x).
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapTradeFeeItem {
    /// Token contract address.
    pub token: Option<String>,
    /// Fee amount (formatted by token decimals).
    pub amount: Option<String>,
    /// Fee amount in USD.
    pub amount_usd: Option<String>,
}

// ── Trade Analytics Implementation ──────────────────────────────────

impl ZeroXModule {
    /// Get completed swap trades from Trade Analytics API.
    ///
    /// - `cursor`: pagination cursor from previous response (`None` for first page)
    /// - `start_timestamp`: unix seconds, trades on or after this time
    /// - `end_timestamp`: unix seconds, trades on or before this time
    ///
    /// Returns max 200 trades per request. Use `next_cursor` for pagination.
    pub async fn swap_trades(
        &self,
        cursor: Option<&str>,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> AtlasResult<TradeAnalyticsResponse> {
        let url = format!("{ZEROX_API_BASE}/trade-analytics/swap");
        let mut query = Vec::new();

        let start_ts;
        let end_ts;

        if let Some(c) = cursor {
            query.push(("cursor", c));
        }
        if let Some(s) = start_timestamp {
            start_ts = s.to_string();
            query.push(("startTimestamp", &start_ts));
        }
        if let Some(e) = end_timestamp {
            end_ts = e.to_string();
            query.push(("endTimestamp", &end_ts));
        }

        let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, *v)).collect();
        let val = self.get(&url, &q).await?;
        serde_json::from_value(val)
            .map_err(|e| AtlasError::Other(format!("Failed to deserialize trade analytics: {e}")))
    }

    /// Get ALL swap trades by paginating through results.
    /// Caution: can make many API calls for large time ranges.
    pub async fn all_swap_trades(
        &self,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> AtlasResult<Vec<SwapTrade>> {
        let mut all_trades = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let resp = self
                .swap_trades(cursor.as_deref(), start_timestamp, end_timestamp)
                .await?;

            all_trades.extend(resp.trades);

            match resp.next_cursor {
                Some(c) if !c.is_empty() => cursor = Some(c),
                _ => break,
            }
        }

        Ok(all_trades)
    }
}
