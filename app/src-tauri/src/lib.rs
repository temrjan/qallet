mod commands;

use commands::AppState;
use rustok_core::explorer::ExplorerClient;
use rustok_core::provider::MultiProvider;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(mobile)]
    let builder = builder.plugin(tauri_plugin_biometric::init());
    builder
        .setup(|_app| {
            #[cfg(target_os = "android")]
            {
                use tauri::Manager;
                _app.get_webview_window("main")
                    .expect("main window")
                    .with_webview(|webview| {
                        webview.jni_handle().exec(|env, context, _webview| {
                            use tauri::wry::prelude::JObject;
                            let loader = env
                                .call_method(
                                    context,
                                    "getClassLoader",
                                    "()Ljava/lang/ClassLoader;",
                                    &[],
                                )
                                .expect("getClassLoader");
                            rustls_platform_verifier::android::init_with_refs(
                                env.get_java_vm().expect("get_java_vm"),
                                env.new_global_ref(context).expect("global_ref context"),
                                env.new_global_ref(
                                    JObject::try_from(loader).expect("JObject from loader"),
                                )
                                .expect("global_ref loader"),
                            );
                        });
                    })?;
            }
            Ok(())
        })
        .manage(AppState {
            provider: if cfg!(debug_assertions) {
                MultiProvider::default_chains()
            } else {
                MultiProvider::mainnets_only()
            },
            explorer: ExplorerClient::new(),
            wallet: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_balance,
            commands::analyze_transaction,
            commands::create_wallet,
            commands::create_wallet_with_mnemonic,
            commands::import_wallet_from_mnemonic,
            commands::get_current_address,
            commands::get_wallet_qr_svg,
            commands::has_wallet,
            commands::is_wallet_unlocked,
            commands::unlock_wallet,
            commands::get_wallet_balance,
            commands::preview_send,
            commands::send_eth,
            commands::is_biometric_enabled,
            commands::enable_biometric_unlock,
            commands::disable_biometric_unlock,
            commands::biometric_unlock_wallet,
            commands::get_transaction_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
