//! Mobile FFI bindings for rustok-core via uniffi.
//!
//! Exposes a minimal subset of rustok-core's wallet API to mobile clients
//! (React Native through uniffi-bindgen-react-native). Not used by Tauri
//! desktop builds.

uniffi::setup_scaffolding!();

/// Errors returned across the mobile FFI boundary.
///
/// Designed per `docs/PHASE-2-CONSTRAINTS.md` C2/C3:
/// - Structured variants — no opaque `message: String` payload that could
///   carry sensitive context (entropy fragments, partial keys, paths).
/// - Per-domain taxonomy — each domain owns its `*Kind` sub-enum, so the
///   top-level enum stays small (fixes C3 enum-scaling concern).
/// - No sensitive context crosses FFI; details should be logged Rust-side
///   via `tracing::error!` (instrumentation added per command in commits 3-9).
///
/// Sub-enums marked `Reserved` are taxonomy slots populated when their
/// domain is wired through FFI in subsequent Phase 2 commits.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BindingsError {
    /// Wallet / keyring lifecycle errors (mnemonic, encryption, unlock).
    /// Variants populated by wallet service in commit 3.
    #[error("wallet error: {kind}")]
    Wallet {
        /// Specific wallet error variant.
        kind: WalletErrorKind,
    },

    /// Send / transaction errors. Variants populated by send service in commit 4.
    #[error("send error: {kind}")]
    Send {
        /// Specific send error variant.
        kind: SendErrorKind,
    },

    /// RPC / chain communication errors. Variants populated when chain ops
    /// cross FFI (commits 4-5).
    #[error("rpc error: {kind}")]
    Rpc {
        /// Specific RPC error variant.
        kind: RpcErrorKind,
    },

    /// txguard analysis errors. Variants populated when txguard exposed via
    /// FFI (commits 4 / 9).
    #[error("txguard error: {kind}")]
    TxGuard {
        /// Specific txguard error variant.
        kind: TxGuardErrorKind,
    },

    /// FFI encoding / serialization errors (hex parse, address parse, U256).
    /// Variants populated when FFI mirror types added (commit 9).
    #[error("encoding error: {kind}")]
    Encoding {
        /// Specific encoding error variant.
        kind: EncodingErrorKind,
    },

    /// Swap module errors (0x API, route execution, txguard verdict).
    /// Variants populated by swap module in commit 8.
    #[error("swap error: {kind}")]
    Swap {
        /// Specific swap error variant.
        kind: SwapErrorKind,
    },

    /// Internal / unexpected errors. Sensitive context never crosses FFI;
    /// see Rust logs (`tracing::error!`) for details.
    #[error("internal: see Rust logs")]
    Internal,
}

/// Wallet / keyring error variants.
///
/// `thiserror::Error` derive enforces explicit `Display` per variant — prevents
/// the parent's `#[error("wallet error: {kind}")]` from leaking inner field
/// values through Debug formatting if future variants gain payload fields.
#[derive(Debug, thiserror::Error, uniffi::Enum)]
pub enum WalletErrorKind {
    /// Failed to generate a BIP-39 mnemonic phrase (entropy source failure or
    /// BIP-39 derivation problem).
    #[error("mnemonic generation failed")]
    MnemonicGeneration,
}

/// Send / transaction error variants. Reserved — populated in commit 4.
#[derive(Debug, thiserror::Error, uniffi::Enum)]
pub enum SendErrorKind {
    /// Taxonomy slot reserved; populated when send service lands.
    #[error("reserved")]
    Reserved,
}

/// RPC / chain communication error variants. Reserved — populated in commits 4-5.
#[derive(Debug, thiserror::Error, uniffi::Enum)]
pub enum RpcErrorKind {
    /// Taxonomy slot reserved; populated when chain ops cross FFI.
    #[error("reserved")]
    Reserved,
}

/// txguard analysis error variants. Reserved — populated in commits 4 / 9.
#[derive(Debug, thiserror::Error, uniffi::Enum)]
pub enum TxGuardErrorKind {
    /// Taxonomy slot reserved; populated when txguard exposed via FFI.
    #[error("reserved")]
    Reserved,
}

/// FFI encoding / serialization error variants. Reserved — populated in commit 9.
#[derive(Debug, thiserror::Error, uniffi::Enum)]
pub enum EncodingErrorKind {
    /// Taxonomy slot reserved; populated when FFI mirror types added.
    #[error("reserved")]
    Reserved,
}

/// Swap module error variants. Reserved — populated in commit 8.
#[derive(Debug, thiserror::Error, uniffi::Enum)]
pub enum SwapErrorKind {
    /// Taxonomy slot reserved; populated when swap module lands.
    #[error("reserved")]
    Reserved,
}

/// Generate a fresh 12-word BIP-39 mnemonic phrase.
///
/// Suitable for displaying once during onboarding. Underlying core function
/// uses cryptographically secure randomness.
///
/// # Errors
///
/// Returns [`BindingsError::Wallet`] with
/// [`WalletErrorKind::MnemonicGeneration`] if the underlying entropy source
/// fails or BIP-39 derivation cannot complete. Underlying error details are
/// dropped at the FFI boundary by design (per C2); future commits will add
/// `tracing::error!` instrumentation Rust-side for diagnostics.
#[uniffi::export]
pub fn generate_mnemonic() -> Result<String, BindingsError> {
    rustok_core::keyring::LocalKeyring::random_mnemonic_phrase()
        .map(|phrase| phrase.to_string())
        .map_err(|_| BindingsError::Wallet {
            kind: WalletErrorKind::MnemonicGeneration,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_12_word_mnemonic() {
        let phrase = generate_mnemonic().expect("mnemonic generation should succeed");
        assert_eq!(phrase.split_whitespace().count(), 12);
    }
}
