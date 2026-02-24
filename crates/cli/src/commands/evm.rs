//! `atlas evm` — EVM chain operations (balance, send).
//!
//! M.2: Financial Primitives — base layer L1 operations.

use anyhow::Result;
use atlas_utils::output::OutputFormat;

/// `atlas evm balance [--chain ethereum] [--token ETH]`
pub async fn balance(chain: &str, token: Option<&str>, fmt: OutputFormat) -> Result<()> {
    let token_display = token.unwrap_or("ETH (native)");

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "status": "not_implemented",
                "chain": chain,
                "token": token_display,
                "message": "EVM balance check coming in next release. Use atlas profile list to see addresses.",
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!("🔗 EVM Balance — {chain}");
            println!("   Token: {token_display}");
            println!("   ⚠️  EVM operations coming in next release.");
            println!("   Use `atlas profile list` to see your addresses.");
        }
    }
    Ok(())
}

/// `atlas evm send <to> <amount> [--chain ethereum] [--token ETH]`
pub async fn send(to: &str, amount: &str, chain: &str, token: Option<&str>, _fmt: OutputFormat) -> Result<()> {
    let token_display = token.unwrap_or("ETH");

    anyhow::bail!(
        "EVM send is not yet implemented. Chain: {chain}, To: {to}, Amount: {amount} {token_display}. Coming in next release."
    )
}
