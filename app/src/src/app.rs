use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::pages::{analyze, balance, receive, wallet};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <nav class="flex gap-4 p-4 border-b border-gray-700 text-sm">
                <A href="/" attr:class="hover:text-blue-400">"Balance"</A>
                <A href="/analyze" attr:class="hover:text-blue-400">"Analyze"</A>
                <A href="/receive" attr:class="hover:text-blue-400">"Receive"</A>
                <A href="/wallet" attr:class="hover:text-blue-400">"Wallet"</A>
            </nav>
            <main class="p-6">
                <Routes fallback=|| view! { <p>"Page not found"</p> }>
                    <Route path=path!("/") view=balance::BalancePage />
                    <Route path=path!("/analyze") view=analyze::AnalyzePage />
                    <Route path=path!("/receive") view=receive::ReceivePage />
                    <Route path=path!("/wallet") view=wallet::WalletPage />
                </Routes>
            </main>
        </Router>
    }
}
