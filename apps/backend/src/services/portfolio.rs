//! Portfolio service — multi-chain token balances, prices, metadata.


use serde::{Deserialize, Serialize};

use crate::clients::alchemy::{AlchemyClient, PortfolioToken, TokenMetadata};
use crate::clients::cache::{Cache, CacheTtl};

/// Portfolio service with caching.
pub struct PortfolioService;

#[derive(Serialize, Deserialize, Clone)]
pub struct WalletPortfolio {
    pub address: String,
    pub networks: Vec<String>,
    pub tokens: Vec<PortfolioToken>,
    pub total_value_usd: f64,
}

impl PortfolioService {
    /// Get full portfolio for an address across multiple chains.
    pub async fn get_portfolio(
        alchemy: &AlchemyClient,
        cache: &Cache,
        address: &str,
        networks: &[&str],
    ) -> anyhow::Result<WalletPortfolio> {
        let cache_key = Cache::key("portfolio", &[address, &networks.join(",")]);

        // Check cache
        if let Some(cached) = cache.get::<WalletPortfolio>(&cache_key).await {
            return Ok(cached);
        }

        // Fetch from Alchemy
        let tokens = alchemy.get_portfolio(address, networks).await?;

        // Calculate total USD value
        let total_value_usd: f64 = tokens.iter()
            .filter_map(|t| {
                let prices = t.token_prices.as_ref()?;
                let usd_price = prices.iter().find(|p| p.currency == "usd")?;
                let price: f64 = usd_price.value.parse().ok()?;
                let metadata = t.token_metadata.as_ref()?;
                let decimals = metadata.decimals.unwrap_or(18) as u32;
                let balance = hex_to_f64(&t.token_balance, decimals);
                Some(balance * price)
            })
            .sum();

        let result = WalletPortfolio {
            address: address.to_string(),
            networks: networks.iter().map(|s| s.to_string()).collect(),
            tokens,
            total_value_usd,
        };

        // Cache it
        let _ = cache.set(&cache_key, &result, CacheTtl::PORTFOLIO).await;

        Ok(result)
    }

    /// Get token metadata with 24h cache.
    pub async fn get_token_metadata(
        alchemy: &AlchemyClient,
        cache: &Cache,
        network: &str,
        contract: &str,
    ) -> anyhow::Result<TokenMetadata> {
        let cache_key = Cache::key("token_meta", &[network, contract]);

        if let Some(cached) = cache.get::<TokenMetadata>(&cache_key).await {
            return Ok(cached);
        }

        let metadata = alchemy.get_token_metadata(network, contract).await?;
        let _ = cache.set(&cache_key, &metadata, CacheTtl::TOKEN_METADATA).await;

        Ok(metadata)
    }
}

/// Convert hex balance string to f64 with decimal adjustment.
fn hex_to_f64(hex_str: &str, decimals: u32) -> f64 {
    let hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let value = u128::from_str_radix(hex, 16).unwrap_or(0);
    value as f64 / 10f64.powi(decimals as i32)
}
