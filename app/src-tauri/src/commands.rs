//! Tauri commands — bridge between Leptos frontend and Rust core.
//!
//! Wallet lifecycle commands delegate to [`rustok_core::wallet::WalletService`]
//! (registered as a separate Tauri-managed `Arc<WalletService>` in
//! `lib.rs::run`'s setup callback) per Phase 2 C1-A redesign.
//! Non-wallet commands (provider, explorer, txguard analysis, proxy,
//! biometric storage helpers) remain on [`AppState`] / `AppHandle`.

use std::sync::{Arc, Mutex};

use alloy_primitives::Address;
use rustok_core::convert::{preview_to_dto, send_result_to_dto, verdict_to_dto};
use rustok_core::explorer::ExplorerClient;
use rustok_core::keyring::LocalKeyring;
use rustok_core::provider::MultiProvider;
use rustok_core::wallet::{WalletService, WalletServiceError};
use rustok_types::{
    AnalysisResponse, SendPreviewDto, SendResponseDto, TransactionHistoryDto, UnifiedBalance,
    WalletInfo, WalletInfoWithMnemonic,
};
use tauri::{Manager, State};
use zeroize::Zeroizing;

/// Shared application state for non-wallet domains.
///
/// Wallet lifecycle moved to `WalletService` (registered separately as
/// `Arc<WalletService>` in the Tauri builder's `setup` callback) per
/// Phase 2 C1-A redesign — `app_data_dir` is only available after the app
/// handle exists, so the service cannot be constructed at `manage` time.
pub struct AppState {
    /// NOTE: std::sync::Mutex — lock must never be held across .await points.
    /// Clone the provider before any await, then drop the guard immediately.
    pub provider: Mutex<MultiProvider>,
    pub explorer: ExplorerClient,
}

// ─── Pure helpers (testable without Tauri runtime) ──────────────────

/// Parse optional tx value string into U256.
fn parse_tx_value(value: Option<&str>) -> Result<alloy_primitives::U256, String> {
    use alloy_primitives::U256;
    match value {
        Some(v) if !v.is_empty() => v
            .parse::<U256>()
            .map_err(|e| format!("invalid value '{v}': {e}")),
        _ => Ok(U256::ZERO),
    }
}

/// Render a [`WalletServiceError`] as the `String` error type Tauri commands
/// emit. Sensitive context never crosses FFI per C2 — the service emits only
/// structured variant tags + minimal numeric fields.
fn format_wallet_err(e: WalletServiceError) -> String {
    e.to_string()
}

// ─── Tauri commands — non-wallet domain ─────────────────────────────

#[tauri::command]
pub async fn get_balance(
    address: String,
    state: State<'_, AppState>,
) -> Result<UnifiedBalance, String> {
    let addr = address
        .parse()
        .map_err(|e| format!("invalid address: {e}"))?;
    let provider = state
        .provider
        .lock()
        .map_err(|e| format!("state lock: {e}"))?
        .clone();
    let balance = provider.unified_balance(addr).await;
    Ok(balance.into())
}

#[tauri::command]
pub async fn analyze_transaction(
    to: String,
    data: Option<String>,
    value: Option<String>,
) -> Result<AnalysisResponse, String> {
    use alloy_primitives::Bytes;

    let to_addr = to.parse().map_err(|e| format!("invalid to address: {e}"))?;
    let calldata: Bytes = match data {
        Some(d) if !d.is_empty() => d.parse().map_err(|e| format!("invalid calldata: {e}"))?,
        _ => Bytes::new(),
    };
    let tx_value = parse_tx_value(value.as_deref())?;

    let parsed = txguard::parser::parse(to_addr, &calldata, tx_value)
        .map_err(|e| format!("parse error: {e}"))?;
    let engine = txguard::rules::RulesEngine::default();
    let verdict = engine.analyze(&parsed);

    Ok(verdict_to_dto(verdict))
}

// ─── Tauri commands — wallet lifecycle (delegates to WalletService) ─

