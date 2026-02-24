use anyhow::Result;
use atlas_core::Engine;
use atlas_utils::output::{render, OutputFormat};

/// `atlas vault details <VAULT_ADDRESS>`
pub async fn vault_details(vault: &str, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = engine.get_vault_details(vault).await?;
    render(fmt, &output)?;
    Ok(())
}

/// `atlas vault deposits`
pub async fn vault_deposits(fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = engine.get_vault_deposits().await?;
    render(fmt, &output)?;
    Ok(())
}
