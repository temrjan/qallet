//! Bridge between Leptos WASM and Tauri backend via invoke().

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Copy text to the system clipboard via `tauri-plugin-clipboard-manager`.
///
/// The plugin handles platform quirks (Android WebView requires a native
/// bridge; iOS WKWebView and desktop work too). Returns `true` on success.
///
/// Previously used `navigator.clipboard.writeText` via `js_sys::eval`, which
/// failed on Android release builds where the WebView refused the API
/// without a secure context.
pub async fn copy_to_clipboard(text: &str) -> bool {
    #[derive(Serialize)]
    struct WriteTextArgs<'a> {
        text: &'a str,
    }
    tauri_invoke::<_, ()>("plugin:clipboard-manager|write_text", &WriteTextArgs { text })
        .await
        .is_ok()
}

/// Type-safe invoke wrapper for calling tauri::command from WASM.
pub async fn tauri_invoke<A, R>(cmd: &str, args: &A) -> Result<R, String>
where
    A: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let args_js = serde_wasm_bindgen::to_value(args).map_err(|e| format!("serialize args: {e}"))?;

    let result = invoke(cmd, args_js)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize result: {e}"))
}
