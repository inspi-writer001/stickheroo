use leptos::prelude::*;

#[component]
pub fn StickFigure(
    #[prop(default = false)] is_enemy: bool,
    #[prop(into, optional)] animating: Option<Signal<bool>>,
) -> impl IntoView {
    let base_class = if is_enemy {
        "stick-figure enemy"
    } else {
        "stick-figure"
    };

    let class = move || {
        if let Some(anim) = animating {
            if anim.get() {
                return format!("{} attacking", base_class);
            }
        }
        base_class.to_string()
    };

    // Mirror enemy figure
    let transform = if is_enemy {
        "scale(-1, 1) translate(-80, 0)"
    } else {
        ""
    };

    view! {
        <svg class={class} viewBox="0 0 80 120">
            <g transform={transform} class="body">
                // Head
                <circle cx="40" cy="20" r="12"/>
                // Eyes
                <circle cx="36" cy="18" r="2" fill="currentColor"/>
                <circle cx="44" cy="18" r="2" fill="currentColor"/>
                // Body
                <line x1="40" y1="32" x2="40" y2="70"/>
                // Arms
                <line x1="40" y1="42" x2="20" y2="55"/>
                <line x1="40" y1="42" x2="60" y2="50"/>
                // Weapon (sword)
                <line x1="60" y1="50" x2="72" y2="38" stroke-width="3"/>
                // Legs
                <line x1="40" y1="70" x2="25" y2="100"/>
                <line x1="40" y1="70" x2="55" y2="100"/>
                // Feet
                <line x1="25" y1="100" x2="18" y2="100"/>
                <line x1="55" y1="100" x2="62" y2="100"/>
            </g>
        </svg>
    }
}
