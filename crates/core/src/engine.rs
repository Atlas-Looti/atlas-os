// Legacy engine — retained only for DB sync operations.
//
// All trading, market data, and account operations have been migrated
// to the modular Orchestrator + Protocol Module architecture.
// See `crates/modules/hyperliquid/` and `crates/core/src/orchestrator.rs`.

use alloy::primitives::Address;
use anyhow::{Context, Result};
use hypersdk::hypercore::{
    self as hypercore,
    types::Side,
    HttpClient,
};
use tracing::info;

use crate::config::AppConfig;
use crate::auth::AuthManager;

/// Minimal engine for DB sync operations.
pub struct Engine {
    pub client: HttpClient,
    pub address: Address,
    pub config: AppConfig,
}

impl Engine {
    /// Create from the active wallet profile.
    pub async fn from_active_profile() -> Result<Self> {
        let config = crate::workspace::load_config()?;
        let signer = AuthManager::get_active_signer()?;
        let address = signer.address();
        let testnet = config.modules.hyperliquid.config.network == "testnet";

        let client = if testnet {
            hypercore::testnet()
        } else {
            hypercore::mainnet()
        };

        Ok(Self { client, address, config })
    }

    // ═══════════════════════════════════════════════════════════════════
    //  DATA SYNC — API → LOCAL SQLITE CACHE
    // ═══════════════════════════════════════════════════════════════════

    /// Sync fills from the API into the local database.
    pub async fn sync_fills(&self, db: &crate::db::AtlasDb) -> Result<usize> {
        use crate::db::DbFill;

        info!("syncing fills from API");

        let api_fills = self.client.user_fills(self.address).await
            .context("Failed to fetch fills from API")?;

        let db_fills: Vec<DbFill> = api_fills.iter().map(|f| {
            let side = match f.side {
                Side::Bid => "Buy",
                Side::Ask => "Sell",
            };
            DbFill {
                protocol: "hyperliquid".to_string(),
                coin: f.coin.clone(),
                px: f.px.to_string(),
                sz: f.sz.to_string(),
                side: side.to_string(),
                time_ms: f.time as i64,
                fee: f.fee.to_string(),
                hash: f.hash.clone(),
                oid: f.oid as i64,
                closed_pnl: f.closed_pnl.to_string(),
            }
        }).collect();

        let inserted = db.insert_fills(&db_fills)?;
        info!(fetched = api_fills.len(), inserted, "fills sync complete");
        Ok(inserted)
    }

    /// Sync historical orders from the API into the local database.
    pub async fn sync_orders(&self, db: &crate::db::AtlasDb) -> Result<usize> {
        use crate::db::DbOrder;

        info!("syncing orders from API");

        let api_orders = self.client.historical_orders(self.address).await
            .context("Failed to fetch historical orders from API")?;

        let db_orders: Vec<DbOrder> = api_orders.iter().map(|o| {
            let side = match o.side {
                Side::Bid => "Buy",
                Side::Ask => "Sell",
            };
            let order_type = format!("{:?}", o.order_type);
            DbOrder {
                protocol: "hyperliquid".to_string(),
                coin: o.coin.clone(),
                side: side.to_string(),
                limit_px: o.limit_px.to_string(),
                sz: o.sz.to_string(),
                oid: o.oid as i64,
                timestamp_ms: o.timestamp as i64,
                status: "historical".to_string(),
                order_type,
            }
        }).collect();

        let inserted = db.insert_orders(&db_orders)?;
        info!(fetched = api_orders.len(), inserted, "orders sync complete");
        Ok(inserted)
    }

    /// Sync all data (fills + orders) from the API into the local database.
    pub async fn sync_all(&self, db: &crate::db::AtlasDb) -> Result<(usize, usize)> {
        let fills = self.sync_fills(db).await?;
        let orders = self.sync_orders(db).await?;
        Ok((fills, orders))
    }
}
// Builder fee constants for protocol revenue injection.
//
// ╔══════════════════════════════════════════════════════════════════╗
// ║  BUILDER FEE — MANDATORY REVENUE LAYER                        ║
// ║                                                                ║
// ║  Every order submitted through Atlas MUST include the builder  ║
// ║  parameter pointing to BUILDER_ADDRESS. This is the sole      ║
// ║  monetization mechanism of the protocol. Do NOT remove or      ║
// ║  bypass this. Any order path that skips builder fee injection  ║
// ║  is a critical bug.                                            ║
// ╚══════════════════════════════════════════════════════════════════╝

use serde::{Deserialize, Serialize};

/// The address that receives builder fees on Hyperliquid.
/// Replace with the actual revenue wallet before production deployment.
pub const BUILDER_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

/// Builder fee in basis points (10 bps = 0.1%).
pub const BUILDER_FEE_BPS: u16 = 10;

/// Builder fee payload injected into the exchange action JSON.
///
/// The Hyperliquid API accepts this as part of the order action:
/// ```json
/// {
///   "action": {
///     "type": "order",
///     "orders": [...],
///     "grouping": "na",
///     "builder": { "b": "0xADDRESS", "f": 10 }
///   }
/// }
/// ```
///
/// The builder field is NOT part of the signed data (excluded from the
/// RMP hash), so it can be injected after signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderFee {
    /// Builder address (hex with 0x prefix).
    pub b: String,
    /// Fee in basis points (e.g. 10 = 0.1%).
    pub f: u16,
}

impl Default for BuilderFee {
    fn default() -> Self {
        Self {
            b: BUILDER_ADDRESS.to_string(),
            f: BUILDER_FEE_BPS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_address_is_valid_hex() {
        assert!(BUILDER_ADDRESS.starts_with("0x"));
        assert_eq!(BUILDER_ADDRESS.len(), 42);
    }

    #[test]
    fn test_builder_fee_reasonable() {
        // Fee should be between 1 and 100 bps
        assert!(BUILDER_FEE_BPS >= 1);
        assert!(BUILDER_FEE_BPS <= 100);
    }

    #[test]
    fn test_builder_fee_default() {
        let fee = BuilderFee::default();
        assert_eq!(fee.b, BUILDER_ADDRESS);
        assert_eq!(fee.f, BUILDER_FEE_BPS);
    }

    #[test]
    fn test_builder_fee_serialization() {
        let fee = BuilderFee::default();
        let json = serde_json::to_string(&fee).unwrap();
        assert!(json.contains("\"b\""));
        assert!(json.contains("\"f\""));
    }
}
