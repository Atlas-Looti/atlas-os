use std::fs;

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{bail, Context, Result};
use keyring::Entry;
use tracing::info;

use atlas_types::profile::{WalletProfile, WalletStore};

/// Keyring service name — all Atlas private keys are stored under this.
const KEYRING_SERVICE: &str = "atlas_os";

/// Manages wallet profiles and their secrets.
///
/// Public metadata lives in `$HOME/.atlas-os/keystore/wallets.json`.
/// Private keys live ONLY in the OS keyring (never on disk).
pub struct AuthManager;

impl AuthManager {
    // ── Wallet Store I/O ────────────────────────────────────────────

    /// Load the wallet store from disk (public for CLI use).
    pub fn load_store_pub() -> Result<WalletStore> {
        Self::load_store()
    }

    /// Load the wallet store from disk.
    fn load_store() -> Result<WalletStore> {
        let path = crate::workspace::resolve("keystore/wallets.json")?;
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let store: WalletStore =
            serde_json::from_str(&raw).context("Failed to parse wallets.json")?;
        Ok(store)
    }

    /// Persist the wallet store to disk.
    fn save_store(store: &WalletStore) -> Result<()> {
        let path = crate::workspace::resolve("keystore/wallets.json")?;
        let json =
            serde_json::to_string_pretty(store).context("Failed to serialize wallet store")?;
        fs::write(&path, &json)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    // ── Keyring helpers ─────────────────────────────────────────────

    /// Store a hex-encoded private key in the OS keyring.
    fn store_key(profile_name: &str, hex_key: &str) -> Result<()> {
        let entry = Entry::new(KEYRING_SERVICE, profile_name)
            .context("Failed to create keyring entry")?;
        entry
            .set_password(hex_key)
            .context("Failed to store key in OS keyring")?;
        Ok(())
    }

    /// Retrieve a hex-encoded private key from the OS keyring.
    fn retrieve_key(profile_name: &str) -> Result<String> {
        let entry = Entry::new(KEYRING_SERVICE, profile_name)
            .context("Failed to access keyring entry")?;
        let key = entry
            .get_password()
            .with_context(|| format!("No keyring entry found for profile '{profile_name}'"))?;
        Ok(key)
    }

    // ── Public API ──────────────────────────────────────────────────

    /// Generate a brand-new random EVM wallet, store it, and print the
    /// private key **exactly once** so the user can back it up.
    pub fn create_new_wallet(name: &str) -> Result<()> {
        let mut store = Self::load_store()?;
        if store.exists(name) {
            bail!("Profile '{name}' already exists");
        }

        // Generate a random signer (private key).
        let signer = PrivateKeySigner::random();
        let address: Address = signer.address();
        let private_key_hex = hex::encode(signer.credential().to_bytes());

        // Store private key in OS keyring — NEVER on disk.
        Self::store_key(name, &private_key_hex)?;

        let address_str = format!("{address}");

        // Persist public metadata.
        store.add(WalletProfile {
            name: name.to_string(),
            address: address_str.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        });
        Self::save_store(&store)?;

        // ── PRINT PRIVATE KEY ONCE ──────────────────────────────────
        // After this, the key lives exclusively in the OS keyring.
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║  NEW WALLET CREATED                                        ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Profile : {:<49}║", name);
        println!("║  Address : {:<49}║", address_str);
        println!("║  Private : {:<49}║", private_key_hex);
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  ⚠  BACK UP YOUR PRIVATE KEY NOW. It will NOT be shown     ║");
        println!("║     again. It is stored ONLY in your OS secure keyring.     ║");
        println!("╚══════════════════════════════════════════════════════════════╝");

        info!(profile = name, %address, "wallet created and stored in keyring");
        Ok(())
    }

    /// Import an existing EVM private key (hex string, with or without 0x).
    pub fn import_wallet(name: &str, raw_hex: &str) -> Result<()> {
        let mut store = Self::load_store()?;
        if store.exists(name) {
            bail!("Profile '{name}' already exists");
        }

        let hex_clean = raw_hex.strip_prefix("0x").unwrap_or(raw_hex);

        // Parse into a signer to validate and derive the address.
        let signer: PrivateKeySigner = hex_clean
            .parse()
            .context("Invalid private key hex string")?;
        let address = signer.address();
        let address_str = format!("{address}");

        // Store in OS keyring.
        Self::store_key(name, hex_clean)?;

        // Persist public metadata.
        store.add(WalletProfile {
            name: name.to_string(),
            address: address_str.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        });
        Self::save_store(&store)?;

        println!("✓ Imported profile '{name}' → {address_str}");
        info!(profile = name, %address, "wallet imported");
        Ok(())
    }

    /// Switch the active profile in config.toml.
    pub fn switch_profile(name: &str) -> Result<()> {
        let store = Self::load_store()?;
        if !store.exists(name) {
            bail!("Profile '{name}' does not exist");
        }

        let mut config = crate::workspace::load_config()?;
        config.system.active_profile = name.to_string();
        crate::workspace::save_config(&config)?;

        println!("✓ Active profile switched to '{name}'");
        info!(profile = name, "profile switched");
        Ok(())
    }

    /// List all stored profiles.
    pub fn list_profiles() -> Result<()> {
        let store = Self::load_store()?;
        let config = crate::workspace::load_config()?;

        if store.wallets.is_empty() {
            println!("No profiles found. Create one with: atlas auth new <name>");
            return Ok(());
        }

        println!("┌──────────────────┬────────────────────────────────────────────┬──────────┐");
        println!("│ Profile          │ Address                                    │ Active   │");
        println!("├──────────────────┼────────────────────────────────────────────┼──────────┤");
        for w in &store.wallets {
            let active = if w.name == config.system.active_profile {
                "  ●"
            } else {
                ""
            };
            println!("│ {:<16} │ {:<42} │ {:<8} │", w.name, w.address, active);
        }
        println!("└──────────────────┴────────────────────────────────────────────┴──────────┘");

        Ok(())
    }

    /// Get a `PrivateKeySigner` for the currently active profile.
    pub fn get_active_signer() -> Result<PrivateKeySigner> {
        let config = crate::workspace::load_config()?;
        let profile_name = &config.system.active_profile;
        let hex_key = Self::retrieve_key(profile_name)?;
        let signer: PrivateKeySigner = hex_key
            .parse()
            .context("Corrupted key in keyring — invalid hex")?;
        Ok(signer)
    }
}