#[tauri::command]
pub async fn has_wallet(wallet_service: State<'_, Arc<WalletService>>) -> Result<bool, String> {
    wallet_service.has_wallet().await.map_err(format_wallet_err)
}

#[tauri::command]
pub async fn is_wallet_unlocked(
    wallet_service: State<'_, Arc<WalletService>>,
) -> Result<bool, String> {
    Ok(wallet_service.is_unlocked().await)
}

#[tauri::command]
pub async fn unlock_wallet(
    password: String,
    wallet_service: State<'_, Arc<WalletService>>,
) -> Result<WalletInfo, String> {
    let pwd = Zeroizing::new(password);
    let id = wallet_service
        .unlock(pwd)
        .await
        .map_err(format_wallet_err)?;
    Ok(WalletInfo { address: id })
}

#[tauri::command]
pub async fn lock_wallet(wallet_service: State<'_, Arc<WalletService>>) -> Result<(), String> {
    wallet_service.lock().await;
    Ok(())
}

#[tauri::command]
pub async fn create_wallet(
    password: String,
    wallet_service: State<'_, Arc<WalletService>>,
) -> Result<WalletInfo, String> {
    let pwd = Zeroizing::new(password);
    let id = wallet_service
        .create_wallet(pwd)
        .await
        .map_err(format_wallet_err)?;
    Ok(WalletInfo { address: id })
}

/// Generate a fresh wallet *and* return the recovery mnemonic alongside the
/// address. Implemented as `create_wallet` + immediate `reveal_mnemonic_for_onboarding`,
/// preserving the prior desktop UX (mnemonic shown once on creation).
///
/// The mnemonic is **encrypted at rest** between these two internal calls
/// (per C1-A) — onboarding-mnemonic file is removed on the successful reveal,
/// matching the legacy "show once" semantic.
#[tauri::command]
pub async fn create_wallet_with_mnemonic(
    password: String,
    wallet_service: State<'_, Arc<WalletService>>,
) -> Result<WalletInfoWithMnemonic, String> {
    let pwd = Zeroizing::new(password);
    let id = wallet_service
        .create_wallet(pwd.clone())
        .await
        .map_err(format_wallet_err)?;
    let phrase = wallet_service
        .reveal_mnemonic_for_onboarding(&id, pwd)
        .await
        .map_err(format_wallet_err)?;
    Ok(WalletInfoWithMnemonic {
        address: id,
        mnemonic: phrase.to_string(),
    })
}

#[tauri::command]
pub async fn import_wallet_from_mnemonic(
    phrase: String,
    password: String,
    wallet_service: State<'_, Arc<WalletService>>,
) -> Result<WalletInfo, String> {
    let phrase_z = Zeroizing::new(phrase);
    let pwd = Zeroizing::new(password);
    let id = wallet_service
        .import_from_mnemonic(phrase_z, pwd)
        .await
        .map_err(format_wallet_err)?;
    Ok(WalletInfo { address: id })
}

/// Generate a random BIP39 recovery phrase without creating a wallet.
///
/// **Known C1 violation** — kept for the legacy desktop create-wallet wizard
/// which calls this then `import_wallet_from_mnemonic` separately. Mobile
/// bindings do NOT expose this path; mobile uses the
/// `create_wallet` + `reveal_mnemonic_for_onboarding` pair which keeps the
/// raw phrase off the FFI boundary by design. Removed in Phase 4 cleanup
/// once the desktop wizard migrates to the new pair.
#[tauri::command]
pub async fn generate_mnemonic_phrase() -> Result<String, String> {
    let phrase = LocalKeyring::random_mnemonic_phrase()
        .map_err(|e| format!("failed to generate mnemonic: {e}"))?;
    Ok(phrase.to_string())
}

#[tauri::command]
pub async fn get_wallet_qr_svg(
    wallet_service: State<'_, Arc<WalletService>>,
) -> Result<String, String> {
    wallet_service
        .current_qr_svg()
        .await
        .map_err(format_wallet_err)
}

