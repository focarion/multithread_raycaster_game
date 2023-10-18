use std::time::Instant;

use winit::event::VirtualKeyCode;

use crate::{assets::{Assets, Linedef}, render::BSPTree};
/* use rodio::{OutputStreamHandle, source::Source, SpatialSink}; */
/* use std::io::Cursor; */
const JUMP_ACCEL: f64 = 3.0; // Initial jump acceleration in m/s
// PlayerStates struct: Holds boolean flags for various player states like crouching, jumping, etc.
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
    pub pos: (f64, f64, f64),
    pub dir: (f64, f64),
    pub move_dir: (f64, f64),
    pub plane: (f64, f64),
    pub vertical_velocity: f64,
    pub movespeed: f64,
    pub screen_pitch: f64,
    pub height: f64,
    pub hitbox_radius: f64,
    pub states: PlayerStates,
    pub timings: PlayerTimings
}
// Implementing methods for Player struct
impl Player {
    // Constructor: Initializes a new player with default values
    pub fn new() -> Self {
        Self {
            pos : (0.5, 0.5, 0.0),
            dir : (-1.0, 0.0),
            move_dir: (-1.0, 0.0),
            plane : (0.0,0.66),
            vertical_velocity: 0.0,
            movespeed: 5.0,
            screen_pitch: 0.0,
            height: 1.0,
            hitbox_radius: 0.2,
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
    // handle_key: Handles key press events and updates player state
    pub fn handle_key(&mut self, keycode: VirtualKeyCode, delta_time: f64, bsp_tree: &BSPTree) {
        self.move_dir = (self.dir.0, self.dir.1);  // Reset move_dir to dir before handling keys
        match keycode {
            VirtualKeyCode::W => self.walk_in_direction(self.move_dir, delta_time, bsp_tree),
            VirtualKeyCode::A => self.walk_in_direction((-self.move_dir.1, self.move_dir.0), delta_time, bsp_tree),
            VirtualKeyCode::S => self.walk_in_direction((-self.move_dir.0, -self.move_dir.1), delta_time, bsp_tree),
            VirtualKeyCode::D => self.walk_in_direction((self.move_dir.1, -self.move_dir.0), delta_time, bsp_tree),
            VirtualKeyCode::LControl => self.crouch(),
            VirtualKeyCode::Space => self.jump(),
            _ => {}
        }
    }
    // walk_in_direction: Moves the player in a given direction
    pub fn walk_in_direction(&mut self, dir: (f64, f64), delta_time: f64, bsp_tree: &BSPTree) {
        let steps = 10;
        let step_x = dir.0 * self.movespeed * delta_time / steps as f64;
        let step_y = dir.1 * self.movespeed * delta_time / steps as f64;
    
        for _ in 0..steps {
            let new_x = self.pos.0 + step_x;
            let new_y = self.pos.1 + step_y;
    
            if !bsp_tree.check_collision((new_x, new_y, self.pos.2), self.hitbox_radius) {
                self.pos.0 = new_x;
                self.pos.1 = new_y;
            } else {
                if !bsp_tree.check_collision((new_x, self.pos.1, self.pos.2), self.hitbox_radius) {
                    self.pos.0 = new_x;
                }
                if !bsp_tree.check_collision((self.pos.0, new_y, self.pos.2), self.hitbox_radius) {
                    self.pos.1 = new_y;
                }
            }
        }
    }
    
    
    
    // rotate: Rotates the player's direction and plane
    pub fn rotate(&mut self, x: f64, x_sensitivity: f64) {
        let theta = -x / x_sensitivity;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let old_dir_x = self.dir.0;
        self.dir.0 = self.dir.0 * cos_theta - self.dir.1 * sin_theta;
        self.dir.1 = old_dir_x * sin_theta + self.dir.1 * cos_theta;

        let old_plane_x = self.plane.0;
        self.plane.0 = self.plane.0 * cos_theta - self.plane.1 * sin_theta;
        self.plane.1 = old_plane_x * sin_theta + self.plane.1 * cos_theta;
    }
    // adjust_pitch: Adjusts the player's screen pitch
    pub fn adjust_pitch(&mut self, y: f64, y_sensitivity: f64, scale_width: usize) {
        self.screen_pitch -= y * (y_sensitivity / 10000.0);
        let max_pitch_percentage = 1.4;
        self.screen_pitch = f64::clamp(self.screen_pitch, -((scale_width as f64) * max_pitch_percentage), (scale_width as f64) * max_pitch_percentage);
    }
    // move_camera: Handles camera movement
    pub fn move_camera(&mut self, x: f64, y: f64, x_sensitivity: f64, y_sensitivity: f64, scale_width: usize) {
        self.rotate(x, x_sensitivity);
        self.adjust_pitch(y, y_sensitivity, scale_width);
    }
    pub fn update_vertical_position(&mut self, delta_time: f64, linedefs: &Vec<Linedef>) {
        let gravity = 9.81;
        let jump_speed = 5.0;
        let crouch_speed = 2.0;
        let crouch_offset = if self.states.is_crouching { -0.5 } else { 0.0 };
        let mut closest_floor_level = 0.0;
    
        for linedef in linedefs.iter() {
            if self.pos.0 >= linedef.start.x && self.pos.0 <= linedef.end.x && self.pos.1 >= linedef.start.y && self.pos.1 <= linedef.end.y {
                closest_floor_level = linedef.floor_level.max(closest_floor_level);
            }
        }
    
        closest_floor_level += crouch_offset;
    
        self.vertical_velocity -= gravity * delta_time;
    
        if self.states.is_jumping && self.pos.2 == closest_floor_level {
            self.vertical_velocity = jump_speed;
            self.states.is_jumping = false;
        }
    
        if self.states.is_crouching {
            self.pos.2 -= crouch_speed * delta_time;
        }
    
        self.pos.2 += self.vertical_velocity * delta_time;
    
        if self.pos.2 < closest_floor_level {
            self.pos.2 = closest_floor_level;
            self.vertical_velocity = 0.0;
        }
    }
    
    
    
    
    
    // jump: Makes the player jump (i mean, what else would it do?)
    pub fn jump(&mut self) {
        if !self.states.is_crouched {
            if self.timings.movement_cooldown.elapsed() > std::time::Duration::from_millis(1250) {
                if !self.states.is_jumping && !self.states.is_crouching {
                    self.states.is_jumping = true;
                    self.vertical_velocity = JUMP_ACCEL;
                }
                self.timings.movement_cooldown = std::time::Instant::now();
            } 
        }
    }
    // crouch: Makes the player crouch
    pub fn crouch(&mut self) {
        if self.timings.movement_cooldown.elapsed() > std::time::Duration::from_millis(250) {
            if !self.states.is_jumping {
                self.states.is_crouching = !self.states.is_crouching;  // Toggle crouch state
            }
            self.timings.movement_cooldown = std::time::Instant::now();
        }
    }
    
}
// GameState struct: Holds the current game state including the player and assets
pub struct GameState {
    pub player: Player,
    pub assets: Assets,
}
// Implementing methods for GameState struct
impl GameState {
    // Constructor: Initializes a new game state
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            assets: Assets::new()
        }
    }
}