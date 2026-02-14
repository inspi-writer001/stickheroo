use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::app::WalletState;
use crate::wallet;

#[component]
pub fn WalletButton() -> impl IntoView {
    let wallet_state = expect_context::<RwSignal<WalletState>>();
    let loading = RwSignal::new(false);

    let on_click = move |_| {
        let ws = wallet_state;
        loading.set(true);
        spawn_local(async move {
            let current = ws.get_untracked();
            if current.connected {
                let _ = wallet::disconnect_wallet().await;
                ws.set(WalletState {
                    connected: false,
                    pubkey: None,
                });
            } else {
                match wallet::connect_wallet().await {
                    Ok(pubkey) => {
                        ws.set(WalletState {
                            connected: true,
                            pubkey: Some(pubkey),
                        });
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Wallet connect error: {}", e).into(),
                        );
                    }
                }
            }
            loading.set(false);
        });
    };

    let button_text = move || {
        if loading.get() {
            return "...".to_string();
        }
        let ws = wallet_state.get();
        if ws.connected {
            if let Some(ref pk) = ws.pubkey {
                let truncated = format!("{}...{}", &pk[..4], &pk[pk.len() - 4..]);
                return truncated;
            }
            "Connected".to_string()
        } else {
            "Connect".to_string()
        }
    };

    let indicator_class = move || {
        if wallet_state.get().connected {
            "wallet-indicator connected"
        } else {
            "wallet-indicator"
        }
    };

    view! {
        <button class="btn btn-small wallet-btn" on:click=on_click disabled=move || loading.get()>
            <span class={indicator_class}></span>
            {button_text}
        </button>
    }
}
