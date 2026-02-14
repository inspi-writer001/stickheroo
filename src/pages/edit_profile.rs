use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::app::{MintedCharacters, WalletState};
use crate::game_state::CharacterTemplate;
use crate::solana_bridge;

#[component]
pub fn EditProfilePage() -> impl IntoView {
    let wallet_state = expect_context::<RwSignal<WalletState>>();
    let minted_chars = expect_context::<RwSignal<MintedCharacters>>();
    let tx_status = RwSignal::new(Option::<Result<String, String>>::None);
    let saving = RwSignal::new(false);
    let avatar_url = RwSignal::new(Option::<String>::None);

    // Handle file selection
    let on_avatar_click = move |_| {
        let document = web_sys::window().unwrap().document().unwrap();
        let input: web_sys::HtmlInputElement = document
            .create_element("input")
            .unwrap()
            .dyn_into()
            .unwrap();
        input.set_type("file");
        input.set_accept("image/*");

        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
            let input: web_sys::HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    match web_sys::Url::create_object_url_with_blob(&file) {
                        Ok(url) => avatar_url.set(Some(url)),
                        Err(_) => web_sys::console::error_1(&"Failed to create object URL".into()),
                    }
                }
            }
        });

        input.set_onchange(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        input.click();
    };

    let on_save_profile = move |_| {
        let ws = wallet_state.get_untracked();
        if !ws.connected {
            tx_status.set(Some(Err("Connect wallet first".into())));
            return;
        }

        let pubkey_str = ws.pubkey.unwrap_or_default();
        saving.set(true);
        tx_status.set(None);

        spawn_local(async move {
            let result = async {
                let pubkey_bytes: [u8; 32] =
                    crate::pages::character_select::bs58_decode(&pubkey_str)?
                        .try_into()
                        .map_err(|_| "Invalid pubkey length".to_string())?;
                let pubkey = solana_pubkey::Pubkey::new_from_array(pubkey_bytes);

                let bundle = mojo_rust_sdk::world::World::build_profile_picture_tx(
                    pubkey,
                    pubkey,
                    "Mojo Profile",
                    "https://arweave.net/demo/profile-metadata",
                )
                .map_err(|e| format!("Build tx: {}", e))?;

                let sig = solana_bridge::send_transaction_bundle(bundle).await?;
                Ok::<String, String>(sig)
            }
            .await;

            match result {
                Ok(sig) => {
                    tx_status.set(Some(Ok(format!(
                        "Profile saved! Tx: {}...{}",
                        &sig[..sig.len().min(8)],
                        &sig[sig.len().saturating_sub(8)..]
                    ))));
                }
                Err(e) => tx_status.set(Some(Err(e))),
            }
            saving.set(false);
        });
    };

    let all_characters = CharacterTemplate::all();

    view! {
        <div class="page page-enter">
            <h2 class="section-title">"EDIT PROFILE"</h2>

            <div class="profile-section">
                <div class="avatar-area" on:click=on_avatar_click>
                    {move || {
                        if let Some(url) = avatar_url.get() {
                            view! {
                                <img src={url} style="width: 100%; height: 100%; object-fit: cover; border-radius: 14px;" />
                            }.into_any()
                        } else {
                            view! {
                                <svg width="60" height="60" viewBox="0 0 60 60">
                                    <rect x="15" y="25" width="30" height="25" rx="3" fill="none"
                                        stroke="var(--text-dim)" stroke-width="2"/>
                                    <polyline points="22,35 30,27 38,35" fill="none"
                                        stroke="var(--text-dim)" stroke-width="2"/>
                                    <line x1="30" y1="27" x2="30" y2="45"
                                        stroke="var(--text-dim)" stroke-width="2"/>
                                </svg>
                            }.into_any()
                        }
                    }}
                </div>
                <p style="text-align: center; font-size: 0.8rem; color: var(--text-dim);">
                    "Click to upload profile picture"
                </p>

                <div class="panel panel-glow">
                    <h3 class="section-title" style="font-size: 0.9rem;">"OWNED CHARACTERS"</h3>
                    <div class="owned-characters">
                        {move || {
                            if !wallet_state.get().connected {
                                return view! {
                                    <p style="color: var(--text-dim); font-size: 0.8rem;">
                                        "Connect wallet to view characters"
                                    </p>
                                }.into_any();
                            }
                            let minted = minted_chars.get();
                            if minted.characters.is_empty() {
                                return view! {
                                    <p style="color: var(--text-dim); font-size: 0.8rem;">
                                        "No characters found — mint one from the Characters page!"
                                    </p>
                                }.into_any();
                            }
                            let cards = minted.characters.iter().map(|ch| {
                                let hue = ch.index * 60;
                                let name = ch.name.clone();
                                let template = all_characters.get(ch.index).cloned();
                                let stats = template.map(|t| format!("HP:{} ATK:{} DEF:{}", t.hp, t.atk, t.def))
                                    .unwrap_or_default();
                                view! {
                                    <div class="character-card" style="cursor: default;">
                                        <div class="character-avatar">
                                            <svg width="60" height="60" viewBox="0 0 60 60">
                                                <circle cx="30" cy="15" r="10" fill="none"
                                                    stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                                                <line x1="30" y1="25" x2="30" y2="42"
                                                    stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                                                <line x1="30" y1="30" x2="18" y2="38"
                                                    stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                                                <line x1="30" y1="30" x2="42" y2="38"
                                                    stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                                                <line x1="30" y1="42" x2="20" y2="55"
                                                    stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                                                <line x1="30" y1="42" x2="40" y2="55"
                                                    stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                                            </svg>
                                        </div>
                                        <div class="character-name">{name}</div>
                                        <div class="character-stats">{stats}</div>
                                    </div>
                                }
                            }).collect::<Vec<_>>();
                            view! { <>{cards}</> }.into_any()
                        }}
                    </div>
                </div>

                <button
                    class="btn"
                    on:click=on_save_profile
                    disabled=move || saving.get() || !wallet_state.get().connected
                >
                    {move || if saving.get() { "SAVING..." } else { "SAVE PROFILE" }}
                </button>

                {move || tx_status.get().map(|status| {
                    match status {
                        Ok(msg) => view! {
                            <div class="tx-status success">{msg}</div>
                        }.into_any(),
                        Err(msg) => view! {
                            <div class="tx-status error">{msg}</div>
                        }.into_any(),
                    }
                })}
            </div>
        </div>
    }
}
