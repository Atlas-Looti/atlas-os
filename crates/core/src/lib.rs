// ── From atlas-common ──
pub mod constants;
pub mod error;
pub mod traits;
pub mod types;

// ── From atlas-types ──
pub mod config;
pub mod output;
pub mod profile;

// ── From atlas-utils ──
pub mod fmt;
pub mod parse;
pub mod prompt;
pub mod risk;

// ── Core modules ──
pub mod auth;
pub mod backend;
pub mod db;
pub mod engine;
pub mod orchestrator;
pub mod workspace;

pub use auth::AuthManager;
pub use backend::BackendClient;
pub use engine::Engine;
pub use orchestrator::Orchestrator;
pub use workspace::init_workspace;
