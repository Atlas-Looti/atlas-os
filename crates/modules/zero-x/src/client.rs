//! 0x Swap API v2 module — multi-chain DEX aggregator.
//!
//! Uses AllowanceHolder flow (recommended by 0x).
//! Supports 19+ EVM chains via chainId parameter.

use async_trait::async_trait;
use atlas_core::constants::{ATLAS_FEE_WALLET, BUILDER_FEE_BPS};
use atlas_core::error::{AtlasError, AtlasResult};
use atlas_core::traits::SwapModule;
use atlas_core::types::{Chain, Protocol, SwapQuote};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::info;

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;

/// Atlas backend API sub-route for 0x proxy.
/// Backend mounts at /atlas-os/0x (see apps/backend index).
const ZEROX_API_BASE: &str = "/atlas-os/0x";

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
        Chain::HyperliquidL1 => 998, // HyperEVM mainnet (not supported by 0x)
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
    /// Backend API URL (no trailing slash).
    pub backend_url: String,
    /// Atlas API key for backend auth (X-API-Key / Authorization: Bearer).
    pub api_key: Option<String>,
    /// Default chain for SwapModule::quote() when trait doesn't pass chain.
    pub default_chain: Chain,
    /// Default slippage in bps for quote().
    pub default_slippage_bps: u32,
    /// Atlas builder fee recipient address.
    pub fee_recipient: Option<String>,
    /// Atlas builder fee in bps (default: 1 bps = 0.01%).
    pub fee_bps: u16,
    /// EVM signer for on-chain execution (None = quote-only mode).
    signer: Option<PrivateKeySigner>,
}

/// Parse chain name (e.g. from config) to Chain enum. Falls back to Ethereum if unknown.
pub fn parse_chain(s: &str) -> Chain {
    match s.to_lowercase().as_str() {
        "ethereum" | "eth" | "1" => Chain::Ethereum,
        "arbitrum" | "arb" | "42161" => Chain::Arbitrum,
        "base" | "8453" => Chain::Base,
        _ => Chain::Ethereum,
    }
}

impl ZeroXModule {
    pub fn new(backend_url: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");

        info!("0x module initialized (API v2, AllowanceHolder flow)");

        let backend_url = backend_url.trim_end_matches('/').to_string();
        Self {
            http,
            backend_url,
            api_key: None,
            default_chain: Chain::Ethereum,
            default_slippage_bps: 100,
            fee_recipient: Some(ATLAS_FEE_WALLET.to_string()),
            fee_bps: BUILDER_FEE_BPS,
            signer: None,
        }
    }

