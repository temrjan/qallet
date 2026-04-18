use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use rustok_types::WalletInfo;
use serde::Serialize;

use crate::app::WalletState;
use crate::bridge::tauri_invoke;

#[derive(Serialize)]
struct EmptyArgs {}

#[derive(Serialize)]
struct ImportArgs {
    phrase: String,
    password: String,
}

/// Three-step wizard for creating a new wallet with BIP39 recovery phrase.
///
/// 1. Backup intro — three acknowledgement checkboxes.
/// 2. Show 12-word phrase — user writes it down.
/// 3. Password — derive wallet and persist via `import_wallet_from_mnemonic`.
#[component]
pub fn WalletPage() -> impl IntoView {
    let auth_state = use_context::<RwSignal<WalletState>>()
        .expect("WalletState context missing — must be provided in App");
    let navigate = use_navigate();

    let (step, set_step) = signal(1u8);
    let (ack_seen, set_ack_seen) = signal([false, false, false]);
    let (phrase, set_phrase) = signal(None::<String>);
    let (saved_check, set_saved_check) = signal(false);
    let (password, set_password) = signal(String::new());
    let (confirm, set_confirm) = signal(String::new());
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    // Step 1 → 2: generate the phrase on transition so navigating back to
    // step 1 and forward again produces a fresh phrase (the old one was
    // shown, so regenerate for safety if user is still in setup).
    let go_to_phrase = move |_| {
        set_error.set(None);
        set_loading.set(true);
        spawn_local(async move {
            match tauri_invoke::<_, String>("generate_mnemonic_phrase", &EmptyArgs {}).await {
                Ok(p) => {
                    set_phrase.set(Some(p));
                    set_step.set(2);
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    let go_to_password = move |_| {
        set_error.set(None);
        set_step.set(3);
    };

    let create_wallet = {
        let navigate = navigate.clone();
        move |_| {
            let pwd = password.get();
            let pwd_confirm = confirm.get();
            let ph = phrase.get().unwrap_or_default();

            if pwd.len() < 8 {
                set_error.set(Some("Password must be at least 8 characters".into()));
                return;
            }
            if pwd != pwd_confirm {
                set_error.set(Some("Passwords do not match".into()));
                return;
            }
            if ph.is_empty() {
                set_error.set(Some("Phrase missing — restart wizard".into()));
                return;
            }

            set_loading.set(true);
            set_error.set(None);

            let navigate = navigate.clone();
            spawn_local(async move {
                match tauri_invoke::<_, WalletInfo>(
                    "import_wallet_from_mnemonic",
                    &ImportArgs {
                        phrase: ph,
                        password: pwd,
                    },
                )
                .await
                {
                    Ok(_) => {
                        auth_state.set(WalletState::Unlocked);
                        navigate("/", Default::default());
                    }
                    Err(e) => set_error.set(Some(e)),
                }
                set_loading.set(false);
            });
        }
    };

    let back = move |_| {
        set_error.set(None);
        let s = step.get();
        if s > 1 {
            set_step.set(s - 1);
        }
    };

    view! {
        <div class="wallet-create">
            <div class="unlock-title">"Create Wallet"</div>

            {move || error.get().map(|e| view! {
                <p class="text-red-400 mt-2 text-center">{e}</p>
            })}

            // Step 1 — backup intro
            <div style:display=move || if step.get() == 1 { "" } else { "none" }>
                <p class="text-gray-300 mb-4 text-center">
                    "Before you get your recovery phrase, acknowledge each item:"
                </p>
                {[
                    "My recovery phrase is the ONLY way to restore this wallet. If I lose it, my funds are gone.",
                    "I will write the 12 words down on paper and store them somewhere safe — never in a screenshot, cloud, or chat.",
                    "Anyone who sees these words can steal all my funds. I will never share them — not even with support.",
                ].iter().enumerate().map(|(i, text)| {
                    let text = text.to_string();
                    view! {
                        <label class="ack-row">
                            <input
                                type="checkbox"
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_ack_seen.update(|arr| arr[i] = checked);
                                }
                            />
                            <span>{text}</span>
                        </label>
                    }
                }).collect_view()}

                <button
                    class="bg-indigo-600 px-4 py-3 rounded-xl w-full hover:bg-indigo-700 mt-4 disabled:bg-gray-700"
                    on:click=go_to_phrase
                    disabled=move || !ack_seen.get().iter().all(|b| *b) || loading.get()
                >
                    {move || if loading.get() { "Generating..." } else { "Show Recovery Phrase" }}
                </button>
            </div>

            // Step 2 — show 12 words
            <div style:display=move || if step.get() == 2 { "" } else { "none" }>
                <p class="text-gray-300 mb-3 text-center">
                    "Write these 12 words down in order."
                </p>
                <p class="text-yellow-400 text-sm mb-4 text-center">
                    "Never share. Never paste anywhere online."
                </p>

                <div class="mnemonic-grid">
                    {move || phrase.get().map(|p| {
                        p.split_whitespace()
                            .enumerate()
                            .map(|(i, word)| view! {
                                <div class="mnemonic-word">
                                    <span class="mnemonic-index">{i + 1}.</span>
                                    <span class="mnemonic-text">{word.to_string()}</span>
                                </div>
                            })
                            .collect_view()
                    })}
                </div>

                <label class="ack-row mt-4">
                    <input
                        type="checkbox"
                        on:change=move |ev| set_saved_check.set(event_target_checked(&ev))
                    />
                    <span>"I have written down all 12 words in order."</span>
                </label>

                <button
                    class="bg-indigo-600 px-4 py-3 rounded-xl w-full hover:bg-indigo-700 mt-4 disabled:bg-gray-700"
                    on:click=go_to_password
                    disabled=move || !saved_check.get()
                >
                    "Continue"
                </button>
                <button
                    class="text-gray-400 text-sm w-full text-center mt-2"
                    on:click=back
                >
                    "← Back"
                </button>
            </div>

            // Step 3 — password
            <div style:display=move || if step.get() == 3 { "" } else { "none" }>
                <p class="text-gray-300 mb-4 text-center">
                    "Set a password to unlock this wallet on this device."
                </p>
                <input
                    type="password"
                    class="border border-gray-600 rounded-xl p-2 w-full bg-gray-800 text-white mb-2"
                    placeholder="Password (min 8 characters)"
                    on:input:target=move |ev| set_password.set(ev.target().value())
                />
                <input
                    type="password"
                    class="border border-gray-600 rounded-xl p-2 w-full bg-gray-800 text-white"
                    placeholder="Confirm password"
                    on:input:target=move |ev| set_confirm.set(ev.target().value())
                />
                <button
                    class="mt-4 bg-indigo-600 px-4 py-3 rounded-xl w-full hover:bg-indigo-700 disabled:bg-gray-700"
                    on:click=create_wallet
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Creating..." } else { "Create Wallet" }}
                </button>
                <button
                    class="text-gray-400 text-sm w-full text-center mt-2"
                    on:click=back
                >
                    "← Back"
                </button>
            </div>

            // Footer — existing wallet? Unlock.
            <p class="text-gray-400 text-sm mt-6 text-center">
                "Already have a wallet? "
                <a href="/unlock" class="text-blue-400">"Unlock"</a>
            </p>
        </div>
    }
}

fn event_target_checked(ev: &leptos::ev::Event) -> bool {
    use web_sys::wasm_bindgen::JsCast;
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|el| el.checked())
        .unwrap_or(false)
}
