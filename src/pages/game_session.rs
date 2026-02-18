use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::app::{MintedCharacters, SelectedCharacter, WalletState};
use crate::components::health_bar::HealthBar;
use crate::components::stick_figure::StickFigure;
use crate::game_state::{BattleResult, BattleState, CharacterTemplate, LogKind, PlayerState, Turn};
use crate::solana_bridge;

#[component]
pub fn GameSessionPage() -> impl IntoView {
    let selected_char = expect_context::<RwSignal<SelectedCharacter>>();
    let wallet_state = expect_context::<RwSignal<WalletState>>();
    let minted_chars = expect_context::<RwSignal<MintedCharacters>>();

    // Gate: must have wallet + must have minted the selected character
    let gate_msg = Signal::derive(move || {
        let ws = wallet_state.get();
        if !ws.connected {
            return Some("Connect your wallet to battle!");
        }
        let idx = selected_char.get().index.unwrap_or(usize::MAX);
        let minted = minted_chars.get();
        if !minted.characters.iter().any(|c| c.index == idx) {
            return Some("Mint this character first before battling!");
        }
        None
    });

    let characters = CharacterTemplate::all();
    let idx = selected_char.get_untracked().index.unwrap_or(0);
    let character = characters[idx].clone();

    let battle = RwSignal::new(BattleState::new(&character));
    let player_animating = RwSignal::new(false);
    let enemy_animating = RwSignal::new(false);
    let defending = RwSignal::new(false);
    let player_hit = RwSignal::new(false);
    let enemy_hit = RwSignal::new(false);
    let state_created = RwSignal::new(false);
    let chain_status = RwSignal::new(Option::<String>::None);

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

    // Save state to chain (create or write)
    let save_state_to_chain = move |battle_snap: BattleState, is_create: bool| {
        let ws = wallet_state.get_untracked();
        if !ws.connected {
            return;
        }
        // Only save if character is actually minted
        let idx = selected_char.get_untracked().index.unwrap_or(usize::MAX);
        if !minted_chars.get_untracked().characters.iter().any(|c| c.index == idx) {
            return;
        }
        let pubkey_str = ws.pubkey.unwrap_or_default();
        spawn_local(async move {
            let result = async {
                let pubkey_bytes: [u8; 32] = crate::pages::character_select::bs58_decode(&pubkey_str)?
                    .try_into()
                    .map_err(|_| "Invalid pubkey length".to_string())?;
                let pubkey = solana_pubkey::Pubkey::new_from_array(pubkey_bytes);

                let player_state = PlayerState::from_battle(&battle_snap);
                let state_bytes = player_state.serialize_state();

                let bundle = if is_create {
                    mojo_rust_sdk::world::World::build_create_state_tx(
                        pubkey,
                        "mojo_battle",
                        &state_bytes,
                    )
                } else {
                    mojo_rust_sdk::world::World::build_write_state_tx(
                        pubkey,
                        "mojo_battle",
                        &state_bytes,
                    )
                }
                .map_err(|e| format!("Build state tx: {}", e))?;

                let sig = solana_bridge::send_transaction_bundle(bundle).await?;
                Ok::<String, String>(sig)
            }
            .await;

            match result {
                Ok(sig) => {
                    if is_create {
                        state_created.set(true);
                    }
                    chain_status.set(Some(format!(
                        "State saved! Tx: {}...{}",
                        &sig[..sig.len().min(8)],
                        &sig[sig.len().saturating_sub(8)..]
                    )));
                }
                Err(e) => {
                    chain_status.set(Some(format!("Chain error: {}", e)));
                }
            }
        });
    };

    // Create initial state on battle start
    {
        let battle_snap = battle.get_untracked();
        let is_create = !state_created.get_untracked();
        save_state_to_chain(battle_snap, is_create);
    }

    let do_enemy_turn = move || {
        let is_def = defending.get_untracked();
        spawn_local(async move {
            // Delay before enemy acts
            gloo_timers::future::TimeoutFuture::new(600).await;
            enemy_animating.set(true);
            gloo_timers::future::TimeoutFuture::new(600).await;
            enemy_animating.set(false);

            battle.update(|b| {
                b.enemy_attack(is_def);
            });

            // Player hit flash
            player_hit.set(true);
            gloo_timers::future::TimeoutFuture::new(300).await;
            player_hit.set(false);

            defending.set(false);

            // If battle ended, save final state
            let snap = battle.get_untracked();
            if snap.result.is_some() {
                save_state_to_chain(snap, false);
            }
        });
    };

    let on_attack = move |_| {
        if !is_player_turn() {
            return;
        }
        player_animating.set(true);
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(600).await;
            player_animating.set(false);

            battle.update(|b| {
                b.player_attack();
            });

            // Enemy hit flash
            enemy_hit.set(true);
            gloo_timers::future::TimeoutFuture::new(300).await;
            enemy_hit.set(false);

            let snap = battle.get_untracked();
            if snap.result.is_some() {
                // Battle ended — save final state
                save_state_to_chain(snap, false);
            } else {
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
        let new_battle = BattleState::new(&chars[i]);
        battle.set(new_battle.clone());
        defending.set(false);
        chain_status.set(None);
        // On restart, write (not create) since state account already exists
        save_state_to_chain(new_battle, false);
    };

    view! {
        <div class="page page-enter">
            // Gate check — show message if wallet not connected or character not minted
            {move || gate_msg.get().map(|msg| view! {
                <div class="panel panel-glow" style="text-align:center;padding:3rem;display:flex;flex-direction:column;gap:1.5rem;">
                    <p style="color:var(--yellow);font-size:1.1rem;">{msg}</p>
                    <a href="/select"><button class="btn">"← Mint a Character"</button></a>
                </div>
            })}
            // Battle UI — only rendered when gate passes
            <div class="battle-arena" style:display=move || if gate_msg.get().is_some() { "none" } else { "flex" }>
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
                    <StickFigure
                        is_enemy=false
                        animating=Signal::derive(move || player_animating.get())
                        defending=Signal::derive(move || defending.get())
                        hit=Signal::derive(move || player_hit.get())
                    />
                    <StickFigure
                        is_enemy=true
                        animating=Signal::derive(move || enemy_animating.get())
                        hit=Signal::derive(move || enemy_hit.get())
                    />
                </div>

                <div class="battle-controls">
                    <button
                        class="btn"
                        on:click=on_attack
                        disabled=move || !is_player_turn()
                    >
                        "ATTACK"
                    </button>
                    <button
                        class="btn"
                        on:click=on_defend
                        disabled=move || !is_player_turn()
                    >
                        "DEFEND"
                    </button>
                    <a href="/select"><button class="btn btn-small">"MENU"</button></a>
                </div>

                <div class="score-display">
                    {move || format!("Score: {}", battle.get().score)}
                </div>

                {move || chain_status.get().map(|msg| {
                    view! { <div class="tx-status success" style="font-size: 0.7rem;">{msg}</div> }
                })}

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
