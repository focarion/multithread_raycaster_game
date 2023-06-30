
#[cfg(feature = "debug")]
use profiling::{tracy_client, scope};

use rayon::prelude::*;
use std::{time::Duration, collections::HashSet, num::NonZeroU32};
use winit::{
    event::{Event, DeviceEvent, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, CursorGrabMode, Fullscreen}, dpi::PhysicalSize,
};


mod assets;
mod render;
mod game_state;
mod util;
use game_state::GameState;
use render::{wall_buffer, ceiling_buffer, sprite_buffer, create_final_buffer};
use rodio::OutputStream;

use crate::{assets::{NUM_SPRITES, SPRITE, TEX_WIDTH, TEX_HEIGHT}, util::{rescale_buffer, render_text_to_buffer, game_setup}};
fn main() {
    // Event loop initialization and Game Config Setup, Still WIP and will be reworked later on
    let event_loop = EventLoop::new();
    let monitor_resolution = event_loop.primary_monitor().unwrap().size();
    let monitor = event_loop
        .available_monitors()
        .next()
        .expect("no monitor found!");
    let mode = monitor
        .video_modes()
        .next()
        .expect("no mode found");
    let (monitor_width, monitor_height) = (monitor_resolution.width, monitor_resolution.height);
    // Game config Setup
    let (
        render_screen_width,
        render_screen_height,
        scale_width,
        scale_height,
        thread_amount,
        supersample_factor,
        is_fullscreen
    ) = game_setup(monitor_width, monitor_height);
    // Setup the ThreadPool with the selected number of threads
    rayon::ThreadPoolBuilder::new().num_threads(thread_amount as usize).build_global().unwrap();
    #[cfg(feature = "debug")]
    {
    tracy_client::Client::start();
    
    // profiling::register_thread!("Main Thread");
    use tracing_subscriber::layer::SubscriberExt;
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(tracing_tracy::TracyLayer::new()),
        )
        .unwrap(); 
    }
    let display_size = PhysicalSize::new(scale_width as u32, scale_height as u32);
    let window = WindowBuilder::new()
        .with_title(format!("Multithread Raycaster Game Version: {}", clap::crate_version!()))
        .with_inner_size(display_size)
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();
    let graphics_context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&graphics_context, &window) }.unwrap();
    if is_fullscreen {
        if monitor_height as usize == scale_height && monitor_width as usize == scale_width {
            let fullscreen = Some(Fullscreen::Borderless(Some(monitor.clone())));
             window.set_fullscreen(fullscreen);
            
        } else {
            let fullscreen = Some(Fullscreen::Exclusive(mode.clone()));
             window.set_fullscreen(fullscreen);
        }
        
    }
    surface
        .resize(
            NonZeroU32::new(scale_width as u32).unwrap(),
            NonZeroU32::new(scale_height as u32).unwrap(),
        )
        .unwrap();
    let start_time = std::time::Instant::now();
    let mut frames = 0;
    let mut last_fps_print_time = start_time;
    let mut previous_frame_time = std::time::Instant::now();
    
    let mut mouse_lock = false;
    let mut mouse_lock_setup = true;
    let mut game_state = GameState::new();
    #[allow(unused_assignments)]
    let mut pressed_keys = HashSet::new();
    let mut frames_per_second = String::new();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut footsteps_audio = game_state.create_footsteps(&stream_handle);
    let mut footsteps_cycle = false;
    #[cfg(feature = "debug")]
    scope!("Main Loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let current_frame_time = std::time::Instant::now();
        let frame_duration = current_frame_time.duration_since(previous_frame_time);
        previous_frame_time = current_frame_time;
        let frame_time_ms = frame_duration.as_millis();
        
        if !game_state.player.states.is_crouched {
            game_state.player.movespeed = (frame_time_ms as f64 / 1000.0) * 3.5; 
        } else {
            game_state.player.movespeed = (frame_time_ms as f64 / 1000.0) * 2.0;
        }
        if game_state.player.states.is_walking && !game_state.player.states.was_player_walking && footsteps_audio.empty() {
            footsteps_cycle = if footsteps_cycle == false {true} else {false};
            if footsteps_cycle {
                footsteps_audio = game_state.create_footsteps(&stream_handle);
                footsteps_audio.set_emitter_position([0.3, 0., 0.]);
            } else {
                footsteps_audio = game_state.create_footsteps(&stream_handle);
                footsteps_audio.set_emitter_position([-0.3, 0., 0.]);
            }
        
        }else if !game_state.player.states.is_walking && game_state.player.states.was_player_walking && !footsteps_audio.empty(){

        } else if !game_state.player.states.is_walking && game_state.player.states.was_player_walking && footsteps_audio.empty() {
            footsteps_audio.stop();
        }
          
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                #[cfg(feature = "debug")]
                profiling::scope!("Redraw");
                {
                    frames += 1;
                    if last_fps_print_time.elapsed() > Duration::from_secs(1) {
                        //println!("FPS: {:.1}", frames as f64 / last_fps_print_time.elapsed().as_secs_f64());
                        frames_per_second = format!("FPS: {:.0}", frames as f64 / last_fps_print_time.elapsed().as_secs_f64());
                        last_fps_print_time = std::time::Instant::now();
                        frames = 0;
                        if mouse_lock_setup {
                            window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                            window.set_cursor_visible(false);
                            mouse_lock = true;
                            mouse_lock_setup = false;
                        }
                    }
                    let half_screen_height = render_screen_height as f64 / 2.0;
                    let ray_dir_x0 = game_state.player.dir.0 - game_state.player.plane.0;
                    let ray_dir_y0 = game_state.player.dir.1 - game_state.player.plane.1;
                    let ray_dir_x1 = game_state.player.dir.0 + game_state.player.plane.0;
                    let ray_dir_y1 = game_state.player.dir.1 + game_state.player.plane.1;

                    let ceiling_floor_buffer = ceiling_buffer(render_screen_width, render_screen_height, game_state.player.screen_pitch, half_screen_height, ray_dir_x1, ray_dir_x0, ray_dir_y1, ray_dir_y0, game_state.player.pos, game_state.assets.textures.clone(), supersample_factor);

                    let (wall_buffer, z_buffer) = wall_buffer(render_screen_width, render_screen_height,game_state.player.plane, game_state.player.dir, game_state.player.pos, game_state.map.clone(),game_state.player.screen_pitch, game_state.assets.textures.clone(), supersample_factor);
                    

                    let sprite_data: Vec<(usize, f64)> = (0..NUM_SPRITES).into_par_iter().map(|index| {
                        let order = index;
                        let dist = (game_state.player.pos.0 - SPRITE[index].x) * (game_state.player.pos.0 - SPRITE[index].x) + (game_state.player.pos.1 - SPRITE[index].y) * (game_state.player.pos.1 - SPRITE[index].y);
                        (order, dist)
                    }).collect();

                    let mut sprite_data = sprite_data;
                    sprite_data.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                    let sprite_order: Vec<usize> = sprite_data.into_iter().map(|(order, _)| order).collect();

                    let sprite_buffer = sprite_buffer(render_screen_width, render_screen_height, sprite_order, game_state.player.pos, game_state.player.plane, game_state.player.dir, game_state.player.screen_pitch, z_buffer, game_state.assets.textures.clone(), supersample_factor);

                    


                    let final_buffer = create_final_buffer(wall_buffer, ceiling_floor_buffer, sprite_buffer);
                    #[cfg(feature = "debug")]
                    profiling::scope!("Draw Frame");
                    {
                        #[cfg(feature = "debug")]
                        profiling::scope!("Draw to Winit Frame");
                        {
                            let final_buffer = rescale_buffer(&final_buffer, render_screen_width, render_screen_height, scale_width, scale_height);
                            let mut buffer: Vec<u32> = final_buffer.into_par_iter().flatten_iter().collect();
                            if !mouse_lock {
                                render_text_to_buffer(255, 75, 75, "Mouse Lock Off Press Alt To Enable",45.0, game_state.assets.fonts[0].clone(), (0.0, 0.0),&mut buffer,  scale_width, scale_height, );
                            }
                            render_text_to_buffer(255, 255, 255, &format!("{}", frames_per_second),35.0, game_state.assets.fonts[0].clone(), (render_screen_width as f32 * 0.85, 0.0),&mut buffer,  scale_width, scale_height, );
                            //info!("Finish Assemblying final buffer");
                            #[cfg(feature = "debug")]
                            profiling::scope!("Set Buffer");
                            {
                                let mut draw_buffer = surface.buffer_mut().unwrap();
                                {
                                    draw_buffer.copy_from_slice(&buffer);
                                }
                                draw_buffer.present().unwrap();

                            }
                            //info!("Set Buffer");
                        }
                        game_state.player.update_jump();
                        #[cfg(feature = "debug")]
                        profiling::finish_frame!();
                    }
                }
            }
            Event::MainEventsCleared {} =>{
                window.request_redraw();
                game_state.player.states.is_walking = false;
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {event: WindowEvent::KeyboardInput {input: KeyboardInput {virtual_keycode: Some(VirtualKeyCode::Escape), ..}, ..}, ..} => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {event: WindowEvent::KeyboardInput {input: KeyboardInput {virtual_keycode: Some(VirtualKeyCode::LAlt), state: ElementState::Released, ..}, ..}, ..} => {
                if mouse_lock {
                    window.set_cursor_grab(CursorGrabMode::None).unwrap();
                    window.set_cursor_visible(true);
                    mouse_lock = false;
                } else {
                    window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                    window.set_cursor_visible(false);
                    mouse_lock = true;
                }
                },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => { 
                    if mouse_lock {
                        let (x, y) = delta;
                        game_state.player.move_camera(x, y, scale_width);
                    }
                    
                    
                    },
                _ => (),
            },
             Event::WindowEvent {
                 event: WindowEvent::KeyboardInput {
                     input: KeyboardInput {
                         state: keycode_state,  virtual_keycode, ..
                     }, ..
                 }, ..
             } =>  {
                 if let (state, Some(key)) = (keycode_state, virtual_keycode) {
                        match state {
                            ElementState::Pressed => {
                                pressed_keys.insert(key);
                            }
                            ElementState::Released => {
                                pressed_keys.remove(&key);
                            }
                        }
                    }
             }
                _ => (),
            
            
        }
        
        for keycode in pressed_keys.clone() {
                match keycode {
                    VirtualKeyCode::W => {
                        game_state.player.walk_forward(&game_state.map);
                        game_state.player.states.is_walking = true;
                    },
                    VirtualKeyCode::A => {
                        game_state.player.walk_left(&game_state.map);
                        game_state.player.states.is_walking = true;
                    },
                    VirtualKeyCode::S => {
                        game_state.player.walk_backward(&game_state.map);
                        game_state.player.states.is_walking = true;
                    },
                    VirtualKeyCode::D => {
                        game_state.player.walk_right(&game_state.map);
                        game_state.player.states.is_walking = true;
                    },
                    VirtualKeyCode::LControl => {
                        game_state.player.crouch()
                    },
                    VirtualKeyCode::Space => {
                        game_state.player.jump()
                    },
                    
                    
                    _ => continue
                }
            }
    });
}
