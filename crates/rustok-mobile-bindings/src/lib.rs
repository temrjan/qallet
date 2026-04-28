//! Mobile FFI bindings for rustok-core via uniffi.
//!
//! Exposes a minimal subset of rustok-core's wallet API to mobile clients
//! (React Native through uniffi-bindgen-react-native). Not used by Tauri
//! desktop builds.

uniffi::setup_scaffolding!();

/// Errors returned across the mobile FFI boundary.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BindingsError {
    /// Failed to generate a BIP-39 mnemonic phrase.
    #[error("mnemonic generation failed: {message}")]
    MnemonicGeneration {
        /// Underlying core error description.
        message: String,
    },
}

/// Generate a fresh 12-word BIP-39 mnemonic phrase.
///
/// Suitable for displaying once during onboarding. Underlying core function
/// uses cryptographically secure randomness.
///
/// # Errors
///
/// Returns [`BindingsError::MnemonicGeneration`] if the underlying entropy
/// source fails or BIP-39 derivation cannot complete.
#[uniffi::export]
pub fn generate_mnemonic() -> Result<String, BindingsError> {
    rustok_core::keyring::LocalKeyring::random_mnemonic_phrase()
        .map(|phrase| phrase.to_string())
        .map_err(|e| BindingsError::MnemonicGeneration {
            message: e.to_string(),
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
