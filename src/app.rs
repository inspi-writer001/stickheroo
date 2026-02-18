use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use serde::{Deserialize, Serialize};

use crate::components::wallet_button::WalletButton;
use crate::pages::{
    character_select::CharacterSelectPage, edit_profile::EditProfilePage,
    game_session::GameSessionPage, preview::PreviewPage, start::StartPage,
};
use crate::wallet;

// Global wallet state
#[derive(Clone, Debug, Default)]
pub struct WalletState {
    pub connected: bool,
    pub pubkey: Option<String>,
}

// Selected character context
#[derive(Clone, Debug, Default)]
pub struct SelectedCharacter {
    pub index: Option<usize>,
}

// Collection context — stores a created collection pubkey for minting
#[derive(Clone, Debug, Default)]
pub struct CollectionState {
    pub pubkey: Option<solana_pubkey::Pubkey>,
}

// Track minted characters for the profile page
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MintedCharacters {
    pub characters: Vec<MintedCharacterInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MintedCharacterInfo {
    pub name: String,
    pub index: usize,
    pub tx_signature: String,
}

#[component]
pub fn App() -> impl IntoView {
    let wallet = RwSignal::new(WalletState::default());
    let selected_char = RwSignal::new(SelectedCharacter::default());

    // Load persisted collection pubkey from localStorage
    let initial_collection = {
        let mut cs = CollectionState::default();
        if let Some(stored) = wallet::load_from_storage("mojo_collection") {
            if stored.len() == 44 || stored.len() == 43 {
                // Try to decode base58 pubkey
                if let Ok(bytes) = crate::pages::character_select::bs58_decode(&stored) {
                    if let Ok(arr) = <[u8; 32]>::try_from(bytes) {
                        cs.pubkey = Some(solana_pubkey::Pubkey::new_from_array(arr));
                    }
                }
            }
        }
        cs
    };
    let collection = RwSignal::new(initial_collection);

    // Load persisted minted characters from localStorage
    let initial_minted = wallet::load_from_storage("mojo_minted_chars")
        .and_then(|s| serde_json::from_str::<MintedCharacters>(&s).ok())
        .unwrap_or_default();
    let minted = RwSignal::new(initial_minted);

    // Persist minted characters whenever they change
    Effect::new(move || {
        let m = minted.get();
        if let Ok(json) = serde_json::to_string(&m) {
            wallet::save_to_storage("mojo_minted_chars", &json);
        }
    });

    // Persist collection pubkey whenever it changes
    Effect::new(move || {
        let c = collection.get();
        if let Some(pk) = c.pubkey {
            wallet::save_to_storage("mojo_collection", &pk.to_string());
        }
    });

    provide_context(wallet);
    provide_context(selected_char);
    provide_context(collection);
    provide_context(minted);

    view! {
        <Router>
            <div class="app-container">
                <header class="header">
                    <span class="header-title">"MOJO SDK DEMO"</span>
                    <nav class="nav-links">
                        <a href="/">"Home"</a>
                        <a href="/select">"Characters"</a>
                        <a href="/battle">"Battle"</a>
                        <a href="/profile">"Profile"</a>
                    </nav>
                    <WalletButton />
                </header>
                <Routes fallback=|| view! { <p>"Page not found"</p> }>
                    <Route path=path!("/") view=StartPage />
                    <Route path=path!("/select") view=CharacterSelectPage />
                    <Route path=path!("/battle") view=GameSessionPage />
                    <Route path=path!("/profile") view=EditProfilePage />
                    <Route path=path!("/preview") view=PreviewPage />
                </Routes>
            </div>
        </Router>
    }
}
