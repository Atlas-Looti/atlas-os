use anyhow::Result;
use atlas_core::output::{render, OutputFormat};
use atlas_core::output::{BalanceRow, PositionRow, StatusOutput};

/// `atlas status` — fast textual summary, no TUI.
pub async fn run(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;

    // Determine active modules
    let mut modules = Vec::new();
    if config.modules.hyperliquid.enabled {
        modules.push("hyperliquid".to_string());
    }
    if config.modules.zero_x.enabled {
        modules.push("zero_x".to_string());
    }

    let network = if config.modules.hyperliquid.config.network == "testnet" {
        "Testnet".to_string()
    } else {
        "Mainnet".to_string()
    };

    let orch_res = crate::factory::from_active_profile().await;
    match orch_res {
        Ok(orch) => {
            let perp = orch.perp(None).map_err(|e| anyhow::anyhow!("{e}"))?;
            let balances = perp.balances().await.map_err(|e| anyhow::anyhow!("{e}"))?;
            let positions = perp.positions().await.map_err(|e| anyhow::anyhow!("{e}"))?;
            let orders = perp
                .open_orders()
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let bal = balances.first();

            let balance_rows: Vec<BalanceRow> = balances
                .iter()
                .map(|b| BalanceRow {
                    asset: b.asset.clone(),
                    total: b.total.to_string(),
                    available: b.available.to_string(),
                    protocol: "hyperliquid".to_string(),
                })
                .collect();

            let pos_rows: Vec<PositionRow> = positions
                .iter()
                .map(|p| PositionRow {
                    coin: p.symbol.clone(),
                    side: if p.size > rust_decimal::Decimal::ZERO {
                        "long".into()
                    } else {
                        "short".into()
                    },
                    size: p.size.to_string(),
                    entry_price: p.entry_price.map(|e| e.to_string()),
                    mark_price: p.mark_price.map(|m| m.to_string()),
                    unrealized_pnl: p.unrealized_pnl.map(|u| u.to_string()),
                    liquidation_price: p.liquidation_price.map(|l| l.to_string()),
                    leverage: p.leverage,
                    margin_mode: p.margin_mode.clone(),
                    protocol: "hyperliquid".into(),
                })
                .collect();

            // Get address from auth manager
            let address = atlas_core::auth::AuthManager::get_active_signer()
                .map(|s| format!("{:#x}", alloy::signers::Signer::address(&s)))
                .unwrap_or_else(|_| "unknown".to_string());

            let output = StatusOutput {
                profile: config.system.active_profile.clone(),
                address,
                network,
                modules,
                balances: balance_rows,
                account_value: bal.map(|b| b.total.to_string()),
                margin_used: bal.map(|b| b.locked.to_string()),
                net_position: None,
                withdrawable: bal.map(|b| b.available.to_string()),
                positions: pos_rows,
                open_orders: orders.len(),
            };
            render(fmt, &output)?;
        }
        Err(e) => {
            let output = StatusOutput {
                profile: config.system.active_profile.clone(),
                address: "unknown".into(),
                network,
                modules,
                balances: vec![],
                account_value: None,
                margin_used: None,
                net_position: None,
                withdrawable: None,
                positions: vec![],
                open_orders: 0,
            };
            render(fmt, &output)?;
            if fmt == OutputFormat::Table {
                eprintln!("Warning: connection failed: {e:#}");
                eprintln!("Hint: Run `atlas profile list` to check profiles, or `atlas doctor` to diagnose.");
            }
        }
    }

    Ok(())
}
