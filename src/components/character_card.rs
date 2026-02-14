use leptos::prelude::*;
use leptos::ev::MouseEvent;

use crate::game_state::CharacterTemplate;

#[component]
pub fn CharacterCard(
    character: CharacterTemplate,
    index: usize,
    #[prop(into)] selected: Signal<bool>,
    on_click: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    let card_class = move || {
        if selected.get() {
            "character-card selected"
        } else {
            "character-card"
        }
    };

    // Simple SVG character silhouette that varies by index
    let hue = index * 60;
    let svg_color = format!("hsl({}, 80%, 60%)", hue);

    view! {
        <div class={card_class} on:click=on_click>
            <div class="character-avatar">
                <svg width="60" height="60" viewBox="0 0 60 60">
                    // Head
                    <circle cx="30" cy="15" r="10" fill="none" stroke={svg_color.clone()} stroke-width="2"/>
                    // Body
                    <line x1="30" y1="25" x2="30" y2="42" stroke={svg_color.clone()} stroke-width="2"/>
                    // Arms
                    <line x1="30" y1="30" x2="18" y2="38" stroke={svg_color.clone()} stroke-width="2"/>
                    <line x1="30" y1="30" x2="42" y2="38" stroke={svg_color.clone()} stroke-width="2"/>
                    // Legs
                    <line x1="30" y1="42" x2="20" y2="55" stroke={svg_color.clone()} stroke-width="2"/>
                    <line x1="30" y1="42" x2="40" y2="55" stroke={svg_color} stroke-width="2"/>
                </svg>
            </div>
            <div class="character-name">{character.name.clone()}</div>
            <div class="character-stats">
                {format!("HP:{} ATK:{} DEF:{}", character.hp, character.atk, character.def)}
            </div>
            <div style="font-size: 0.65rem; color: var(--text-dim); margin-top: 0.3rem;">
                {character.description.clone()}
            </div>
        </div>
    }
}
