mod app;
mod components;
mod game_state;
mod pages;
mod solana_bridge;
pub mod svg_metadata;
mod wallet;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
