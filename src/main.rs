#![feature(portable_simd)]
use render::Renderer;
use softbuffer::Surface;

use std::{collections::HashSet, num::NonZeroU32, rc::Rc, time::{Duration, Instant}};
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::{DeviceEvent, ElementState, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, keyboard::{Key, NamedKey}, window::{CursorGrabMode, Window, WindowAttributes, WindowId}
};
use state::State;
use voxel::{SparseVoxelOctree, Voxel, VoxelColor};

mod state;
mod voxel;
mod render;
struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    last_fps_print_time : Instant,
    delta_time: f64,
    frames: u64,
    mouse_lock: bool,
    mouse_lock_setup: bool,
    pressed_keys: HashSet<Key>,
    frames_per_second: String,
    last_frame_time: Instant,
    state: State,
    svo: SparseVoxelOctree,
    renderer: Option<Renderer>,
    threads: usize,
}
impl App {
    
}
impl Default for App {
    fn default() -> Self {
        Self { window: Default::default(), delta_time: 0.0, surface: Default::default(), frames: 0, last_fps_print_time: Instant::now(), mouse_lock: false, mouse_lock_setup: true, pressed_keys: HashSet::new(), frames_per_second: String::new(), last_frame_time: Instant::now(), state: State::new(), svo: SparseVoxelOctree::new(), renderer: Default::default(), threads: 3}
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let monitor_resolution = event_loop.primary_monitor().unwrap().size();
        let (monitor_width, monitor_height) = (monitor_resolution.width, monitor_resolution.height);

        let display_size = PhysicalSize::new(monitor_width, monitor_height);
        let atrributes = WindowAttributes::default()
            .with_title(format!("Multithread Raycaster Game Version: {}", clap::crate_version!()))
            .with_inner_size(display_size)
            .with_resizable(false);
        self.window = Some(Rc::new(event_loop.create_window(atrributes).unwrap()));
        let context = softbuffer::Context::new(<Option<Rc<Window>> as Clone>::clone(&self.window).unwrap().clone()).unwrap();
         self.surface = Some(softbuffer::Surface::new(&context, <Option<Rc<Window>> as Clone>::clone(&self.window).unwrap().clone()).unwrap());
        self.surface.as_mut().unwrap()
        .resize(
            NonZeroU32::new(monitor_width).unwrap(),
            NonZeroU32::new(monitor_height).unwrap(),
        )
        .unwrap();
        
        self.renderer = Some(Renderer::new(monitor_width as usize, monitor_height as usize, self.threads));

        event_loop.set_control_flow(ControlFlow::Poll);
        self.svo.insert(10, 20, 30, Voxel { color: VoxelColor {r: 255, g: 0, b: 0, a: 255} });
        let color_pallete = &self.state.assets.voxels[0].palette;
        for voxel_input in self.state.assets.voxels[0].models[0].voxels.clone().into_iter(){
            let voxel_color = color_pallete[voxel_input.i as usize];
            let voxel = Voxel {
                color: VoxelColor {r: voxel_color.r, g: voxel_color.g, b: voxel_color.b, a: voxel_color.a}
            };
            self.svo.insert(voxel_input.x as usize, voxel_input.y as usize, voxel_input.z as usize, voxel)
        }
    }
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
    }
    fn device_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _device_id: winit::event::DeviceId,
            event: DeviceEvent,
        ) {
        match event {
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
                self.frames += 1;
                if self.last_fps_print_time.elapsed() > Duration::from_secs(1) {
                    self.frames_per_second = format!("FPS: {:.0}", self.frames as f64 / self.last_fps_print_time.elapsed().as_secs_f64());
                    self.last_fps_print_time = std::time::Instant::now();
                    self.frames = 0;

                    self.window.as_ref().unwrap().set_title(&self.frames_per_second);
                    if self.mouse_lock_setup {
                        self.window.as_ref().unwrap().set_cursor_grab(CursorGrabMode::Confined).unwrap();
                        self.window.as_ref().unwrap().set_cursor_visible(false);
                        self.mouse_lock = true;
                        self.mouse_lock_setup = false;
                    }
                }
                if !self.mouse_lock {
                }
                Renderer::render_voxels(self.renderer.as_mut().unwrap(), &self.svo, self.threads);

                if let Ok(mut draw_buffer) = self.surface.as_mut().unwrap().buffer_mut() {
                    // Flatten the 2D buffer efficiently
                    for (i, row) in self.renderer.as_ref().unwrap().buffer.iter().enumerate() {
                        draw_buffer[i * self.renderer.as_ref().unwrap().render_width..(i + 1) * self.renderer.as_ref().unwrap().render_width].copy_from_slice(row);
                    }
                    draw_buffer.present().unwrap();
                }
                puffin::GlobalProfiler::lock().new_frame();
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
    let server_addr = format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT);
    let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();
    eprintln!("Serving demo profile data on {server_addr}. Run `puffin_viewer` to view it.");
    puffin::set_scopes_on(false);
    event_loop.run_app(&mut app).unwrap();
}
