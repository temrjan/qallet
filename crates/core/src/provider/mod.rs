//! Multi-chain RPC provider.
//!
//! Connects to multiple EVM chains and provides a unified interface
//! for querying balances, sending transactions, and fetching state.

mod chains;
mod multi;

pub use chains::{Chain, default_chains};
pub use multi::{GasFees, MultiProvider, ProviderError, UnifiedBalance, format_wei};
