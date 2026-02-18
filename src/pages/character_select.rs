use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;

use crate::app::{CollectionState, MintedCharacterInfo, MintedCharacters, SelectedCharacter, WalletState};
use crate::components::character_card::CharacterCard;
use crate::game_state::CharacterTemplate;
use crate::solana_bridge;
use crate::svg_metadata;
use crate::wallet;

#[component]
pub fn CharacterSelectPage() -> impl IntoView {
    let wallet_state = expect_context::<RwSignal<WalletState>>();
    let selected_char = expect_context::<RwSignal<SelectedCharacter>>();
    let collection_state = expect_context::<RwSignal<CollectionState>>();
    let minted_chars = expect_context::<RwSignal<MintedCharacters>>();
    let characters = CharacterTemplate::all();
    let tx_status = RwSignal::new(Option::<Result<String, String>>::None);
    let minting = RwSignal::new(false);
    let navigate = use_navigate();

    let on_mint = move |_| {
        let sel = selected_char.get_untracked();
        let ws = wallet_state.get_untracked();
        let nav = navigate.clone();

        let Some(idx) = sel.index else {
            tx_status.set(Some(Err("Select a character first".into())));
            return;
        };

        if !ws.connected {
            tx_status.set(Some(Err("Connect wallet first".into())));
            return;
        }

        let character = CharacterTemplate::all()[idx].clone();
        let pubkey_str = ws.pubkey.unwrap_or_default();

        minting.set(true);
        tx_status.set(None);

        spawn_local(async move {
            let result = async {
                let pubkey_bytes: [u8; 32] = bs58_decode(&pubkey_str)?
                    .try_into()
                    .map_err(|_| "Invalid pubkey length".to_string())?;
                let pubkey = solana_pubkey::Pubkey::new_from_array(pubkey_bytes);

                // Step 1: Create collection if we don't have one yet
                let collection_pubkey = if let Some(col) = collection_state.get_untracked().pubkey {
                    col
                } else {
                    tx_status.set(Some(Ok("Creating collection... (approve in Phantom)".into())));

                    let col_meta = r#"{"name":"Mojo Arena Characters","description":"On-chain characters for the Mojo Arena demo","image":""}"#;
                    let collection_uri = wallet::upload_to_irys(col_meta, "application/json").await
                        .unwrap_or_else(|_| svg_metadata::build_collection_metadata_uri("Mojo Arena Characters"));
                    let col_bundle = mojo_rust_sdk::world::World::build_character_collection_tx(
                        pubkey,
                        "Mojo Arena Characters",
                        &collection_uri,
                    )
                    .map_err(|e| format!("Build collection tx: {}", e))?;

                    // Extract the collection pubkey from the ephemeral signer
                    use solana_signer::Signer;
                    let col_pubkey = col_bundle.signers[0].pubkey();

                    solana_bridge::send_transaction_bundle(col_bundle).await?;

                    // Store collection for future mints
                    collection_state.set(CollectionState {
                        pubkey: Some(col_pubkey),
                    });

                    tx_status.set(Some(Ok("Collection created! Now minting character...".into())));
                    col_pubkey
                };

                // Step 2: Generate PNG on canvas, upload image + metadata JSON to Arweave.
                // This mirrors the TS pattern: upload image → get URL → embed in JSON → upload JSON.
                // First upload prompts ONE Phantom approval for Irys devnet funding.
                // If upload fails we abort — the metadata_uri must always be a real HTTPS URL.
                tx_status.set(Some(Ok("Uploading image & metadata to Arweave...".into())));
                let char_uri = wallet::upload_character_metadata(
                    idx,
                    &character.name,
                    &character.description,
                    character.hp,
                    character.atk,
                    character.def,
                ).await
                .map_err(|e| format!("Arweave upload failed: {}", e))?;
                tx_status.set(Some(Ok("Image on Arweave! Minting character... (approve in Phantom)".into())));
                let bundle = mojo_rust_sdk::world::World::build_select_character_tx(
                    &collection_pubkey,
                    pubkey, // authority
                    pubkey, // buyer
                    pubkey, // payer
                    &character.name,
                    &char_uri,
                )
                .map_err(|e| format!("Build mint tx: {}", e))?;

                let sig = solana_bridge::send_transaction_bundle(bundle).await?;
                Ok::<String, String>(sig)
            }
            .await;

            match result {
                Ok(sig) => {
                    // Record minted character
                    minted_chars.update(|m| {
                        m.characters.push(MintedCharacterInfo {
                            name: character.name.clone(),
                            index: idx,
                            tx_signature: sig.clone(),
                        });
                    });
                    tx_status.set(Some(Ok(format!(
                        "Minted! Tx: {}...{}",
                        &sig[..sig.len().min(8)],
                        &sig[sig.len().saturating_sub(8)..]
                    ))));
                    nav("/preview", Default::default());
                }
                Err(e) => {
                    tx_status.set(Some(Err(e)));
                }
            }
            minting.set(false);
        });
    };

    view! {
        <div class="page page-enter">
            <h2 class="section-title">"SELECT YOUR CHARACTER"</h2>
            <div class="character-grid">
                {characters
                    .into_iter()
                    .enumerate()
                    .map(|(i, ch)| {
                        let is_selected = Signal::derive(move || {
                            selected_char.get().index == Some(i)
                        });
                        view! {
                            <CharacterCard
                                character=ch
                                index=i
                                selected=is_selected
                                on_click=move |_| {
                                    selected_char.set(SelectedCharacter { index: Some(i) });
                                }
                            />
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
            <div style="display: flex; gap: 1rem; align-items: center;">
                <button
                    class="btn"
                    on:click=on_mint
                    disabled=move || minting.get() || selected_char.get().index.is_none()
                >
                    {move || if minting.get() { "MINTING..." } else { "MINT CHARACTER" }}
                </button>
                <a href="/battle">
                    <button
                        class="btn"
                        disabled=move || selected_char.get().index.is_none()
                    >
                        "BATTLE →"
                    </button>
                </a>
            </div>
            {move || {
                let col = collection_state.get();
                col.pubkey.map(|_| view! {
                    <div class="tx-status success" style="font-size: 0.7rem;">
                        "Collection ready — mints go directly"
                    </div>
                })
            }}
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
    }
}

pub fn bs58_decode(s: &str) -> Result<Vec<u8>, String> {
    let alphabet = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut result = vec![0u8; 32];
    let mut scratch = vec![0u32; 44];

    for &c in s.as_bytes() {
        let mut val = alphabet
            .iter()
            .position(|&x| x == c)
            .ok_or_else(|| format!("Invalid base58 char: {}", c as char))?
            as u32;

        for digit in scratch.iter_mut().rev() {
            val += *digit * 58;
            *digit = val % 256;
            val /= 256;
        }
    }

    let leading_zeros = s.bytes().take_while(|&b| b == b'1').count();
    let start = scratch.iter().position(|&d| d != 0).unwrap_or(scratch.len());
    let decoded_len = leading_zeros + scratch.len() - start;

    if decoded_len != 32 {
        return Err(format!("Expected 32 bytes, got {}", decoded_len));
    }

    for i in 0..leading_zeros {
        result[i] = 0;
    }
    for (i, &d) in scratch[start..].iter().enumerate() {
        result[leading_zeros + i] = d as u8;
    }

    Ok(result)
}
