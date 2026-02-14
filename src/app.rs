use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::wallet_button::WalletButton;
use crate::pages::{
    character_select::CharacterSelectPage, edit_profile::EditProfilePage,
    game_session::GameSessionPage, preview::PreviewPage, start::StartPage,
};

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
#[derive(Clone, Debug, Default)]
pub struct MintedCharacters {
    pub characters: Vec<MintedCharacterInfo>,
}

#[derive(Clone, Debug)]
pub struct MintedCharacterInfo {
    pub name: String,
    pub index: usize,
    pub tx_signature: String,
}

#[component]
pub fn App() -> impl IntoView {
    let wallet = RwSignal::new(WalletState::default());
    let selected_char = RwSignal::new(SelectedCharacter::default());
    let collection = RwSignal::new(CollectionState::default());
    let minted = RwSignal::new(MintedCharacters::default());

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
