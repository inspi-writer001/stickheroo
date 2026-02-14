use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::app::{CollectionState, SelectedCharacter, WalletState};
use crate::game_state::CharacterTemplate;
use crate::solana_bridge;

#[component]
pub fn PreviewPage() -> impl IntoView {
    let wallet_state = expect_context::<RwSignal<WalletState>>();
    let selected_char = expect_context::<RwSignal<SelectedCharacter>>();
    let collection_state = expect_context::<RwSignal<CollectionState>>();
    let tx_status = RwSignal::new(Option::<Result<String, String>>::None);
    let minting = RwSignal::new(false);

    let characters = CharacterTemplate::all();
    let idx = selected_char.get_untracked().index.unwrap_or(0);
    let character = characters[idx].clone();
    let char_for_mint = character.clone();

    let on_proceed = move |_| {
        let ws = wallet_state.get_untracked();
        if !ws.connected {
            tx_status.set(Some(Err("Connect wallet first".into())));
            return;
        }

        let ch = char_for_mint.clone();
        let pubkey_str = ws.pubkey.unwrap_or_default();
        minting.set(true);
        tx_status.set(None);

        spawn_local(async move {
            let result = async {
                let pubkey_bytes: [u8; 32] = crate::pages::character_select::bs58_decode(&pubkey_str)?
                    .try_into()
                    .map_err(|_| "Invalid pubkey length".to_string())?;
                let pubkey = solana_pubkey::Pubkey::new_from_array(pubkey_bytes);
                let collection = collection_state
                    .get_untracked()
                    .pubkey
                    .ok_or_else(|| "No collection — mint a character first".to_string())?;

                let bundle = mojo_rust_sdk::world::World::build_character_tx(
                    &collection,
                    pubkey,
                    pubkey,
                    &ch.name,
                    &format!("https://arweave.net/demo/{}", ch.name.to_lowercase()),
                )
                .map_err(|e| format!("Build tx: {}", e))?;

                let sig = solana_bridge::send_transaction_bundle(bundle).await?;
                Ok::<String, String>(sig)
            }
            .await;

            match result {
                Ok(sig) => {
                    tx_status.set(Some(Ok(format!(
                        "Minted! Tx: {}...{}",
                        &sig[..8],
                        &sig[sig.len().saturating_sub(8)..]
                    ))));
                }
                Err(e) => tx_status.set(Some(Err(e))),
            }
            minting.set(false);
        });
    };

    let hue = idx * 60;

    view! {
        <div class="page page-enter">
            <h2 class="section-title">"CHARACTER PREVIEW"</h2>

            <div class="preview-layout">
                <div class="preview-image">
                    <svg width="180" height="240" viewBox="0 0 180 240">
                        // Character figure (larger version)
                        <circle cx="90" cy="50" r="25" fill="none"
                            stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                        <line x1="90" y1="75" x2="90" y2="150"
                            stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                        <line x1="90" y1="95" x2="55" y2="125"
                            stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                        <line x1="90" y1="95" x2="125" y2="125"
                            stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                        <line x1="90" y1="150" x2="60" y2="210"
                            stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                        <line x1="90" y1="150" x2="120" y2="210"
                            stroke={format!("hsl({}, 80%, 60%)", hue)} stroke-width="2"/>
                        // Name
                        <text x="90" y="230" text-anchor="middle" fill="var(--green-primary)"
                            font-family="var(--font-mono)" font-size="14">
                            {character.name.clone()}
                        </text>
                    </svg>
                </div>

                <div class="preview-details">
                    <div class="metadata-field">
                        <span class="metadata-label">"Name"</span>
                        <span class="metadata-value">{character.name.clone()}</span>
                    </div>
                    <div class="metadata-field">
                        <span class="metadata-label">"Description"</span>
                        <span class="metadata-value">{character.description.clone()}</span>
                    </div>
                    <div class="metadata-field">
                        <span class="metadata-label">"Collection"</span>
                        <span class="metadata-value">"Mojo Arena Characters"</span>
                    </div>
                    <div class="metadata-field">
                        <span class="metadata-label">"Traits"</span>
                        <span class="metadata-value">
                            {format!("HP: {} | ATK: {} | DEF: {}", character.hp, character.atk, character.def)}
                        </span>
                    </div>
                    <div class="metadata-field">
                        <span class="metadata-label">"Cost"</span>
                        <span class="cost-display">"~0.01 SOL"</span>
                    </div>

                    <button
                        class="btn"
                        on:click=on_proceed
                        disabled=move || minting.get()
                    >
                        {move || if minting.get() { "MINTING..." } else { "PROCEED" }}
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
        </div>
    }
}
