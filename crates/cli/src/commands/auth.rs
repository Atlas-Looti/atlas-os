use anyhow::Result;
use atlas_core::AuthManager;

/// `atlas auth new <name>`
pub fn new_wallet(name: &str) -> Result<()> {
    AuthManager::create_new_wallet(name)
}

/// `atlas auth import <name>` — prompts for the private key on stdin.
pub fn import_wallet(name: &str) -> Result<()> {
    println!("Enter private key (hex, with or without 0x prefix):");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to read input: {e}"))?;
    let key = input.trim();

    if key.is_empty() {
        anyhow::bail!("No key provided");
    }

    AuthManager::import_wallet(name, key)
}

/// `atlas auth switch <name>`
pub fn switch_profile(name: &str) -> Result<()> {
    AuthManager::switch_profile(name)
}

/// `atlas auth list`
pub fn list_profiles() -> Result<()> {
    AuthManager::list_profiles()
}
