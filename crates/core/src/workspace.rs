use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use atlas_types::config::AppConfig;
use tracing::info;

/// Dotfolder name under `$HOME`.
const DOTFOLDER: &str = ".atlas-os";

/// Required subdirectories inside the dotfolder.
const SUBDIRS: &[&str] = &["logs", "data", "keystore"];

/// Resolve the root path: `$HOME/.atlas-os/`.
pub fn root_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(DOTFOLDER))
}

/// Resolve a path relative to the dotfolder root.
pub fn resolve(relative: &str) -> Result<PathBuf> {
    Ok(root_dir()?.join(relative))
}

/// Ensure the full dotfolder structure exists, creating directories and
/// default files as needed. This is idempotent — safe to call on every launch.
///
/// ```text
/// $HOME/.atlas-os/
/// ├── config.toml
/// ├── logs/
/// ├── data/
/// └── keystore/
///     └── wallets.json  (created empty if missing)
/// ```
pub fn init_workspace() -> Result<()> {
    let root = root_dir()?;

    // Create root + subdirectories.
    for sub in SUBDIRS {
        let dir = root.join(sub);
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
            info!("created directory: {}", dir.display());
        }
    }

    // Seed config.toml with defaults if absent.
    let config_path = root.join("config.toml");
    if !config_path.exists() {
        let default_config = AppConfig::default();
        let toml_str = default_config
            .to_toml_string()
            .context("Failed to serialize default config")?;
        fs::write(&config_path, &toml_str)
            .with_context(|| format!("Failed to write {}", config_path.display()))?;
        info!("created default config: {}", config_path.display());
    }

    // Seed empty wallets.json if absent.
    let wallets_path = root.join("keystore/wallets.json");
    if !wallets_path.exists() {
        let empty_store = atlas_types::profile::WalletStore::default();
        let json = serde_json::to_string_pretty(&empty_store)
            .context("Failed to serialize empty wallet store")?;
        fs::write(&wallets_path, &json)
            .with_context(|| format!("Failed to write {}", wallets_path.display()))?;
        info!("created wallets store: {}", wallets_path.display());
    }

    info!("workspace initialized at {}", root.display());
    Ok(())
}

/// Load the config from disk. If the config is outdated (missing fields),
/// regenerate with defaults while preserving `active_profile`.
pub fn load_config() -> Result<AppConfig> {
    let config_path = root_dir()?.join("config.toml");
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;

    match AppConfig::from_toml_str(&raw) {
        Ok(config) => Ok(config),
        Err(_) => {
            // Config schema changed — try to preserve active_profile
            info!("config.toml outdated, migrating to new schema");
            let mut new_config = AppConfig::default();

            // Attempt to extract active_profile from old config
            if let Ok(old) = raw.parse::<toml::Table>() {
                if let Some(general) = old.get("general").and_then(|v| v.as_table()) {
                    if let Some(profile) = general.get("active_profile").and_then(|v| v.as_str()) {
                        new_config.general.active_profile = profile.to_string();
                    }
                }
                if let Some(network) = old.get("network").and_then(|v| v.as_table()) {
                    if let Some(testnet) = network.get("testnet").and_then(|v| v.as_bool()) {
                        new_config.network.testnet = testnet;
                    }
                }
            }

            // Write the migrated config
            save_config(&new_config)?;
            info!("config migrated successfully");
            Ok(new_config)
        }
    }
}

/// Write the config back to disk.
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = root_dir()?.join("config.toml");
    let toml_str = config
        .to_toml_string()
        .context("Failed to serialize config")?;
    fs::write(&config_path, &toml_str)
        .with_context(|| format!("Failed to write {}", config_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_dir_under_home() {
        let root = root_dir().unwrap();
        let home = dirs::home_dir().unwrap();
        assert_eq!(root, home.join(".atlas-os"));
    }

    #[test]
    fn test_resolve_relative_path() {
        let path = resolve("keystore/wallets.json").unwrap();
        assert!(path.ends_with("keystore/wallets.json"));
        assert!(path.starts_with(root_dir().unwrap()));
    }

    #[test]
    fn test_resolve_nested() {
        let path = resolve("logs/audit.log").unwrap();
        let root = root_dir().unwrap();
        assert_eq!(path, root.join("logs/audit.log"));
    }

    #[test]
    fn test_init_workspace_idempotent() {
        // Should not fail when called multiple times
        init_workspace().unwrap();
        init_workspace().unwrap();
    }

    #[test]
    fn test_load_and_save_config() {
        init_workspace().unwrap();
        let config = load_config().unwrap();
        // Save and reload should be stable
        save_config(&config).unwrap();
        let reloaded = load_config().unwrap();
        assert_eq!(reloaded.general.active_profile, config.general.active_profile);
        assert_eq!(reloaded.trading.mode, config.trading.mode);
    }

    #[test]
    fn test_subdirs_exist_after_init() {
        init_workspace().unwrap();
        let root = root_dir().unwrap();
        for sub in SUBDIRS {
            assert!(root.join(sub).is_dir(), "{sub} directory should exist");
        }
    }

    #[test]
    fn test_config_file_exists_after_init() {
        init_workspace().unwrap();
        let config_path = root_dir().unwrap().join("config.toml");
        assert!(config_path.is_file());
    }

    #[test]
    fn test_wallets_file_exists_after_init() {
        init_workspace().unwrap();
        let wallets = root_dir().unwrap().join("keystore/wallets.json");
        assert!(wallets.is_file());
    }
}
