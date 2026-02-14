use mojo_rust_sdk::mojo;
use serde::{Deserialize, Serialize};

// On-chain compatible state structs via mojo! macro
mojo! {
    pub struct PlayerState {
        pub health: [u8; 8],
        pub max_health: [u8; 8],
        pub attack: [u8; 8],
        pub defense: [u8; 8],
        pub score: [u8; 8],
    }
}

mojo! {
    pub struct EnemyState {
        pub health: [u8; 8],
        pub max_health: [u8; 8],
        pub attack: [u8; 8],
        pub defense: [u8; 8],
    }
}

// Client-side game state (richer, for UI)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterTemplate {
    pub name: String,
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
    pub description: String,
}

impl CharacterTemplate {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                name: "Freya".into(),
                hp: 100,
                atk: 18,
                def: 12,
                description: "Norse warrior goddess".into(),
            },
            Self {
                name: "Odin".into(),
                hp: 120,
                atk: 15,
                def: 15,
                description: "Allfather of wisdom".into(),
            },
            Self {
                name: "Thor".into(),
                hp: 110,
                atk: 22,
                def: 8,
                description: "God of thunder".into(),
            },
            Self {
                name: "Loki".into(),
                hp: 80,
                atk: 25,
                def: 5,
                description: "Trickster shapeshifter".into(),
            },
            Self {
                name: "Hel".into(),
                hp: 90,
                atk: 20,
                def: 10,
                description: "Queen of the dead".into(),
            },
            Self {
                name: "Tyr".into(),
                hp: 130,
                atk: 14,
                def: 18,
                description: "God of war and law".into(),
            },
        ]
    }
}

#[derive(Clone, Debug)]
pub struct BattleState {
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub enemy_hp: i32,
    pub enemy_max_hp: i32,
    pub player_atk: u32,
    pub player_def: u32,
    pub enemy_atk: u32,
    pub enemy_def: u32,
    pub turn: Turn,
    pub log: Vec<LogEntry>,
    pub result: Option<BattleResult>,
    pub score: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Turn {
    Player,
    Enemy,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub message: String,
    pub kind: LogKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogKind {
    Damage,
    Heal,
    Info,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BattleResult {
    Victory,
    Defeat,
}

impl BattleState {
    pub fn new(player: &CharacterTemplate) -> Self {
        let enemy_hp = 80 + (player.hp / 2);
        Self {
            player_hp: player.hp as i32,
            player_max_hp: player.hp as i32,
            enemy_hp: enemy_hp as i32,
            enemy_max_hp: enemy_hp as i32,
            player_atk: player.atk,
            player_def: player.def,
            enemy_atk: 12,
            enemy_def: 8,
            turn: Turn::Player,
            log: vec![LogEntry {
                message: "Battle begins!".into(),
                kind: LogKind::Info,
            }],
            result: None,
            score: 0,
        }
    }

    pub fn player_attack(&mut self) -> u32 {
        let base = self.player_atk as i32 - (self.enemy_def as i32 / 2);
        let variance = simple_random(5) as i32;
        let damage = (base + variance).max(1) as u32;
        self.enemy_hp = (self.enemy_hp - damage as i32).max(0);
        self.log.push(LogEntry {
            message: format!("You deal {} damage!", damage),
            kind: LogKind::Damage,
        });
        self.score += damage;
        if self.enemy_hp <= 0 {
            self.result = Some(BattleResult::Victory);
            self.log.push(LogEntry {
                message: "VICTORY! Enemy defeated!".into(),
                kind: LogKind::Info,
            });
        } else {
            self.turn = Turn::Enemy;
        }
        damage
    }

    pub fn player_defend(&mut self) {
        self.log.push(LogEntry {
            message: "You raise your guard!".into(),
            kind: LogKind::Info,
        });
        self.turn = Turn::Enemy;
    }

    pub fn enemy_attack(&mut self, player_defending: bool) -> u32 {
        let base = self.enemy_atk as i32 - (self.player_def as i32 / 2);
        let variance = simple_random(4) as i32;
        let mut damage = (base + variance).max(1) as u32;
        if player_defending {
            damage /= 2;
            self.log.push(LogEntry {
                message: format!("Blocked! Enemy deals only {} damage.", damage),
                kind: LogKind::Heal,
            });
        } else {
            self.log.push(LogEntry {
                message: format!("Enemy strikes for {} damage!", damage),
                kind: LogKind::Damage,
            });
        }
        self.player_hp = (self.player_hp - damage as i32).max(0);
        if self.player_hp <= 0 {
            self.result = Some(BattleResult::Defeat);
            self.log.push(LogEntry {
                message: "DEFEAT! You have fallen...".into(),
                kind: LogKind::Info,
            });
        } else {
            self.turn = Turn::Player;
        }
        damage
    }
}

fn simple_random(max: u32) -> u32 {
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf).unwrap_or_default();
    u32::from_le_bytes(buf) % max
}
