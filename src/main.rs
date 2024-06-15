use softbuffer::Surface;

use std::{collections::HashSet, num::NonZeroU32, rc::Rc, time::{Duration, Instant}};
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::{DeviceEvent, ElementState, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, keyboard::{Key, NamedKey}, window::{CursorGrabMode, Fullscreen, Window, WindowAttributes, WindowId}
};


mod assets;
mod render;
mod game_state;
mod util;
use game_state::GameState;

use crate::{ assets::{Linedef, Vertex, TextureId},  util::{render_text_to_buffer, game_setup}, render::{BSPTree, Renderer}};
struct App {
    window: Option<Rc<Window>>,
    renderer: Option<Renderer>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    game_state: Option<GameState>,
    last_fps_print_time : Instant,
    delta_time: f64,
    frames: u64,
    mouse_lock: bool,
    mouse_lock_setup: bool,
    pressed_keys: HashSet<Key>,
    frames_per_second: String,
    last_frame_time: Instant,
    bsp_tree: Option<BSPTree>,
    linedefs: Option<Vec<Linedef>>,
    vertices: Option<Vec<Vertex>>,
    render_screen_width: Option<usize>,
    render_screen_height: Option<usize>,
    scale_width: Option<usize>,
    scale_height: Option<usize>
}

impl Default for App {
    fn default() -> Self {
        Self { window: Default::default(), delta_time: 0.0, render_screen_height: Default::default(), render_screen_width: Default::default(), scale_height: Default::default(), scale_width: Default::default(), renderer: Default::default(), surface: Default::default(), vertices: Default::default(), game_state: Default::default(), linedefs: Default::default(), bsp_tree: Default::default(), frames: 0, last_fps_print_time: Instant::now(), mouse_lock: false, mouse_lock_setup: true, pressed_keys: HashSet::new(), frames_per_second: String::new(), last_frame_time: Instant::now()}
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        
        let monitor = event_loop.primary_monitor().unwrap();
        let monitor_mode = event_loop.primary_monitor().unwrap().video_modes().next().unwrap();
        let monitor_resolution = event_loop.primary_monitor().unwrap().size();
        let (monitor_width, monitor_height) = (monitor_resolution.width, monitor_resolution.height);
        let (
            render_screen_width,
            render_screen_height,
            scale_width,
            scale_height,
            thread_amount,
            _supersample_factor,
            is_fullscreen
        ) = game_setup(monitor_width, monitor_height);
        let display_size = PhysicalSize::new(scale_width as u32, scale_height as u32);
        rayon::ThreadPoolBuilder::new().num_threads(thread_amount as usize).build_global().unwrap();
        let atrributes = WindowAttributes::default()
            .with_title(format!("Multithread Raycaster Game Version: {}", clap::crate_version!()))
            .with_inner_size(display_size)
            .with_resizable(false);
        self.window = Some(Rc::new(event_loop.create_window(atrributes).unwrap()));
        let context = softbuffer::Context::new(<Option<Rc<Window>> as Clone>::clone(&self.window).unwrap().clone()).unwrap();
         self.surface = Some(softbuffer::Surface::new(&context, <Option<Rc<Window>> as Clone>::clone(&self.window).unwrap().clone()).unwrap());
        if is_fullscreen {
            if monitor_height as usize == scale_height && monitor_width as usize == scale_width {
                let fullscreen = Some(Fullscreen::Borderless(Some(monitor.clone())));
                self.window.as_ref().unwrap().set_fullscreen(fullscreen);
                
            } else {
                let fullscreen = Some(Fullscreen::Exclusive(monitor_mode.clone()));
                self.window.as_ref().unwrap().set_fullscreen(fullscreen);
            }
            
        }
        self.surface.as_mut().unwrap()
        .resize(
            NonZeroU32::new(scale_width as u32).unwrap(),
            NonZeroU32::new(scale_height as u32).unwrap(),
        )
        .unwrap();
        self.renderer = Some(Renderer::new(render_screen_width, render_screen_height));
        event_loop.set_control_flow(ControlFlow::Poll);
        let vertices = vec![
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
            Linedef { start: vertices[0], end: vertices[1], height: 1.0, floor_height: 0.0, ceiling_height: 1.0, texture: TextureId::BrickWalls, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[1], end: vertices[2], height: 2.0, floor_height: 0.0, ceiling_height: 2.0, texture: TextureId::RedBricks, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[2], end: vertices[3], height: 2.0, floor_height: 0.0, ceiling_height: 2.0, texture: TextureId::CorrodedCooperWall, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[3], end: vertices[4], height: 1.0, floor_height: 0.0, ceiling_height: 1.0, texture: TextureId::BrickWalls, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[4], end: vertices[5], height: 2.0, floor_height: 0.0, ceiling_height: 2.0, texture: TextureId::RedBricks, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[5], end: vertices[6], height: 2.0, floor_height: 0.0, ceiling_height: 2.0, texture: TextureId::CorrodedCooperWall, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[6], end: vertices[7], height: 1.0, floor_height: 0.0, ceiling_height: 1.0, texture: TextureId::BrickWalls, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[7], end: vertices[8], height: 3.0, floor_height: 0.0, ceiling_height: 3.0, texture: TextureId::RedBricks, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[8], end: vertices[9], height: 2.0, floor_height: 0.0, ceiling_height: 2.0, texture: TextureId::CorrodedCooperWall, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[9], end: vertices[10], height: 1.0, floor_height: 0.0, ceiling_height: 1.0, texture: TextureId::BrickWalls, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[10], end: vertices[11], height: 2.0, floor_height: 0.0, ceiling_height: 2.0, texture: TextureId::RedBricks, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
            Linedef { start: vertices[11], end: vertices[0], height: 2.0, floor_height: 0.0, ceiling_height: 2.0, texture: TextureId::BrickWalls, floor_texture: TextureId::BrickWalls, ceiling_texture: TextureId::BrickWalls, wrap_texture: true },
    /*         Linedef { start: vertices[1], end: vertices[2], height: 1.0, floor_level: 0.0, texture: TextureId::CorrodedCooperWall },
            Linedef { start: vertices[2], end: vertices[3], height: 2.0, floor_level: 0.0, texture: TextureId::GreyBricks },
            Linedef { start: vertices[3], end: vertices[4], height: 1.0, floor_level: 0.0, texture: TextureId::RedFlooring },
            Linedef { start: vertices[4], end: vertices[5], height: 2.0, floor_level: 0.0, texture: TextureId::RedBricks },
            Linedef { start: vertices[5], end: vertices[0], height: 1.0, floor_level: 0.0, texture: TextureId::Ceiling }, */
        ];
        self.game_state = Some(GameState::new());
        self.bsp_tree = Some(BSPTree::build(linedefs.clone()));
        self.linedefs = Some(linedefs);
        self.vertices = Some(vertices);
        self.render_screen_width = Some(render_screen_width);
        self.render_screen_height = Some(render_screen_height);
        self.scale_width = Some(scale_width);
        self.scale_height = Some(scale_height);
    }
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
        self.game_state.as_mut().unwrap().player.states.is_walking = false;
        for keycode in self.pressed_keys.clone() {
            self.game_state.as_mut().unwrap().player.handle_key(keycode, self.delta_time, &self.bsp_tree.as_ref().unwrap());
        }
    }
    fn device_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _device_id: winit::event::DeviceId,
            event: DeviceEvent,
        ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_lock {
                    let (x, y) = delta;
                    self.game_state.as_mut().unwrap().player.move_camera(x, y, self.scale_width.unwrap() as f64 * 4.0,  self.scale_height.unwrap() as f64 * 4.0, self.scale_width.unwrap());
                }
            }
            _ => {}
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                
                let current_frame_time = std::time::Instant::now();
                self.delta_time = (current_frame_time - self.last_frame_time).as_secs_f64().clone();
                self.last_frame_time = current_frame_time;
                self.game_state.as_mut().unwrap().player.update_vertical_position(self.delta_time, &self.linedefs.as_mut().unwrap());
                self.frames += 1;
                if self.last_fps_print_time.elapsed() > Duration::from_secs(1) {
                    self.frames_per_second = format!("FPS: {:.0}", self.frames as f64 / self.last_fps_print_time.elapsed().as_secs_f64());
                    self.last_fps_print_time = std::time::Instant::now();
                    self.frames = 0;
                    if self.mouse_lock_setup {
                        self.window.as_ref().unwrap().set_cursor_grab(CursorGrabMode::Confined).unwrap();
                        self.window.as_ref().unwrap().set_cursor_visible(false);
                        self.mouse_lock = true;
                        self.mouse_lock_setup = false;
                    }
                }
                self.renderer.as_mut().unwrap().raycast_and_render_polygons(&self.bsp_tree.as_ref().unwrap(), &self.game_state.as_ref().unwrap().player,&self.game_state.as_ref().unwrap().assets, self.render_screen_width.unwrap(), self.render_screen_height.unwrap());
                if !self.mouse_lock {
                    render_text_to_buffer(255, 75, 75, "Mouse Lock Off Press Alt To Enable",45.0, self.game_state.as_ref().unwrap().assets.fonts[0].clone(), (0.0, 0.0),&mut self.renderer.as_mut().unwrap().buffer,  self.scale_width.unwrap(), self.scale_height.unwrap(), );
                }
                render_text_to_buffer(255, 255, 255, &format!("{}", self.frames_per_second),35.0, self.game_state.as_ref().unwrap().assets.fonts[0].clone(), (self.render_screen_width.unwrap() as f32 * 0.85, 0.0),&mut self.renderer.as_mut().unwrap().buffer,  self.scale_width.unwrap(), self.scale_height.unwrap(), );
                let mut draw_buffer = self.surface.as_mut().unwrap().buffer_mut().unwrap();
                {
                    draw_buffer.copy_from_slice(&self.renderer.as_mut().unwrap().buffer)
                }
                draw_buffer.present().unwrap();
            },
            WindowEvent::KeyboardInput { event, .. } => {
                let (state, key) = (event.state, event.logical_key.clone());
                match event.logical_key {
                    Key::Named(NamedKey::Escape) => {
                        event_loop.exit();
                    },
                    Key::Named(NamedKey::Alt) => {
                        match state {
                            ElementState::Pressed => {
                                if self.mouse_lock {
                                    self.window.as_ref().unwrap().set_cursor_grab(CursorGrabMode::None).unwrap();
                                    self.window.as_ref().unwrap().set_cursor_visible(true);
                                    self.mouse_lock = false;
                                } else {
                                    self.window.as_ref().unwrap().set_cursor_grab(CursorGrabMode::Confined).unwrap();
                                    self.window.as_ref().unwrap().set_cursor_visible(false);
                                    self.mouse_lock = true;
                                }
                            }
                            _ => {}
                        }
                            
                    },
                    _ => {}

                }
                match state {
                    ElementState::Pressed => {
                        self.pressed_keys.insert(key);
                    }
                    ElementState::Released => {
                        self.pressed_keys.remove(&key);
                    }
                }
            }   
                    
            _ => {}
            
    }
}
}
fn main() {
    let event_loop = EventLoop::builder().build().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
