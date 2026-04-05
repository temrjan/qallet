use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;

use crate::bridge::tauri_invoke;

#[derive(Serialize)]
struct EmptyArgs {}

#[component]
pub fn ReceivePage() -> impl IntoView {
    let (address, set_address) = signal(None::<String>);

    // Fetch current wallet address on mount.
    spawn_local(async move {
        if let Ok(Some(addr)) =
            tauri_invoke::<_, Option<String>>("get_current_address", &EmptyArgs {}).await
        {
            set_address.set(Some(addr));
        }
    });

    view! {
        <div>
            <h1 class="text-2xl font-bold mb-4">"Receive"</h1>
            {move || match address.get() {
                Some(addr) => view! {
                    <div class="text-center">
                        <p class="text-gray-400 mb-2">"Share this address to receive ETH:"</p>
                        <p class="font-mono text-lg break-all bg-gray-800 p-4 rounded">{addr}</p>
                    </div>
                }.into_any(),
                None => view! {
                    <p class="text-gray-400">"No wallet loaded. Create one first."</p>
                }.into_any(),
            }}
        </div>
    }
}
