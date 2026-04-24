use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use rustok_types::UnifiedBalance;
use serde::Serialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::app::WalletState;
use crate::bridge::tauri_invoke;

/// Interval between automatic balance refreshes while the tab is visible.
const AUTO_REFRESH_MS: u32 = 30_000;

/// Fetch balance silently (no loading spinner, no retry, no error surfacing).
/// Used by polling and visibility-change handlers.
fn silent_refresh(set_balance: WriteSignal<Option<UnifiedBalance>>) {
    spawn_local(async move {
        if let Ok(b) = tauri_invoke::<_, UnifiedBalance>("get_wallet_balance", &EmptyArgs {}).await
        {
            set_balance.set(Some(b));
        }
    });
}

fn document_hidden() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .map(|d| d.hidden())
        .unwrap_or(false)
}

#[derive(Serialize)]
struct EmptyArgs {}

#[component]
pub fn HomePage() -> impl IntoView {
    let state = use_context::<RwSignal<WalletState>>()
        .expect("WalletState context missing — must be provided in App");
    let navigate = use_navigate();

    let (balance, set_balance) = signal(None::<UnifiedBalance>);
    let (address, set_address) = signal(None::<String>);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    // Guard: redirect to the appropriate page when the wallet is not unlocked.
    // Runs whenever `state` changes.
    let nav_guard = navigate.clone();
    Effect::new(move |_| match state.get() {
        WalletState::Uninit => nav_guard("/welcome", Default::default()),
        WalletState::Locked => nav_guard("/unlock", Default::default()),
        WalletState::Loading | WalletState::Unlocked => {}
    });

    // Auto-refresh: polling every AUTO_REFRESH_MS when tab visible and wallet unlocked.
    // Skips when document.hidden() to avoid wasting RPC quota on background tabs.
    // forget() leaks the handle for the session — HomePage only unmounts on logout.
    gloo_timers::callback::Interval::new(AUTO_REFRESH_MS, move || {
        if state.get_untracked() != WalletState::Unlocked || document_hidden() {
            return;
        }
        silent_refresh(set_balance);
    })
    .forget();

    // Refetch immediately when app returns from background (visibilitychange event).
    let closure = Closure::wrap(Box::new(move || {
        if state.get_untracked() != WalletState::Unlocked || document_hidden() {
            return;
        }
        silent_refresh(set_balance);
    }) as Box<dyn FnMut()>);
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        let _ = doc
            .add_event_listener_with_callback("visibilitychange", closure.as_ref().unchecked_ref());
    }
    // Leak closure — HomePage lives for the session (only unmounts on logout/lock).
    closure.forget();

    // Balance fetch: runs when wallet becomes Unlocked.
    // Android TLS init can race the first RPC call — one retry after 800ms.
    Effect::new(move |_| {
        if state.get() != WalletState::Unlocked {
            return;
        }
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            if let Ok(Some(addr)) =
                tauri_invoke::<_, Option<String>>("get_current_address", &EmptyArgs {}).await
            {
                set_address.set(Some(addr));
            }

            match tauri_invoke::<_, UnifiedBalance>("get_wallet_balance", &EmptyArgs {}).await {
                Ok(b) if b.chains.is_empty() && !b.errors.is_empty() => {
                    gloo_timers::callback::Timeout::new(800, move || {
                        spawn_local(async move {
                            match tauri_invoke::<_, UnifiedBalance>(
                                "get_wallet_balance",
                                &EmptyArgs {},
                            )
                            .await
                            {
                                Ok(b2) => set_balance.set(Some(b2)),
                                Err(e) => set_error.set(Some(e)),
                            }
                            set_loading.set(false);
                        });
                    })
                    .forget();
                    return;
                }
                Ok(b) => set_balance.set(Some(b)),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    view! {
        <div>
            {move || match state.get() {
                WalletState::Loading => {
                    view! { <p class="text-gray-400">"Loading..."</p> }.into_any()
                }
                WalletState::Uninit | WalletState::Locked => {
                    view! { <p class="text-gray-400">"Redirecting..."</p> }.into_any()
                }
                WalletState::Unlocked => {
                    let addr = address.get();
                    let bal = balance.get();
                    let err = error.get();
                    let is_loading = loading.get();

                    view! {
                        <div>
                            {addr.map(|a| {
                                let short = if a.len() > 14 {
                                    format!("{}...{}", &a[..6], &a[a.len() - 4..])
                                } else {
                                    a
                                };
                                view! {
                                    <div class="home-address">
                                        <span>{short}</span>
                                    </div>
                                }
                            })}

                            {is_loading.then(|| view! {
                                <p class="text-gray-400">"Loading balance..."</p>
                            })}

                            {bal.map(|b| view! {
                                <div>
                                    <p class="home-balance">{b.approximate_total_formatted}</p>
                                    <ul class="chain-list list-none">
                                        {b.chains.into_iter().map(|c| view! {
                                            <li>{c.chain_name} ": " {c.formatted} " ETH"</li>
                                        }).collect_view()}
                                    </ul>
                                    {(!b.errors.is_empty()).then(|| view! {
                                        <p class="text-yellow-400 text-sm text-center">
                                            {format!("{} chain(s) failed", b.errors.len())}
                                        </p>
                                        <button
                                            class="text-blue-400 text-sm text-center w-full mt-1"
                                            style="background:none;border:none;cursor:pointer"
                                            on:click=move |_| {
                                                set_balance.set(None);
                                                set_error.set(None);
                                                set_loading.set(true);
                                                spawn_local(async move {
                                                    match tauri_invoke::<_, UnifiedBalance>(
                                                        "get_wallet_balance",
                                                        &EmptyArgs {},
                                                    ).await {
                                                        Ok(b) => set_balance.set(Some(b)),
                                                        Err(e) => set_error.set(Some(e)),
                                                    }
                                                    set_loading.set(false);
                                                });
                                            }
                                        >
                                            "Refresh"
                                        </button>
                                    })}
                                </div>
                            })}

                            {err.map(|e| view! { <p class="text-red-400 text-center">{e}</p> })}

                            <div class="action-row">
                                <a href="/send" class="action-btn">
                                    <span class="icon">"↑"</span>
                                    <span>"Send"</span>
                                </a>
                                <a href="/receive" class="action-btn">
                                    <span class="icon">"↓"</span>
                                    <span>"Receive"</span>
                                </a>
                                <a href="/scan" class="action-btn">
                                    <span class="icon">"⛨"</span>
                                    <span>"Scan"</span>
                                </a>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