#[tauri::command]
pub async fn get_current_address(
    wallet_service: State<'_, Arc<WalletService>>,
) -> Result<Option<String>, String> {
    Ok(wallet_service.current_address().await)
}

#[tauri::command]
pub async fn get_wallet_balance(
    wallet_service: State<'_, Arc<WalletService>>,
    state: State<'_, AppState>,
) -> Result<UnifiedBalance, String> {
    let addr_str = wallet_service
        .current_address()
        .await
        .ok_or_else(|| "wallet not unlocked".to_string())?;
    let addr: Address = addr_str
        .parse()
        .map_err(|e| format!("invalid address: {e}"))?;
    let provider = state
        .provider
        .lock()
        .map_err(|e| format!("state lock: {e}"))?
        .clone();
    let balance = provider.unified_balance(addr).await;
    Ok(balance.into())
}

#[tauri::command]
pub async fn preview_send(
    to: String,
    amount: String,
    wallet_service: State<'_, Arc<WalletService>>,
    state: State<'_, AppState>,
) -> Result<SendPreviewDto, String> {
    let from_str = wallet_service
        .current_address()
        .await
        .ok_or_else(|| "wallet not unlocked".to_string())?;
    let from: Address = from_str
        .parse()
        .map_err(|e| format!("invalid address: {e}"))?;
    let to_addr: Address = to.parse().map_err(|e| format!("invalid address: {e}"))?;
    let amount_wei = rustok_core::amount::parse_eth_amount(&amount).map_err(|e| e.to_string())?;

    let provider = state
        .provider
        .lock()
        .map_err(|e| format!("state lock: {e}"))?
        .clone();
    let preview = rustok_core::send::preview_send(&provider, from, to_addr, amount_wei)
        .await
        .map_err(|e| e.to_string())?;

    Ok(preview_to_dto(preview, to_addr, amount_wei))
}

#[tauri::command]
pub async fn send_eth(
    to: String,
    amount: String,
    wallet_service: State<'_, Arc<WalletService>>,
    state: State<'_, AppState>,
) -> Result<SendResponseDto, String> {
    let signer = wallet_service
        .current_signer()
        .await
        .ok_or_else(|| "wallet not unlocked".to_string())?;
    let from = signer.address();

    let to_addr: Address = to.parse().map_err(|e| format!("invalid address: {e}"))?;
    let amount_wei = rustok_core::amount::parse_eth_amount(&amount).map_err(|e| e.to_string())?;

    let provider = state
        .provider
        .lock()
        .map_err(|e| format!("state lock: {e}"))?
        .clone();
    let preview = rustok_core::send::preview_send(&provider, from, to_addr, amount_wei)
        .await
        .map_err(|e| e.to_string())?;

    let result =
        rustok_core::send::execute_send(&provider, signer, to_addr, amount_wei, &preview.route)
            .await
            .map_err(|e| e.to_string())?;

    Ok(send_result_to_dto(result))
}

#[tauri::command]
pub async fn get_transaction_history(
    wallet_service: State<'_, Arc<WalletService>>,
    state: State<'_, AppState>,
) -> Result<TransactionHistoryDto, String> {
    let addr_str = wallet_service
        .current_address()
        .await
        .ok_or_else(|| "wallet not unlocked".to_string())?;
    let addr: Address = addr_str
        .parse()
        .map_err(|e| format!("invalid address: {e}"))?;
    let chains = state
        .provider
        .lock()
        .map_err(|e| format!("state lock: {e}"))?
        .chains()
        .to_vec();
    let history = state.explorer.fetch_history(addr, &chains, 20).await;
    Ok(history)
}

// ─── Biometric unlock ─────────────────────────────────────────────
//
// Security model unchanged from prior implementation: password is stored
// directly in OS-native secure storage (Android Keystore / iOS Keychain via
// tauri-plugin-keystore on mobile; system keyring via the `keyring` crate on
// desktop). The biometric unlock command delegates the actual wallet unlock
// to `WalletService::unlock` once the password is retrieved.

