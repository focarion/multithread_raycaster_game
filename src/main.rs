
#[cfg(feature = "debug")]
use profiling::{tracy_client, scope};

use std::{time::Duration, collections::HashSet, num::NonZeroU32, rc::Rc};
use winit::{
    event::{Event, DeviceEvent, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, CursorGrabMode, Fullscreen}, dpi::PhysicalSize, keyboard::{Key, NamedKey},
};


mod assets;
mod render;
mod game_state;
mod util;
use game_state::GameState;

use crate::{ assets::{Linedef, Vertex, TextureId},  util::{render_text_to_buffer, game_setup}, render::{BSPTree, Renderer}};
fn main() {
    // Event loop initialization and Game Config Setup, Still WIP and will be reworked later on
    let event_loop = EventLoop::new().unwrap();
    let monitor_resolution = event_loop.available_monitors().next().unwrap().size();
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
        _supersample_factor,
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
    let window = Rc::new(WindowBuilder::new()
        .with_title(format!("Multithread Raycaster Game Version: {}", clap::crate_version!()))
        .with_inner_size(display_size)
        .with_resizable(false)
        .build(&event_loop)
        .unwrap());
    let context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
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
    
    let mut mouse_lock = false;
    let mut mouse_lock_setup = true;
    #[allow(unused_assignments)]
    let mut pressed_keys = HashSet::new();
    let mut frames_per_second = String::new();
    let mut last_frame_time = std::time::Instant::now();
/*     let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut footsteps_audio = game_state.create_footsteps(&stream_handle);
    let mut footsteps_cycle = false; */

    let vertices = [
        Vertex { x: 0.0, y: -1.0 }, // Start
        Vertex { x: 0.0, y: 1.0 }, //End
        Vertex { x:0.25, y: 2.0 },
        Vertex { x:1.0, y: 2.75 },
        Vertex { x:3.0, y: 2.75 },
        Vertex { x:3.75, y: 2.0 },
        Vertex { x:4.0, y: 1.0 }, // Start 2
        Vertex { x:4.0, y: -1.0 }, // End 2
        Vertex { x:3.75, y: -2.0 },
        Vertex { x:3.0, y: -2.75 },
        Vertex { x:1.0, y: -2.75 },
        Vertex { x:0.25, y: -2.0 },


    ];

    let linedefs = vec![
        Linedef { start: vertices[0], end: vertices[1], height: 2.0, floor_level: 0.0, texture: TextureId::BrickWalls },
        Linedef { start: vertices[1], end: vertices[2], height: 2.0, floor_level: 0.0, texture: TextureId::RedBricks },
        Linedef { start: vertices[2], end: vertices[3], height: 2.0, floor_level: 0.0, texture: TextureId::CorrodedCooperWall },
        Linedef { start: vertices[3], end: vertices[4], height: 2.0, floor_level: 0.0, texture: TextureId::BrickWalls },
        Linedef { start: vertices[4], end: vertices[5], height: 2.0, floor_level: 0.0, texture: TextureId::RedBricks },
        Linedef { start: vertices[5], end: vertices[6], height: 2.0, floor_level: 0.0, texture: TextureId::CorrodedCooperWall },
        Linedef { start: vertices[6], end: vertices[7], height: 2.0, floor_level: 0.0, texture: TextureId::BrickWalls },
        Linedef { start: vertices[7], end: vertices[8], height: 2.0, floor_level: 0.0, texture: TextureId::RedBricks },
        Linedef { start: vertices[8], end: vertices[9], height: 2.0, floor_level: 0.0, texture: TextureId::CorrodedCooperWall },
        Linedef { start: vertices[9], end: vertices[10], height: 2.0, floor_level: 0.0, texture: TextureId::BrickWalls },
        Linedef { start: vertices[10], end: vertices[11], height: 2.0, floor_level: 0.0, texture: TextureId::RedBricks },
        Linedef { start: vertices[11], end: vertices[0], height: 2.0, floor_level: 0.0, texture: TextureId::BrickWalls },
/*         Linedef { start: vertices[1], end: vertices[2], height: 1.0, floor_level: 0.0, texture: TextureId::CorrodedCooperWall },
        Linedef { start: vertices[2], end: vertices[3], height: 2.0, floor_level: 0.0, texture: TextureId::GreyBricks },
        Linedef { start: vertices[3], end: vertices[4], height: 1.0, floor_level: 0.0, texture: TextureId::RedFlooring },
        Linedef { start: vertices[4], end: vertices[5], height: 2.0, floor_level: 0.0, texture: TextureId::RedBricks },
        Linedef { start: vertices[5], end: vertices[0], height: 1.0, floor_level: 0.0, texture: TextureId::Ceiling }, */
    ];
    let mut game_state = GameState::new();
    let bsp_tree = BSPTree::build(linedefs.clone());
    println!("{:#?}", bsp_tree);
    let mut renderer = Renderer::new(render_screen_width, render_screen_height);
    #[cfg(feature = "debug")]
    scope!("Main Loop");
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        let current_frame_time = std::time::Instant::now();
        let delta_time = (current_frame_time - last_frame_time).as_secs_f64().clone();
        last_frame_time = current_frame_time;
        game_state.player.update_vertical_position(delta_time, &linedefs);
        match event {
            Event::WindowEvent { event, .. } => match event {
                    WindowEvent::RedrawRequested => {
                        frames += 1;
                        if last_fps_print_time.elapsed() > Duration::from_secs(1) {
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
                        renderer.raycast_and_render_polygons(&bsp_tree, &game_state.player,&game_state.assets, render_screen_width, render_screen_height);
                        if !mouse_lock {
                            render_text_to_buffer(255, 75, 75, "Mouse Lock Off Press Alt To Enable",45.0, game_state.assets.fonts[0].clone(), (0.0, 0.0),&mut renderer.buffer,  scale_width, scale_height, );
                        }
                        render_text_to_buffer(255, 255, 255, &format!("{}", frames_per_second),35.0, game_state.assets.fonts[0].clone(), (render_screen_width as f32 * 0.85, 0.0),&mut renderer.buffer,  scale_width, scale_height, );
                        let mut draw_buffer = surface.buffer_mut().unwrap();
                        {
                            draw_buffer.copy_from_slice(&renderer.buffer)
                        }
                        draw_buffer.present().unwrap();
                    },
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    },
                    WindowEvent::KeyboardInput { event, .. } => {
                        let (state, key) = (event.state, event.logical_key.clone());
                        match event.logical_key {
                            Key::Named(NamedKey::Escape) => {
                                elwt.exit();
                            },
                            Key::Named(NamedKey::Alt) => {
                                match state {

                                    ElementState::Pressed => {
                                        if mouse_lock {
                                            window.set_cursor_grab(CursorGrabMode::None).unwrap();
                                            window.set_cursor_visible(true);
                                            mouse_lock = false;
                                        } else {
                                            window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                                            window.set_cursor_visible(false);
                                            mouse_lock = true;
                                        }
                                    }
                                    _ => {}
                                }
                                
                            },
                            _ => {}

                        }
                        match state {
                            ElementState::Pressed => {
                                pressed_keys.insert(key);
                            }
                            ElementState::Released => {
                                pressed_keys.remove(&key);
                            }
                        }
                    }

                    _ => {}
                    
            }
                
            
            
            Event::AboutToWait {} =>{
                window.request_redraw();
                game_state.player.states.is_walking = false;
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => { 
                    if mouse_lock {
                        let (x, y) = delta;
                        game_state.player.move_camera(x, y, scale_width as f64 * 4.0,  scale_height as f64 * 4.0, scale_width);
                    }
                    
                    
                    },
                _ => (),
            },
                _ => (),
            
            
        }
        
        for keycode in pressed_keys.clone() {
            game_state.player.handle_key(keycode, delta_time, &bsp_tree);
        }
    }).unwrap();
}
