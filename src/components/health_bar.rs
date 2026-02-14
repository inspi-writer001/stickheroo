use leptos::prelude::*;

#[component]
pub fn HealthBar(
    #[prop(into)] current: Signal<i32>,
    #[prop(into)] max: Signal<i32>,
    #[prop(into)] label: Signal<String>,
) -> impl IntoView {
    let percent = move || {
        let m = max.get();
        if m <= 0 {
            return 0.0;
        }
        (current.get() as f64 / m as f64 * 100.0).clamp(0.0, 100.0)
    };

    let bar_class = move || {
        let p = percent();
        if p > 60.0 {
            "health-bar-fill high"
        } else if p > 25.0 {
            "health-bar-fill medium"
        } else {
            "health-bar-fill low"
        }
    };

    let width_style = move || format!("width: {}%", percent());

    view! {
        <div>
            <div style="font-size: 0.75rem; margin-bottom: 0.3rem; color: var(--text-dim);">
                {label}
            </div>
            <div class="health-bar-container">
                <div class={bar_class} style={width_style}></div>
                <div class="health-text">
                    {move || format!("{} / {}", current.get(), max.get())}
                </div>
            </div>
        </div>
    }
}