/// Marker file indicating biometric unlock is enabled.
const BIOMETRIC_ENABLED_FILE: &str = "biometric.enabled";

#[tauri::command]
pub async fn is_biometric_enabled(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("no app data dir: {e}"))?;
    Ok(data_dir.join(BIOMETRIC_ENABLED_FILE).exists())
}

#[tauri::command]
pub async fn enable_biometric_unlock(
    password: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let password = Zeroizing::new(password);
    crate::biometric_storage::store_password(&app_handle, &password)?;

    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("no app data dir: {e}"))?;
    std::fs::create_dir_all(&data_dir).map_err(|e| format!("create dir: {e}"))?;
    let marker = data_dir.join(BIOMETRIC_ENABLED_FILE);
    std::fs::write(&marker, []).map_err(|e| format!("write marker: {e}"))?;

    Ok(())
}

#[tauri::command]
pub async fn disable_biometric_unlock(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::biometric_storage::remove_password(&app_handle)?;

    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("no app data dir: {e}"))?;

    let marker = data_dir.join(BIOMETRIC_ENABLED_FILE);
    if marker.exists() {
        let _ = std::fs::remove_file(&marker);
    }

    // Clean up legacy biometric.dat if it exists.
    let legacy = data_dir.join("biometric.dat");
    if legacy.exists() {
        let _ = std::fs::remove_file(&legacy);
    }
    Ok(())
}

#[tauri::command]
pub async fn biometric_unlock_wallet(
    app_handle: tauri::AppHandle,
    wallet_service: State<'_, Arc<WalletService>>,
) -> Result<WalletInfo, String> {
    let password = Zeroizing::new(crate::biometric_storage::retrieve_password(&app_handle)?);
    let id = wallet_service
        .unlock(password)
        .await
        .map_err(format_wallet_err)?;
    Ok(WalletInfo { address: id })
}

// ─── Proxy toggle ───────────────────────────────────────────────────

/// Read whether the Cloudflare Worker proxy is enabled.
#[tauri::command]
pub fn get_proxy_enabled(app_handle: tauri::AppHandle) -> bool {
    if let Ok(data_dir) = app_handle.path().app_data_dir() {
        data_dir.join("proxy.enabled").exists()
    } else {
        false
    }
}

/// Enable or disable the Cloudflare Worker proxy at runtime.
#[tauri::command]
pub async fn set_proxy_enabled(
    enabled: bool,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("no app data dir: {e}"))?;
    let marker = data_dir.join("proxy.enabled");

    if enabled {
        tokio::fs::write(&marker, "")
            .await
            .map_err(|e| format!("failed to write proxy marker: {e}"))?;
    } else {
        let _ = tokio::fs::remove_file(&marker).await;
    }

    let new_provider = if enabled {
        MultiProvider::proxy_chains()
    } else {
        MultiProvider::default_chains()
    };

    *state
        .provider
        .lock()
        .map_err(|e| format!("state lock: {e}"))? = new_provider;

    Ok(())
}

// ─── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_value_none_is_zero() {
        let v = parse_tx_value(None).unwrap();
        assert_eq!(v, alloy_primitives::U256::ZERO);
    }

    #[test]
    fn parse_value_empty_is_zero() {
        let v = parse_tx_value(Some("")).unwrap();
        assert_eq!(v, alloy_primitives::U256::ZERO);
    }

    #[test]
    fn parse_value_valid_decimal() {
        let v = parse_tx_value(Some("1000000000000000000")).unwrap();
        assert_eq!(
            v,
            alloy_primitives::U256::from(1_000_000_000_000_000_000u128)
        );
    }

    #[test]
    fn parse_value_invalid_returns_error() {
        assert!(parse_tx_value(Some("not_a_number")).is_err());
        assert!(parse_tx_value(Some("1.5 ETH")).is_err());
        assert!(parse_tx_value(Some("-1")).is_err());
    }
}
