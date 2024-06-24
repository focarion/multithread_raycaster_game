use std::time::Instant;
use dot_vox::{DotVoxData, load};
use glam::Vec3A;

#[derive(Copy, Clone)]
pub struct PlayerStates {
    pub is_crouching: bool,
    pub is_crouched: bool,
    pub is_jumping: bool,
    pub is_walking: bool,
    pub is_colliding: bool,
    pub is_steps_playing: bool,
    pub was_player_walking: bool,
}
// PlayerTimings struct: Holds timing information for player actions
#[derive(Copy, Clone)]
pub struct PlayerTimings {
    pub last_updated: Instant,
    pub movement_cooldown: Instant
}
// Player struct: The big boy, holds all player-related data
pub struct Player {
    pub pos: Vec3A,
    pub movespeed: f64,
    pub states: PlayerStates,
    pub timings: PlayerTimings
}
impl Player {
    // Constructor: Initializes a new player with default values
    pub fn new() -> Self {
        Self {
            pos : Vec3A::new(0.5, 0.0, 0.5),
            movespeed: 5.0,
            states: PlayerStates {
                is_crouching: false,
                is_crouched: false,
                is_jumping: false,
                is_walking: false,
                is_colliding: false,
                is_steps_playing: false,
                was_player_walking: false,
                },
            timings: PlayerTimings {
                last_updated: std::time::Instant::now(),
                movement_cooldown: std::time::Instant::now()
            }
                
        }
    }
}
pub struct Assets {
    pub voxels: Vec<DotVoxData>
}
impl Assets {
    pub fn new() -> Self {
        Self {
            voxels: vec![load("assets/template.vox").unwrap()]
        }
    }
}
pub struct State {
    pub player: Player,
    pub assets: Assets,
}
impl State {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            assets: Assets::new()
        }
    }
}
