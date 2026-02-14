use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;

use crate::app::WalletState;
use crate::wallet;

#[component]
pub fn StartPage() -> impl IntoView {
    let wallet_state = expect_context::<RwSignal<WalletState>>();
    let navigate = use_navigate();
    let loading = RwSignal::new(false);

    let on_start = move |_| {
        let nav = navigate.clone();
        loading.set(true);
        spawn_local(async move {
            let ws = wallet_state;
            let current = ws.get_untracked();
            if !current.connected {
                match wallet::connect_wallet().await {
                    Ok(pubkey) => {
                        ws.set(WalletState {
                            connected: true,
                            pubkey: Some(pubkey),
                        });
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Wallet error: {}", e).into(),
                        );
                        loading.set(false);
                        return;
                    }
                }
            }
            loading.set(false);
            nav("/select", Default::default());
        });
    };

    let status_text = move || {
        let ws = wallet_state.get();
        if ws.connected {
            format!("Wallet connected: {}", ws.pubkey.unwrap_or_default())
        } else {
            "Wallet not connected".to_string()
        }
    };

    let status_class = move || {
        if wallet_state.get().connected {
            "wallet-status connected"
        } else {
            "wallet-status"
        }
    };

    view! {
        <div class="page start-page page-enter">
            <div class="title-art">
                <svg width="200" height="120" viewBox="0 0 200 120">
                    // Shield outline
                    <path d="M100 10 L160 30 L160 70 Q160 100 100 115 Q40 100 40 70 L40 30 Z"
                        fill="none" stroke="var(--green-primary)" stroke-width="2"
                        style="filter: drop-shadow(0 0 6px var(--green-glow))"/>
                    // Sword
                    <line x1="100" y1="25" x2="100" y2="90" stroke="var(--green-primary)" stroke-width="3"
                        style="filter: drop-shadow(0 0 4px var(--green-glow))"/>
                    // Cross guard
                    <line x1="80" y1="45" x2="120" y2="45" stroke="var(--green-primary)" stroke-width="3"/>
                    // M letter
                    <text x="100" y="108" text-anchor="middle" fill="var(--green-dim)"
                        font-family="var(--font-mono)" font-size="10">"MOJO"</text>
                </svg>
            </div>
            <h1 class="game-title">"MOJO ARENA"</h1>
            <p class="game-subtitle">"SDK DEMO  •  SOLANA DEVNET"</p>
            <button
                class="btn"
                on:click=on_start
                disabled=move || loading.get()
            >
                {move || if loading.get() { "CONNECTING..." } else { "START GAME" }}
            </button>
            <p class={status_class}>{status_text}</p>
        </div>
    }
}
