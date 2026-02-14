use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::app::SelectedCharacter;
use crate::components::health_bar::HealthBar;
use crate::components::stick_figure::StickFigure;
use crate::game_state::{BattleResult, BattleState, CharacterTemplate, LogKind, Turn};

#[component]
pub fn GameSessionPage() -> impl IntoView {
    let selected_char = expect_context::<RwSignal<SelectedCharacter>>();

    let characters = CharacterTemplate::all();
    let idx = selected_char.get_untracked().index.unwrap_or(0);
    let character = characters[idx].clone();

    let battle = RwSignal::new(BattleState::new(&character));
    let player_animating = RwSignal::new(false);
    let enemy_animating = RwSignal::new(false);
    let defending = RwSignal::new(false);

    let player_hp = Signal::derive(move || battle.get().player_hp);
    let player_max_hp = Signal::derive(move || battle.get().player_max_hp);
    let enemy_hp = Signal::derive(move || battle.get().enemy_hp);
    let enemy_max_hp = Signal::derive(move || battle.get().enemy_max_hp);
    let player_label = Signal::derive({
        let name = character.name.clone();
        move || name.clone()
    });
    let enemy_label = Signal::derive(move || "Dark Knight".to_string());

    let is_player_turn = move || battle.get().turn == Turn::Player && battle.get().result.is_none();

    let do_enemy_turn = move || {
        let is_def = defending.get_untracked();
        spawn_local(async move {
            // Delay for enemy attack animation
            gloo_timers::future::TimeoutFuture::new(800).await;
            enemy_animating.set(true);
            gloo_timers::future::TimeoutFuture::new(400).await;
            enemy_animating.set(false);

            battle.update(|b| {
                b.enemy_attack(is_def);
            });
            defending.set(false);
        });
    };

    let on_attack = move |_| {
        if !is_player_turn() {
            return;
        }
        player_animating.set(true);
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(400).await;
            player_animating.set(false);

            battle.update(|b| {
                b.player_attack();
            });

            if battle.get_untracked().result.is_none() {
                do_enemy_turn();
            }
        });
    };

    let on_defend = move |_| {
        if !is_player_turn() {
            return;
        }
        defending.set(true);
        battle.update(|b| {
            b.player_defend();
        });
        do_enemy_turn();
    };

    let on_restart = move |_| {
        let chars = CharacterTemplate::all();
        let i = selected_char.get_untracked().index.unwrap_or(0);
        battle.set(BattleState::new(&chars[i]));
        defending.set(false);
    };

    view! {
        <div class="page page-enter">
            <div class="battle-arena">
                <div class="turn-indicator">
                    {move || {
                        let b = battle.get();
                        if b.result.is_some() {
                            "Battle Over".to_string()
                        } else if b.turn == Turn::Player {
                            "Your Turn".to_string()
                        } else {
                            "Enemy Turn...".to_string()
                        }
                    }}
                </div>

                <div class="battle-header">
                    <div class="combatant">
                        <span class="combatant-name">{player_label}</span>
                        <HealthBar current=player_hp max=player_max_hp label=Signal::derive(|| "HP".to_string()) />
                    </div>
                    <span class="vs-text">"VS"</span>
                    <div class="combatant">
                        <span class="combatant-name">{enemy_label}</span>
                        <HealthBar current=enemy_hp max=enemy_max_hp label=Signal::derive(|| "HP".to_string()) />
                    </div>
                </div>

                <div class="battle-stage">
                    <StickFigure is_enemy=false animating=Signal::derive(move || player_animating.get()) />
                    <StickFigure is_enemy=true animating=Signal::derive(move || enemy_animating.get()) />
                </div>

                <div class="battle-controls">
                    <button
                        class="btn"
                        on:click=on_attack
                        disabled=move || !is_player_turn()
                    >
                        "⚔ ATTACK"
                    </button>
                    <button
                        class="btn"
                        on:click=on_defend
                        disabled=move || !is_player_turn()
                    >
                        "🛡 DEFEND"
                    </button>
                    <a href="/select"><button class="btn btn-small">"MENU"</button></a>
                </div>

                <div class="score-display">
                    {move || format!("Score: {}", battle.get().score)}
                </div>

                <div class="battle-log">
                    {move || {
                        battle.get().log.iter().rev().map(|entry| {
                            let class = match entry.kind {
                                LogKind::Damage => "log-entry damage",
                                LogKind::Heal => "log-entry heal",
                                LogKind::Info => "log-entry info",
                            };
                            let msg = entry.message.clone();
                            view! { <div class={class}>{msg}</div> }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>

            // Battle result overlay
            {move || {
                let b = battle.get();
                b.result.map(|result| {
                    let (text, class) = match result {
                        BattleResult::Victory => ("VICTORY", "result-text victory"),
                        BattleResult::Defeat => ("DEFEAT", "result-text defeat"),
                    };
                    view! {
                        <div class="battle-result">
                            <div class={class}>{text}</div>
                            <div class="score-display">{format!("Final Score: {}", b.score)}</div>
                            <button class="btn" on:click=on_restart>"PLAY AGAIN"</button>
                            <a href="/select"><button class="btn">"BACK TO MENU"</button></a>
                        </div>
                    }
                })
            }}
        </div>
    }
}
