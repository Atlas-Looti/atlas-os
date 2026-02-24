use anyhow::Result;
use atlas_core::Engine;
use atlas_utils::output::{render, OutputFormat};

/// `atlas sub list`
pub async fn sub_list(fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = engine.get_subaccounts().await?;
    render(fmt, &output)?;
    Ok(())
}

/// `atlas agent approve <ADDRESS> [--name NAME]`
pub async fn agent_approve(address: &str, name: Option<&str>, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = engine.approve_agent(address, name).await?;
    render(fmt, &output)?;
    Ok(())
}
