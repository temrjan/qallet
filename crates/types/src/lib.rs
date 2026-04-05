//! Shared DTO types for communication between Rustok core (Tauri backend)
//! and the frontend (Leptos WASM).
//!
//! These types use only primitive Rust types (no `U256`, no `Address`) so the
//! frontend can depend on this crate without pulling in heavy crypto dependencies.
//! Both `Serialize` and `Deserialize` are derived: core serializes, frontend deserializes.

use serde::{Deserialize, Serialize};

/// Unified balance across all chains (DTO).
///
/// Maps from `rustok_core::provider::multi::UnifiedBalance`.
/// The `total` field is intentionally omitted (U256) — use `approximate_total_formatted`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedBalance {
    /// Approximate total formatted (e.g., "~2.5 ETH"). Not fungible across chains.
    pub approximate_total_formatted: String,
    /// Breakdown per chain.
    pub chains: Vec<ChainBalance>,
    /// Chains that failed to query (non-fatal).
    pub errors: Vec<String>,
}

/// Balance on a single chain (DTO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainBalance {
    /// Chain ID.
    pub chain_id: u64,
    /// Human-readable chain name.
    pub chain_name: String,
    /// Balance formatted with decimals (e.g., "1.5").
    pub formatted: String,
}

/// txguard analysis response (DTO).
///
/// Maps from `txguard::types::Verdict`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    /// Recommended action: "allow", "warn", or "block".
    pub action: String,
    /// Risk score from 0 (safe) to 100 (critical).
    pub risk_score: u8,
    /// Human-readable description of the transaction.
    pub description: String,
    /// Individual security findings.
    pub findings: Vec<FindingDto>,
}

/// A single security finding (DTO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingDto {
    /// Rule identifier (e.g., "unlimited_approval").
    pub rule: String,
    /// Severity: "info", "warning", "danger", or "forbidden".
    pub severity: String,
    /// Human-readable description.
    pub description: String,
}

/// Wallet info returned after creation or unlock (DTO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    /// Ethereum address (0x-prefixed, checksummed).
    pub address: String,
}
