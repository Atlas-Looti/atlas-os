use serde::{Deserialize, Serialize};

/// A named wallet profile stored in `keystore/wallets.json`.
///
/// The private key is NEVER stored here — only in the OS keyring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletProfile {
    /// Human-readable profile name (e.g. "bot-sniper").
    pub name: String,
    /// EVM public address (0x-prefixed, checksummed).
    pub address: String,
    /// ISO-8601 timestamp of when this profile was created.
    pub created_at: String,
}

/// The on-disk schema for `keystore/wallets.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletStore {
    pub wallets: Vec<WalletProfile>,
}

impl WalletStore {
    /// Find a profile by name.
    pub fn find(&self, name: &str) -> Option<&WalletProfile> {
        self.wallets.iter().find(|w| w.name == name)
    }

    /// Check if a profile name already exists.
    pub fn exists(&self, name: &str) -> bool {
        self.wallets.iter().any(|w| w.name == name)
    }

    /// Add a profile. Caller must ensure uniqueness.
    pub fn add(&mut self, profile: WalletProfile) {
        self.wallets.push(profile);
    }

    /// Remove a profile by name. Returns true if removed.
    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.wallets.len();
        self.wallets.retain(|w| w.name != name);
        self.wallets.len() < before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_profile(name: &str) -> WalletProfile {
        WalletProfile {
            name: name.to_string(),
            address: format!("0x{:0>40}", name),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_wallet_store_default_empty() {
        let store = WalletStore::default();
        assert!(store.wallets.is_empty());
    }

    #[test]
    fn test_add_and_find() {
        let mut store = WalletStore::default();
        store.add(make_profile("bot-1"));
        assert!(store.exists("bot-1"));
        assert!(!store.exists("bot-2"));
        let found = store.find("bot-1").unwrap();
        assert_eq!(found.name, "bot-1");
    }

    #[test]
    fn test_find_nonexistent() {
        let store = WalletStore::default();
        assert!(store.find("ghost").is_none());
    }

    #[test]
    fn test_remove() {
        let mut store = WalletStore::default();
        store.add(make_profile("bot-1"));
        store.add(make_profile("bot-2"));
        assert!(store.remove("bot-1"));
        assert!(!store.exists("bot-1"));
        assert!(store.exists("bot-2"));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut store = WalletStore::default();
        assert!(!store.remove("ghost"));
    }

    #[test]
    fn test_multiple_profiles() {
        let mut store = WalletStore::default();
        store.add(make_profile("a"));
        store.add(make_profile("b"));
        store.add(make_profile("c"));
        assert_eq!(store.wallets.len(), 3);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut store = WalletStore::default();
        store.add(make_profile("test"));
        let json = serde_json::to_string(&store).unwrap();
        let parsed: WalletStore = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.wallets.len(), 1);
        assert_eq!(parsed.wallets[0].name, "test");
    }
}
