use std::time::Instant;

use crate::assets::Assets;
use rodio::{OutputStreamHandle, source::Source, SpatialSink};
use std::io::Cursor;

#[derive(Copy, Clone)]
pub struct PlayerStates {
    pub is_crouching: bool,
    pub is_crouched: bool,
    pub is_jumping: bool,
    pub is_walking: bool,
    pub is_steps_playing: bool,
    pub was_player_walking: bool,
}
#[derive(Copy, Clone)]
pub struct PlayerTimings {
    pub last_updated: Instant,
    pub movement_cooldown: Instant
}
pub struct Player {
    pub pos: (f64, f64, f64),
    pub dir: (f64, f64),
    pub plane: (f64, f64),
    pub vertical_velocity: f64,
    pub movespeed: f64,
    pub screen_pitch: f64,
    pub hitbox_radius: f64,
    pub states: PlayerStates,
    pub timings: PlayerTimings
}
impl Player {
    pub fn new() -> Self {
        // Initialize player state
        Self {
            pos : (22.0, 11.5, 0.0),
            dir : (-1.0, 0.0),
            plane : (0.0,0.66),
            vertical_velocity: 0.0,
            movespeed: 0.05,
            screen_pitch: 0.0,
            hitbox_radius: 0.2,
            states: PlayerStates {
                 is_crouching: false,
                 is_crouched: false,
                 is_jumping: false,
                 is_walking: false,
                 is_steps_playing: false,
                 was_player_walking: false,
                },
            timings: PlayerTimings {
                last_updated: std::time::Instant::now(),
                movement_cooldown: std::time::Instant::now()
            }
                
        }
    }
    pub fn update_jump(&mut self) -> Self {
        if  self.states.is_jumping && !self.states.is_crouching && !self.states.is_crouched {
            let now = std::time::Instant::now();
            let delta_time = (now - self.timings.last_updated).as_secs_f64() * 3.0;
            self.timings.last_updated = now;
            self.pos.2 += self.vertical_velocity * delta_time;
            self.vertical_velocity -= 50.0 * delta_time;
        if self.pos.2 >= 200.0 {
            self.pos.2 = 200.0;
            self.vertical_velocity = -300.0;
        } else if self.pos.2 <= 0.0 {
            self.pos.2 = 0.0;
            self.states.is_jumping = false;
            self.vertical_velocity = 0.0;
        }
        } else if !self.states.is_jumping && self.states.is_crouching {
            let now = std::time::Instant::now();
            let delta_time = (now - self.timings.last_updated).as_secs_f64() * 3.0;
            self.timings.last_updated = now;
            if !self.states.is_crouched {
                self.pos.2 -= self.vertical_velocity * delta_time;
                if self.pos.2 <= -200.0 {
                    self.pos.2 = -200.0;
                    self.vertical_velocity = 0.0;
                    self.states.is_crouching = false;
                    self.states.is_crouched = true
                }
                
            } else {
                self.pos.2 += self.vertical_velocity * delta_time;
                if self.pos.2 >= 0.0 {
                    self.pos.2 = 0.0;
                    self.vertical_velocity = 0.0;
                    self.states.is_crouching = false;
                    self.states.is_crouched = false
                }
                
            }
            


        }
        Self { pos: self.pos, dir: self.dir, plane: self.plane, vertical_velocity: self.vertical_velocity, movespeed: self.movespeed, screen_pitch: self.screen_pitch, hitbox_radius: self.hitbox_radius, states: self.states, timings: self.timings }
    }
    pub fn walk_forward(&mut self, map: &Vec<Vec<usize>>) {
        let new_pos_x = self.pos.0 + self.dir.0 * self.movespeed;
        let new_pos_y = self.pos.1 + self.dir.1 * self.movespeed;
    
        // Calculate all four hitbox corners for the new X position
        let hitbox_corner_1_x = (new_pos_x + self.hitbox_radius, self.pos.1 + self.hitbox_radius);
        let hitbox_corner_2_x = (new_pos_x - self.hitbox_radius, self.pos.1 + self.hitbox_radius);
        let hitbox_corner_3_x = (new_pos_x + self.hitbox_radius, self.pos.1 - self.hitbox_radius);
        let hitbox_corner_4_x = (new_pos_x - self.hitbox_radius, self.pos.1 - self.hitbox_radius);
    
        // Check for collisions at all hitbox corners for the new X position
        if map[hitbox_corner_1_x.0 as usize][hitbox_corner_1_x.1 as usize] == 0 && 
           map[hitbox_corner_2_x.0 as usize][hitbox_corner_2_x.1 as usize] == 0 &&
           map[hitbox_corner_3_x.0 as usize][hitbox_corner_3_x.1 as usize] == 0 &&
           map[hitbox_corner_4_x.0 as usize][hitbox_corner_4_x.1 as usize] == 0 {
            // If no collisions are detected, update the player's X position
            self.pos.0 = new_pos_x;
        }
    
        // Calculate all four hitbox corners for the new Y position
        let hitbox_corner_1_y = (self.pos.0 + self.hitbox_radius, new_pos_y + self.hitbox_radius);
        let hitbox_corner_2_y = (self.pos.0 - self.hitbox_radius, new_pos_y + self.hitbox_radius);
        let hitbox_corner_3_y = (self.pos.0 + self.hitbox_radius, new_pos_y - self.hitbox_radius);
        let hitbox_corner_4_y = (self.pos.0 - self.hitbox_radius, new_pos_y - self.hitbox_radius);
    
        // Check for collisions at all hitbox corners for the new Y position
        if map[hitbox_corner_1_y.0 as usize][hitbox_corner_1_y.1 as usize] == 0 && 
           map[hitbox_corner_2_y.0 as usize][hitbox_corner_2_y.1 as usize] == 0 &&
           map[hitbox_corner_3_y.0 as usize][hitbox_corner_3_y.1 as usize] == 0 &&
           map[hitbox_corner_4_y.0 as usize][hitbox_corner_4_y.1 as usize] == 0 {
            // If no collisions are detected, update the player's Y position
            self.pos.1 = new_pos_y;
        }
    }
    
    
    pub fn walk_left(&mut self, map: &Vec<Vec<usize>>) {
        let new_pos_x = self.pos.0 - self.dir.1 * self.movespeed;
        let new_pos_y = self.pos.1 + self.dir.0 * self.movespeed;
    
        // Calculate all four hitbox corners for the new X position
        let hitbox_corner_1_x = (new_pos_x + self.hitbox_radius, self.pos.1 + self.hitbox_radius);
        let hitbox_corner_2_x = (new_pos_x - self.hitbox_radius, self.pos.1 + self.hitbox_radius);
        let hitbox_corner_3_x = (new_pos_x + self.hitbox_radius, self.pos.1 - self.hitbox_radius);
        let hitbox_corner_4_x = (new_pos_x - self.hitbox_radius, self.pos.1 - self.hitbox_radius);
    
        // Check for collisions at all hitbox corners for the new X position
        if map[hitbox_corner_1_x.0 as usize][hitbox_corner_1_x.1 as usize] == 0 && 
           map[hitbox_corner_2_x.0 as usize][hitbox_corner_2_x.1 as usize] == 0 &&
           map[hitbox_corner_3_x.0 as usize][hitbox_corner_3_x.1 as usize] == 0 &&
           map[hitbox_corner_4_x.0 as usize][hitbox_corner_4_x.1 as usize] == 0 {
            // If no collisions are detected, update the player's X position
            self.pos.0 = new_pos_x;
        }
    
        // Calculate all four hitbox corners for the new Y position
        let hitbox_corner_1_y = (self.pos.0 + self.hitbox_radius, new_pos_y + self.hitbox_radius);
        let hitbox_corner_2_y = (self.pos.0 - self.hitbox_radius, new_pos_y + self.hitbox_radius);
        let hitbox_corner_3_y = (self.pos.0 + self.hitbox_radius, new_pos_y - self.hitbox_radius);
        let hitbox_corner_4_y = (self.pos.0 - self.hitbox_radius, new_pos_y - self.hitbox_radius);
    
        // Check for collisions at all hitbox corners for the new Y position
        if map[hitbox_corner_1_y.0 as usize][hitbox_corner_1_y.1 as usize] == 0 && 
           map[hitbox_corner_2_y.0 as usize][hitbox_corner_2_y.1 as usize] == 0 &&
           map[hitbox_corner_3_y.0 as usize][hitbox_corner_3_y.1 as usize] == 0 &&
           map[hitbox_corner_4_y.0 as usize][hitbox_corner_4_y.1 as usize] == 0 {
            // If no collisions are detected, update the player's Y position
            self.pos.1 = new_pos_y;
        }
    }
    pub fn walk_right(&mut self, map: &Vec<Vec<usize>>) {
        let new_pos_x = self.pos.0 + self.dir.1 * self.movespeed;
        let new_pos_y = self.pos.1 - self.dir.0 * self.movespeed;
    
        // Calculate all four hitbox corners for the new X position
        let hitbox_corner_1_x = (new_pos_x + self.hitbox_radius, self.pos.1 + self.hitbox_radius);
        let hitbox_corner_2_x = (new_pos_x - self.hitbox_radius, self.pos.1 + self.hitbox_radius);
        let hitbox_corner_3_x = (new_pos_x + self.hitbox_radius, self.pos.1 - self.hitbox_radius);
        let hitbox_corner_4_x = (new_pos_x - self.hitbox_radius, self.pos.1 - self.hitbox_radius);
    
        // Check for collisions at all hitbox corners for the new X position
        if map[hitbox_corner_1_x.0 as usize][hitbox_corner_1_x.1 as usize] == 0 && 
           map[hitbox_corner_2_x.0 as usize][hitbox_corner_2_x.1 as usize] == 0 &&
           map[hitbox_corner_3_x.0 as usize][hitbox_corner_3_x.1 as usize] == 0 &&
           map[hitbox_corner_4_x.0 as usize][hitbox_corner_4_x.1 as usize] == 0 {
            // If no collisions are detected, update the player's X position
            self.pos.0 = new_pos_x;
        }
    
        // Calculate all four hitbox corners for the new Y position
        let hitbox_corner_1_y = (self.pos.0 + self.hitbox_radius, new_pos_y + self.hitbox_radius);
        let hitbox_corner_2_y = (self.pos.0 - self.hitbox_radius, new_pos_y + self.hitbox_radius);
        let hitbox_corner_3_y = (self.pos.0 + self.hitbox_radius, new_pos_y - self.hitbox_radius);
        let hitbox_corner_4_y = (self.pos.0 - self.hitbox_radius, new_pos_y - self.hitbox_radius);
    
        // Check for collisions at all hitbox corners for the new Y position
        if map[hitbox_corner_1_y.0 as usize][hitbox_corner_1_y.1 as usize] == 0 && 
           map[hitbox_corner_2_y.0 as usize][hitbox_corner_2_y.1 as usize] == 0 &&
           map[hitbox_corner_3_y.0 as usize][hitbox_corner_3_y.1 as usize] == 0 &&
           map[hitbox_corner_4_y.0 as usize][hitbox_corner_4_y.1 as usize] == 0 {
            // If no collisions are detected, update the player's Y position
            self.pos.1 = new_pos_y;
        }
    }
    pub fn walk_backward(&mut self, map: &Vec<Vec<usize>>) {
        let new_pos_x = self.pos.0 - self.dir.0 * self.movespeed;
        let new_pos_y = self.pos.1 - self.dir.1 * self.movespeed;
    
        // Calculate all four hitbox corners for the new X position
        let hitbox_corner_1_x = (new_pos_x + self.hitbox_radius, self.pos.1 + self.hitbox_radius);
        let hitbox_corner_2_x = (new_pos_x - self.hitbox_radius, self.pos.1 + self.hitbox_radius);
        let hitbox_corner_3_x = (new_pos_x + self.hitbox_radius, self.pos.1 - self.hitbox_radius);
        let hitbox_corner_4_x = (new_pos_x - self.hitbox_radius, self.pos.1 - self.hitbox_radius);
    
        // Check for collisions at all hitbox corners for the new X position
        if map[hitbox_corner_1_x.0 as usize][hitbox_corner_1_x.1 as usize] == 0 && 
           map[hitbox_corner_2_x.0 as usize][hitbox_corner_2_x.1 as usize] == 0 &&
           map[hitbox_corner_3_x.0 as usize][hitbox_corner_3_x.1 as usize] == 0 &&
           map[hitbox_corner_4_x.0 as usize][hitbox_corner_4_x.1 as usize] == 0 {
            // If no collisions are detected, update the player's X position
            self.pos.0 = new_pos_x;
        }
    
        // Calculate all four hitbox corners for the new Y position
        let hitbox_corner_1_y = (self.pos.0 + self.hitbox_radius, new_pos_y + self.hitbox_radius);
        let hitbox_corner_2_y = (self.pos.0 - self.hitbox_radius, new_pos_y + self.hitbox_radius);
        let hitbox_corner_3_y = (self.pos.0 + self.hitbox_radius, new_pos_y - self.hitbox_radius);
        let hitbox_corner_4_y = (self.pos.0 - self.hitbox_radius, new_pos_y - self.hitbox_radius);
    
        // Check for collisions at all hitbox corners for the new Y position
        if map[hitbox_corner_1_y.0 as usize][hitbox_corner_1_y.1 as usize] == 0 && 
           map[hitbox_corner_2_y.0 as usize][hitbox_corner_2_y.1 as usize] == 0 &&
           map[hitbox_corner_3_y.0 as usize][hitbox_corner_3_y.1 as usize] == 0 &&
           map[hitbox_corner_4_y.0 as usize][hitbox_corner_4_y.1 as usize] == 0 {
            // If no collisions are detected, update the player's Y position
            self.pos.1 = new_pos_y;
        }
    }
    
    
    pub fn move_camera(&mut self, x: f64, y: f64, scale_width: usize) {
        let sensitivity = scale_width as f64 / 5.0;
        let old_dir_x = self.dir.0;
        self.dir.0 = self.dir.0 * (-x / sensitivity).cos() - self.dir.1 * (-x / sensitivity).sin();
        self.dir.1 = old_dir_x * (-x / sensitivity).sin() + self.dir.1 * (-x / sensitivity).cos();
        let old_plane_x = self.plane.0;
        self.plane.0 = self.plane.0 * (-x / sensitivity).cos() - self.plane.1 * (-x / sensitivity).sin();
        self.plane.1 = old_plane_x * (-x / sensitivity).sin() + self.plane.1 * (-x / sensitivity).cos();
        self.screen_pitch -= y;
        let max_pitch_percentage = 1.25;
        self.screen_pitch = f64::clamp(self.screen_pitch, -((scale_width as f64)*max_pitch_percentage), (scale_width as f64)*max_pitch_percentage);
    }
    pub fn jump(&mut self) {
        if !self.states.is_crouched {
            if self.timings.movement_cooldown.elapsed() > std::time::Duration::from_millis(250) {
                if !self.states.is_jumping && !self.states.is_crouching {
                    self.states.is_jumping = true;
                    self.vertical_velocity = 300.0;
                }
                self.timings.movement_cooldown = std::time::Instant::now();
            } 
        }
    }
    pub fn crouch(&mut self) {
        if !self.states.is_jumping {
            if self.timings.movement_cooldown.elapsed() > std::time::Duration::from_millis(250) {
                if !self.states.is_crouching && !self.states.is_crouched {
                    self.states.is_crouching = true;
                    self.vertical_velocity = 250.0;
                } else if !self.states.is_crouching && self.states.is_crouched {
                    self.states.is_crouching = true;
                    self.vertical_velocity = 250.0
                }
                self.timings.movement_cooldown = std::time::Instant::now();
            }
        }
    }
}