    /// Set Atlas API key for backend auth (required for /atlas-os/0x/*).
    pub fn with_api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
        self
    }

    /// Set default chain and slippage (from config) for SwapModule::quote().
    pub fn with_defaults(mut self, default_chain: Chain, default_slippage_bps: u32) -> Self {
        self.default_chain = default_chain;
        self.default_slippage_bps = default_slippage_bps;
        self
    }

    /// Set the builder fee recipient and bps.
    pub fn with_fee(mut self, recipient: String, bps: u16) -> Self {
        self.fee_recipient = Some(recipient);
        self.fee_bps = bps;
        self
    }

    /// Set the EVM signer for on-chain swap execution.
    pub fn with_signer(mut self, signer: PrivateKeySigner) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Get the taker address (signer's address) if available.
    pub fn taker_address(&self) -> Option<String> {
        self.signer.as_ref().map(|s| format!("{:?}", s.address()))
    }

    /// Build the RPC URL for a given chain via the Atlas backend proxy.
    /// Uses Alchemy-style key-in-URL: /atlas-os/rpc/v2/{api_key}/{chain}
    fn rpc_url(&self, chain: &Chain) -> String {
        let chain_slug = match chain {
            Chain::Ethereum => "ethereum",
            Chain::Arbitrum => "arbitrum",
            Chain::Base => "base",
            Chain::HyperliquidL1 => "hyperevm",
            Chain::Solana => "solana", // won't work, non-EVM
        };
        let key = self.api_key.as_deref().unwrap_or("none");
        format!("{}/atlas-os/rpc/v2/{}/{}", self.backend_url, key, chain_slug)
    }

    /// Build an alloy provider pointing at the Atlas backend RPC proxy.
    async fn build_provider(
        &self,
        chain: &Chain,
    ) -> AtlasResult<impl Provider> {
        let signer = self.signer.clone().ok_or_else(|| {
            AtlasError::Auth("No signer available. Import a wallet first: `atlas profile import`".into())
        })?;

        let rpc_url: alloy::transports::http::reqwest::Url = self.rpc_url(chain)
            .parse()
            .map_err(|e| AtlasError::Other(format!("Invalid RPC URL: {e}")))?;

        let wallet = EthereumWallet::from(signer);
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect_http(rpc_url);

        Ok(provider)
    }

    /// Approve a spender (AllowanceHolder) to spend an ERC20 token.
    /// Approves the exact sell amount from the quote (not unlimited).
    async fn approve_token(
        &self,
        chain: &Chain,
        token: &str,
        spender: &str,
        amount: &SwapQuote,
    ) -> AtlasResult<()> {
        let provider = self.build_provider(chain).await?;

        let token_addr: Address = token
            .parse()
            .map_err(|e| AtlasError::Other(format!("Invalid token address: {e}")))?;

        let spender_addr: Address = spender
            .parse()
            .map_err(|e| AtlasError::Other(format!("Invalid spender address: {e}")))?;

        // ERC20 approve(address spender, uint256 amount)
        // selector: 0x095ea7b3
        // Approve exact sell amount (not unlimited — safer)
        let approve_amount = U256::from_str_radix(
            &amount.sell_amount.to_string(),
            10,
        )
        .unwrap_or(U256::MAX);

        let mut calldata = Vec::with_capacity(68);
        calldata.extend_from_slice(&hex::decode("095ea7b3").unwrap());
        // spender address (padded to 32 bytes)
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(spender_addr.as_slice());
        // amount (32 bytes, big-endian)
        calldata.extend_from_slice(&approve_amount.to_be_bytes::<32>());

        let tx_req = TransactionRequest::default()
            .to(token_addr)
            .input(Bytes::from(calldata).into());

        info!("Sending ERC20 approve tx for {} → {}", token, spender);

        let pending = provider
            .send_transaction(tx_req)
            .await
            .map_err(|e| AtlasError::Network(format!("Failed to send approve tx: {e}")))?;

        let receipt = pending
            .get_receipt()
            .await
            .map_err(|e| AtlasError::Network(format!("Failed to get approve receipt: {e}")))?;

        if !receipt.status() {
            return Err(AtlasError::Protocol {
                protocol: "0x".into(),
                message: format!("Token approval reverted for {}", token),
            });
        }

        info!("Token approval confirmed for {}", token);
        Ok(())
    }

    /// GET request to Atlas backend. Sends Authorization when api_key is set.
    async fn get(&self, path: &str, query: &[(&str, &str)]) -> AtlasResult<serde_json::Value> {
        let url = format!("{}{}", self.backend_url, path);
        let mut req = self.http.get(&url).query(query);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        let resp = req
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
        let path = format!("{ZEROX_API_BASE}/swap/allowance-holder/price");
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

        let val = self.get(&path, &query).await?;
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
        let path = format!("{ZEROX_API_BASE}/swap/allowance-holder/quote");
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

        let val = self.get(&path, &query).await?;
        serde_json::from_value(val)
            .map_err(|e| AtlasError::Other(format!("Failed to deserialize 0x quote: {e}")))
    }

    // ── Supported Chains ────────────────────────────────────────

    /// Get list of chains supported by 0x Swap API.
    pub async fn supported_chains(&self) -> AtlasResult<ZeroXChainsResponse> {
        let path = format!("{ZEROX_API_BASE}/swap/chains");
        let val = self.get(&path, &[]).await?;
        serde_json::from_value(val)
            .map_err(|e| AtlasError::Other(format!("Failed to deserialize 0x chains: {e}")))
    }

    // ── Liquidity Sources ───────────────────────────────────────

    /// Get available liquidity sources for a chain.
    pub async fn sources(&self, chain: &Chain) -> AtlasResult<serde_json::Value> {
        let cid = chain_id(chain).to_string();
        let path = format!("{ZEROX_API_BASE}/sources");
        self.get(&path, &[("chainId", &cid)]).await
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
        let sell_amount = amount.to_string();
        let resp = self
            .price(
                &self.default_chain,
                sell_token,
                buy_token,
                &sell_amount,
                None,
                Some(self.default_slippage_bps),
            )
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

        let estimated_gas = resp.transaction.as_ref().and_then(|tx| {
            tx.gas
                .as_ref()
                .and_then(|g| g.as_str())
                .and_then(|s| s.parse::<u64>().ok())
        });

        let allowance_target = resp.allowance_target.clone();

        let tx_data = resp.transaction.as_ref().map(|tx| tx.data.clone());

        Ok(SwapQuote {
            protocol: Protocol::ZeroX,
            chain: self.default_chain.clone(),
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

    async fn swap(&self, quote: &SwapQuote) -> AtlasResult<String> {
        let signer = self.signer.as_ref().ok_or_else(|| {
            AtlasError::Auth("No signer available. Import a wallet first: `atlas profile import`".into())
        })?;
        let taker = format!("{:?}", signer.address());

        // 1. Get a firm quote with transaction data
        let firm = self
            .firm_quote(
                &quote.chain,
                &quote.sell_token,
                &quote.buy_token,
                &quote.sell_amount.to_string(),
                &taker,
                Some(self.default_slippage_bps),
            )
            .await?;

        if !firm.liquidity_available {
            return Err(AtlasError::Protocol {
                protocol: "0x".into(),
                message: "No liquidity available for this swap".into(),
            });
        }

        let tx_data = firm.transaction.as_ref().ok_or_else(|| {
            AtlasError::Protocol {
                protocol: "0x".into(),
                message: "0x quote did not return transaction data. The pair may not be swappable.".into(),
            }
        })?;

        // 2. Check if we need token approval (skip for native ETH sells)
        let is_native = quote.sell_token.to_lowercase() == NATIVE_TOKEN;
        if !is_native {
            if let Some(ref issues) = firm.issues {
                if issues.allowance.is_some() {
                    // Need to approve the AllowanceHolder to spend our tokens
                    let spender = firm
                        .allowance_target
                        .as_deref()
                        .unwrap_or(ALLOWANCE_HOLDER_CANCUN);

                    info!(
                        "Setting token approval for {} on {}",
                        quote.sell_token, spender
                    );

                    self.approve_token(&quote.chain, &quote.sell_token, spender, quote)
                        .await?;
                }
            }
        }

        // 3. Build and send the swap transaction
        let provider = self.build_provider(&quote.chain).await?;

        let to: Address = tx_data
            .to
            .parse()
            .map_err(|e| AtlasError::Other(format!("Invalid 'to' address: {e}")))?;

        let data_bytes = hex::decode(tx_data.data.strip_prefix("0x").unwrap_or(&tx_data.data))
            .map_err(|e| AtlasError::Other(format!("Invalid tx calldata: {e}")))?;

        let value = U256::from_str_radix(
            tx_data.value.strip_prefix("0x").unwrap_or(&tx_data.value),
            if tx_data.value.starts_with("0x") { 16 } else { 10 },
        )
        .unwrap_or(U256::ZERO);

        let tx_req = TransactionRequest::default()
            .to(to)
            .input(Bytes::from(data_bytes).into())
            .value(value);

        info!("Sending swap transaction to {}", tx_data.to);

        let pending = provider
            .send_transaction(tx_req)
            .await
            .map_err(|e| AtlasError::Network(format!("Failed to send swap tx: {e}")))?;

        let tx_hash = format!("{:?}", pending.tx_hash());
        info!("Swap tx sent: {}", tx_hash);

        // 4. Wait for confirmation
        let receipt = pending
            .get_receipt()
            .await
            .map_err(|e| AtlasError::Network(format!("Failed to get tx receipt: {e}")))?;

        if !receipt.status() {
            return Err(AtlasError::Protocol {
                protocol: "0x".into(),
                message: format!("Swap transaction reverted: {tx_hash}"),
            });
        }

        info!("Swap confirmed: {}", tx_hash);
        Ok(tx_hash)
    }
}