pub struct GameState {
    pub player: Player,
    pub map: Vec<Vec<usize>>,
    pub assets: Assets,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            map: vec![
                vec![8,8,8,8,8,8,8,8,8,8,8,4,4,6,4,4,6,4,6,4,4,4,6,4],
                vec![8,0,0,0,0,0,0,0,0,0,8,4,0,0,0,0,0,0,0,0,0,0,0,4],
                vec![8,0,3,3,0,0,0,0,0,8,8,4,0,0,0,0,0,0,0,0,0,0,0,6],
                vec![8,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6],
                vec![8,0,3,3,0,0,0,0,0,8,8,4,0,0,0,0,0,0,0,0,0,0,0,4],
                vec![8,0,0,0,0,0,0,0,0,0,8,4,0,0,0,0,0,6,6,6,0,6,4,6],
                vec![8,8,8,8,0,8,8,8,8,8,8,4,4,4,4,4,4,6,0,0,0,0,0,6],
                vec![7,7,7,7,0,7,7,7,7,0,8,0,8,0,8,0,8,4,0,4,0,6,0,6],
                vec![7,7,0,0,0,0,0,0,7,8,0,8,0,8,0,8,8,6,0,0,0,0,0,6],
                vec![7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,6,0,0,0,0,0,4],
                vec![7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,6,0,6,0,6,0,6],
                vec![7,7,0,0,0,0,0,0,7,8,0,8,0,8,0,8,8,6,4,6,0,6,6,6],
                vec![7,7,7,7,0,7,7,7,7,8,8,4,0,6,8,4,8,3,3,3,0,3,3,3],
                vec![2,2,2,2,0,2,2,2,2,4,6,4,0,0,6,0,6,3,0,0,0,0,0,3],
                vec![2,2,0,0,0,0,0,2,2,4,0,0,0,0,0,0,4,3,0,0,0,0,0,3],
                vec![2,0,0,0,0,0,0,0,2,4,0,0,0,0,0,0,4,3,0,0,0,0,0,3],
                vec![1,0,0,0,0,0,0,0,1,4,4,4,4,4,6,0,6,3,3,0,0,0,3,3],
                vec![2,0,0,0,0,0,0,0,2,2,2,1,2,2,2,6,6,0,0,5,0,5,0,5],
                vec![2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,0,5,0,5,0,0,0,5,5],
                vec![2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
                vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5],
                vec![2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
                vec![2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,0,5,0,5,0,0,0,5,5],
                vec![2,2,2,2,1,2,2,2,2,2,2,1,2,2,2,5,5,5,5,5,5,5,5,5]
            ],
            assets: Assets::new()
        }
    }
    pub fn create_footsteps(&mut self, handle: &OutputStreamHandle) -> SpatialSink {
        let footsteps_audio_bytes = self.assets.sounds[0];
        let cursor = Cursor::new(footsteps_audio_bytes);
        let source = rodio::Decoder::new_mp3(cursor).unwrap();
    
        let sink = SpatialSink::try_new(handle, [0., 0., 0.], [-1., 0., 0.], [1., 0., 0.]).unwrap();
        sink.append(source.convert_samples::<i16>());
        sink
    }
    
}