use profiling::tracy_client;
use rayon::prelude::*;
use std::{time::Duration, collections::HashSet, sync::atomic::{AtomicPtr, Ordering as Order}, ptr};
use softbuffer::GraphicsContext;
use winit::{
    event::{Event, DeviceEvent, WindowEvent, KeyboardInput, VirtualKeyCode /*, ModifiersState*/, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, CursorGrabMode}, dpi::PhysicalSize,
};
use image::io::Reader as ImageReader;
use image::Pixel;

// use tracing::{info, Level};
// use tracing_subscriber::FmtSubscriber;
struct Sprite {
    x: f64,
    y: f64,
    texture: usize
}
const NUM_SPRITES: usize = 19;
const SPRITE: [Sprite; NUM_SPRITES] = [
    Sprite {x: 20.5, y: 11.5, texture: 10},
    Sprite {x: 18.5, y: 4.5, texture: 10},
    Sprite {x: 10.0, y: 4.5, texture: 10},
    Sprite {x: 10.0, y: 12.5, texture: 10},
    Sprite {x: 3.5, y: 6.5, texture: 10},
    Sprite {x: 3.5, y: 20.5, texture: 10},
    Sprite {x: 3.5, y: 14.5, texture: 10},
    Sprite {x: 14.5, y: 20.5, texture: 10},
    Sprite {x: 18.5, y: 10.5, texture: 9},
    Sprite {x: 18.5, y: 11.5, texture: 9},
    Sprite {x: 18.5, y: 12.5, texture: 9},
    Sprite {x: 21.5, y: 1.5, texture: 8},
    Sprite {x: 15.5, y: 1.5, texture: 8},
    Sprite {x: 16.0, y: 1.8, texture: 8},
    Sprite {x: 16.2, y: 1.2, texture: 8},
    Sprite {x: 3.5, y: 2.5, texture: 8},
    Sprite {x: 9.5, y: 15.5, texture: 8},
    Sprite {x: 10.0, y: 15.1, texture: 8},
    Sprite {x: 10.5, y: 15.8, texture: 8},
    ];
const RENDER_SCREEN_HEIGHT: usize = 1080;
const RENDER_SCREEN_WIDTH: usize = 1920;

const MAP_WIDTH: usize = 24;
const MAP_HEIGHT: usize = 24;
const TEX_WIDTH: u32 = 64;
const TEX_HEIGHT: u32 = 64;
const  WORLD_MAP: [[usize; MAP_WIDTH]; MAP_HEIGHT] =
[
    [8,8,8,8,8,8,8,8,8,8,8,4,4,6,4,4,6,4,6,4,4,4,6,4],
    [8,0,0,0,0,0,0,0,0,0,8,4,0,0,0,0,0,0,0,0,0,0,0,4],
    [8,0,3,3,0,0,0,0,0,8,8,4,0,0,0,0,0,0,0,0,0,0,0,6],
    [8,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6],
    [8,0,3,3,0,0,0,0,0,8,8,4,0,0,0,0,0,0,0,0,0,0,0,4],
    [8,0,0,0,0,0,0,0,0,0,8,4,0,0,0,0,0,6,6,6,0,6,4,6],
    [8,8,8,8,0,8,8,8,8,8,8,4,4,4,4,4,4,6,0,0,0,0,0,6],
    [7,7,7,7,0,7,7,7,7,0,8,0,8,0,8,0,8,4,0,4,0,6,0,6],
    [7,7,0,0,0,0,0,0,7,8,0,8,0,8,0,8,8,6,0,0,0,0,0,6],
    [7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,6,0,0,0,0,0,4],
    [7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,6,0,6,0,6,0,6],
    [7,7,0,0,0,0,0,0,7,8,0,8,0,8,0,8,8,6,4,6,0,6,6,6],
    [7,7,7,7,0,7,7,7,7,8,8,4,0,6,8,4,8,3,3,3,0,3,3,3],
    [2,2,2,2,0,2,2,2,2,4,6,4,0,0,6,0,6,3,0,0,0,0,0,3],
    [2,2,0,0,0,0,0,2,2,4,0,0,0,0,0,0,4,3,0,0,0,0,0,3],
    [2,0,0,0,0,0,0,0,2,4,0,0,0,0,0,0,4,3,0,0,0,0,0,3],
    [1,0,0,0,0,0,0,0,1,4,4,4,4,4,6,0,6,3,3,0,0,0,3,3],
    [2,0,0,0,0,0,0,0,2,2,2,1,2,2,2,6,6,0,0,5,0,5,0,5],
    [2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,0,5,0,5,0,0,0,5,5],
    [2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5],
    [2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
    [2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,0,5,0,5,0,0,0,5,5],
    [2,2,2,2,1,2,2,2,2,2,2,1,2,2,2,5,5,5,5,5,5,5,5,5]
];

fn main() {
    tracy_client::Client::start();
    profiling::register_thread!("Main Thread");
    use tracing_subscriber::layer::SubscriberExt;
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(tracing_tracy::TracyLayer::new()),
        )
        .unwrap();  
    let event_loop = EventLoop::new();
    let display_size = PhysicalSize::new(RENDER_SCREEN_WIDTH as u32, RENDER_SCREEN_HEIGHT as u32);
    let window = WindowBuilder::new().with_inner_size(display_size).with_resizable(true).build(&event_loop).unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();
    let start_time = std::time::Instant::now();
    let mut frames = 0;
    let mut last_fps_print_time = start_time;
    let mut previous_frame_time = std::time::Instant::now();
    
    let mut mouse_lock = false;
    let mut init = 0;
    let mut texture: [[u32; (TEX_WIDTH * TEX_HEIGHT) as usize]; 11] = [[0; (TEX_WIDTH * TEX_HEIGHT) as usize]; 11];
    {
        let bricks_texture = ImageReader::open("assets/textures/brick_walls.png").unwrap().decode().unwrap();
        for (texture_x, texture_y, pixel) in bricks_texture.as_rgb8().unwrap().enumerate_pixels() {
            let pixel_colors = pixel.channels();
            let (r, g, b) = (pixel_colors[0] as u32, pixel_colors[1] as u32, pixel_colors[2] as u32);
            texture[0][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
            texture[2][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
            texture[3][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
        }
    }
    {
        let red_bricks_texture = ImageReader::open("assets/textures/red_bricks.png").unwrap().decode().unwrap();
        for (texture_x, texture_y, pixel) in red_bricks_texture.as_rgb8().unwrap().enumerate_pixels() {
            let pixel_colors = pixel.channels();
            let (r, g, b) = (pixel_colors[0] as u32, pixel_colors[1] as u32, pixel_colors[2] as u32);
            texture[4][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
            texture[5][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
            texture[6][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
        }
    }
    {
        let red_flooring_texture = ImageReader::open("assets/textures/red_flooring.png").unwrap().decode().unwrap();
        for (texture_x, texture_y, pixel) in red_flooring_texture.as_rgb8().unwrap().enumerate_pixels() {
            let pixel_colors = pixel.channels();
            let (r, g, b) = (pixel_colors[0] as u32, pixel_colors[1] as u32, pixel_colors[2] as u32);
            texture[7][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
        }
    }
    {
        let ceiling_texture = ImageReader::open("assets/textures/ceiling.png").unwrap().decode().unwrap();
        for (texture_x, texture_y, pixel) in ceiling_texture.as_rgb8().unwrap().enumerate_pixels() {
            let pixel_colors = pixel.channels();
            let (r, g, b) = (pixel_colors[0] as u32, pixel_colors[1] as u32, pixel_colors[2] as u32);
            texture[1][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
        }
    }
    {
        let barrel_texture = ImageReader::open("assets/textures/barrel.png").unwrap().decode().unwrap();
        for (texture_x, texture_y, pixel) in barrel_texture.as_rgb8().unwrap().enumerate_pixels() {
            let pixel_colors = pixel.channels();
            let (r, g, b) = (pixel_colors[0] as u32, pixel_colors[1] as u32, pixel_colors[2] as u32);
            texture[8][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
        }
    }
    {
        let pillar_texture = ImageReader::open("assets/textures/pillar.png").unwrap().decode().unwrap();
        for (texture_x, texture_y, pixel) in pillar_texture.as_rgb8().unwrap().enumerate_pixels() {
            let pixel_colors = pixel.channels();
            let (r, g, b) = (pixel_colors[0] as u32, pixel_colors[1] as u32, pixel_colors[2] as u32);
            texture[9][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
        }
    }
    {
        let light_texture = ImageReader::open("assets/textures/light.png").unwrap().decode().unwrap();
        for (texture_x, texture_y, pixel) in light_texture.as_rgb8().unwrap().enumerate_pixels() {
            let pixel_colors = pixel.channels();
            let (r, g, b) = (pixel_colors[0] as u32, pixel_colors[1] as u32, pixel_colors[2] as u32);
            texture[10][(TEX_WIDTH * texture_y + texture_x) as usize] = (r << 16) | (g << 8) | b;
        }
    }
    let (mut pos_x, mut pos_y) = (22.0, 11.5); 
    let (mut dir_x, mut dir_y) = (-1.0, 0.0);
    let (mut plane_x, mut plane_y) = (0.0, 0.66);
    #[allow(unused_assignments)]
    let mut movespeed = 0.05;
    let mut screen_pitch = 0.0;
    let mut pos_z = 0.0;
    let mut is_jumping = false;
    let mut vertical_velocity = 0.0;
    let mut last_update = std::time::Instant::now();
    
    rayon::ThreadPoolBuilder::new().num_threads(12).build_global().unwrap();
    
    let mut pressed_keys = HashSet::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let current_frame_time = std::time::Instant::now();
        let frame_duration = current_frame_time.duration_since(previous_frame_time);
        previous_frame_time = current_frame_time;
        let frame_time_ms = frame_duration.as_millis();
        
        movespeed = (frame_time_ms as f64 / 1000.0) * 3.5;   
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                frames += 1;
                if last_fps_print_time.elapsed() > Duration::from_secs(1) {
                    println!("FPS: {:.1}", frames as f64 / last_fps_print_time.elapsed().as_secs_f64());
                    last_fps_print_time = std::time::Instant::now();
                    frames = 0;
                }
                let half_screen_height = RENDER_SCREEN_HEIGHT as f64 / 2.0;
                let ray_dir_x0 = dir_x - plane_x;
                let ray_dir_y0 = dir_y - plane_y;
                let ray_dir_x1 = dir_x + plane_x;
                let ray_dir_y1 = dir_y + plane_y;
                let ceiling_floor_buffer = ceiling_buffer(screen_pitch, half_screen_height, pos_z, ray_dir_x1, ray_dir_x0, ray_dir_y1, ray_dir_y0, pos_x, pos_y, texture);
                // let ceiling_floor_buffer: Vec<Vec<u32>> = (0..RENDER_SCREEN_HEIGHT).into_par_iter().map(|y| {
                //     let is_floor = y as isize > RENDER_SCREEN_HEIGHT as isize / 2 + screen_pitch as isize;
                    
                    
                //     let ray_dir_x0 = dir_x - plane_x;
                //     let ray_dir_y0 = dir_y - plane_y;
                //     let ray_dir_x1 = dir_x + plane_x;
                //     let ray_dir_y1 = dir_y + plane_y;
                
                //     // Current y position compared to the center of the screen (the horizon)
                //     let p  = if is_floor {
                //         (y as isize) - (RENDER_SCREEN_HEIGHT as isize) / 2 - screen_pitch as isize
                //     } else {
                //         RENDER_SCREEN_HEIGHT as isize / 2 - y as isize + screen_pitch as isize
                //     };
                
                //     // Vertical position of the camera.
                //     let cam_z = if is_floor { 0.5 * RENDER_SCREEN_HEIGHT as f64 + pos_z.clone()} else { 0.5 * RENDER_SCREEN_HEIGHT as f64 - pos_z.clone()};
                
                //     // Horizontal distance from the camera to the floor for the current row.
                //     // 0.5 is the z position exactly in the middle between floor and ceiling.
                //     let row_distance = cam_z / (p as f64);
                
                //     let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / (RENDER_SCREEN_WIDTH as f64);
                //     let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / (RENDER_SCREEN_WIDTH as f64);
                
                //     let mut floor_x = pos_x + row_distance * ray_dir_x0;
                //     let mut floor_y = pos_y + row_distance * ray_dir_y0;
                
                
                //     let mut row_buffer: [u32; RENDER_SCREEN_WIDTH as usize] = [0; RENDER_SCREEN_WIDTH as usize];
                
                //     for x in 0..RENDER_SCREEN_WIDTH {
                //         let tx = ((TEX_WIDTH as f64) * floor_x.fract()) as isize & (TEX_WIDTH - 1) as isize;
                //         let ty = ((TEX_HEIGHT as f64) * floor_y.fract()) as isize & (TEX_HEIGHT - 1) as isize;
                
                //         floor_x += floor_step_x;
                //         floor_y += floor_step_y;
                
                //         let floor_texture = 7;
                //         let ceiling_texture = 1;
                //         if is_floor {
                //             // floor
                //             let color = texture[floor_texture][(TEX_WIDTH as isize * ty + tx) as usize];
                //             row_buffer[x as usize] = color;
                //         } else {
                //             //ceiling
                //             let color = texture[ceiling_texture][(TEX_WIDTH as isize * ty + tx) as usize];
                //             row_buffer[x as usize] = color;
                //         }
                //     }
                //     row_buffer.to_vec()
                // }).collect();
                let (wall_buffer, z_buffer) = wall_buffer(dir_x, plane_x, dir_y, plane_y, pos_x, pos_y, screen_pitch, pos_z, texture);
                
               let mut transposed_wall_buffer: Vec<Vec<u32>> = vec![vec![0; RENDER_SCREEN_WIDTH]; RENDER_SCREEN_HEIGHT];

                let transposed_wall_buffer_atomic: Vec<AtomicPtr<u32>> = transposed_wall_buffer
                .iter_mut()
                .map(|row| AtomicPtr::new(row.as_mut_ptr()))
                .collect();

                (0..RENDER_SCREEN_HEIGHT).into_par_iter().for_each(|y| {
                    let row_ptr = transposed_wall_buffer_atomic[y as usize].load(Order::Relaxed);
            
                    let mut x = 0;
                    while x < RENDER_SCREEN_WIDTH {
                        unsafe {
                            ptr::write(row_ptr.add(x), *wall_buffer[x as usize].get_unchecked(y));
                            ptr::write(row_ptr.add(x + 1), *wall_buffer[(x + 1) as usize].get_unchecked(y));
                            ptr::write(row_ptr.add(x + 2), *wall_buffer[(x + 2) as usize].get_unchecked(y));
                            ptr::write(row_ptr.add(x + 3), *wall_buffer[(x + 3) as usize].get_unchecked(y));
                        }
                        x += 4;
                    }
                });
                let sprite_data: Vec<(usize, f64)> = (0..NUM_SPRITES).into_par_iter().map(|index| {
                    let order = index;
                    let dist = (pos_x - SPRITE[index].x) * (pos_x - SPRITE[index].x) + (pos_y - SPRITE[index].y) * (pos_y - SPRITE[index].y);
                    (order, dist)
                }).collect();

                let mut sprite_data = sprite_data;
                sprite_data.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                let sprite_order: Vec<usize> = sprite_data.into_iter().map(|(order, _)| order).collect();

                let sprite_buffer = sprite_buffer(sprite_order, pos_x, pos_y, plane_x, dir_y, dir_x, plane_y, screen_pitch, pos_z, z_buffer, texture);

                let mut transposed_sprite_buffer: Vec<Vec<u32>> = vec![vec![0; RENDER_SCREEN_WIDTH]; RENDER_SCREEN_HEIGHT];

                let transposed_sprite_buffer_atomic: Vec<AtomicPtr<u32>> = transposed_sprite_buffer
                .iter_mut()
                .map(|row| AtomicPtr::new(row.as_mut_ptr()))
                .collect();

                (0..RENDER_SCREEN_HEIGHT).into_par_iter().for_each(|y| {
                    let row_ptr = transposed_sprite_buffer_atomic[y as usize].load(Order::Relaxed);
            
                    let mut x = 0;
                    while x < RENDER_SCREEN_WIDTH {
                        unsafe {
                            ptr::write(row_ptr.add(x), *sprite_buffer[x as usize].get_unchecked(y));
                            ptr::write(row_ptr.add(x + 1), *sprite_buffer[(x + 1) as usize].get_unchecked(y));
                            ptr::write(row_ptr.add(x + 2), *sprite_buffer[(x + 2) as usize].get_unchecked(y));
                            ptr::write(row_ptr.add(x + 3), *sprite_buffer[(x + 3) as usize].get_unchecked(y));
                        }
                        x += 4;
                    }
                });
                //info!("Finish Transpose Sprite Buffer");



                let final_buffer = create_final_buffer(transposed_wall_buffer, ceiling_floor_buffer, transposed_sprite_buffer);
                let buffer: Vec<u32> = final_buffer.iter().flatten().cloned().collect();
                //info!("Finish Assemblying final buffer");
                graphics_context.set_buffer(&buffer, RENDER_SCREEN_WIDTH as u16, RENDER_SCREEN_HEIGHT as u16);
                //info!("Set Buffer");
                if init < 3 {
                    init += 1
                } else if init == 3 {
                    window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                    window.set_cursor_visible(false);
                    init = 4;
                    // *control_flow = ControlFlow::Exit
                }
                if is_jumping {
                let now = std::time::Instant::now();
                let delta_time = (now - last_update).as_secs_f64() * 3.0;
                last_update = now;
                pos_z += vertical_velocity * delta_time;
                vertical_velocity -= 50.0 * delta_time;
                if pos_z >= 200.0 {
                    pos_z = 200.0;
                    vertical_velocity = -300.0;
                } else if pos_z <= 0.0 {
                    pos_z = 0.0;
                    is_jumping = false;
                    vertical_velocity = 0.0;
                }
                }

                window.request_redraw()
            }
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
                    let x = delta.0;
                    let old_dir_x = dir_x;
                    dir_x = dir_x * (-x / 300.0).cos() - dir_y * (-x / 300.0).sin();
                    dir_y = old_dir_x * (-x / 300.0).sin() + dir_y * (-x / 300.0).cos();
                    let old_plane_x = plane_x;
                    plane_x = plane_x * (-x / 300.0).cos() - plane_y * (-x / 300.0).sin();
                    plane_y = old_plane_x * (-x / 300.0).sin() + plane_y * (-x / 300.0).cos();
                    let y = delta.1;
                    screen_pitch -= y;
                    screen_pitch = f64::clamp(screen_pitch, -560.0, 560.0);
                    
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
                        if WORLD_MAP[(pos_x + dir_x * movespeed) as usize][(pos_y) as usize] == 0 {pos_x += dir_x * movespeed};
                        if WORLD_MAP[(pos_x) as usize][(pos_y + dir_y * movespeed) as usize] == 0 {pos_y += dir_y * movespeed};
                    },
                    VirtualKeyCode::A => {
                        if WORLD_MAP[(pos_x - plane_x * movespeed) as usize][(pos_y) as usize] == 0 {pos_x -= plane_x * movespeed};
                        if WORLD_MAP[(pos_x) as usize][(pos_y - plane_y * movespeed) as usize] == 0 {pos_y -= plane_y * movespeed};
                    },
                    VirtualKeyCode::S => {
                        if WORLD_MAP[(pos_x - dir_x * movespeed) as usize][(pos_y) as usize] == 0 {pos_x -= dir_x * movespeed};
                        if WORLD_MAP[(pos_x) as usize][(pos_y - dir_y * movespeed) as usize] == 0 {pos_y -= dir_y * movespeed};
                    },
                    VirtualKeyCode::D => {
                        if WORLD_MAP[(pos_x + plane_x * movespeed) as usize][(pos_y) as usize] == 0 {pos_x += plane_x * movespeed};
                        if WORLD_MAP[(pos_x) as usize][(pos_y + plane_y * movespeed) as usize] == 0 {pos_y += plane_y * movespeed};
                    },
                    VirtualKeyCode::LControl => {
                        pos_z = -200.0
                    },
                    VirtualKeyCode::Space => {
                        if !is_jumping {
                            is_jumping = true;
                            vertical_velocity = 300.0;
                        }
                    },
                    
                    
                    _ => continue
                }
            }
    });
}

fn create_final_buffer(transposed_wall_buffer: Vec<Vec<u32>>, ceiling_floor_buffer: Vec<Vec<u32>>, transposed_sprite_buffer: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
    //info!("Start Assemblying final buffer");
    let wall_floor_cel: Vec<Vec<u32>> = transposed_wall_buffer
        .into_par_iter()
        .zip(ceiling_floor_buffer)
        .map(|(wall_vec, floor_ceiling_vec)| {
            let mut draw_buffer: Vec<u32> = Vec::with_capacity(wall_vec.len());
            draw_buffer.extend_from_slice(&wall_vec);

            for (i, (value_wall, value_floor)) in wall_vec.iter().zip(floor_ceiling_vec).enumerate() {
                if *value_wall == 0 {
                    draw_buffer[i] = value_floor;
                }
            }

            draw_buffer
        })
        .collect();
    let final_buffer: Vec<Vec<u32>> = transposed_sprite_buffer
        .into_par_iter()
        .zip(wall_floor_cel)
        .map(|(sprite_vec, wall_floor_vec)| {
            let mut draw_buffer: Vec<u32> = Vec::with_capacity(sprite_vec.len());
            draw_buffer.extend_from_slice(&sprite_vec);

            for (i, (value_floor, value_wall)) in sprite_vec.iter().zip(wall_floor_vec).enumerate() {
                if *value_floor == 0 {
                    draw_buffer[i] = value_wall;
                }
            }

            draw_buffer
        })
        .collect();
    final_buffer
}

fn sprite_buffer(sprite_order: Vec<usize>, pos_x: f64, pos_y: f64, plane_x: f64, dir_y: f64, dir_x: f64, plane_y: f64, screen_pitch: f64, pos_z: f64, z_buffer: Vec<f64>, texture: [[u32; 4096]; 11]) -> Vec<Vec<u32>> {
    let sprite_buffer: Vec<Vec<u32>> = (0..RENDER_SCREEN_WIDTH).into_par_iter().map(|x| {
        let mut column_buffer = vec![0; RENDER_SCREEN_HEIGHT];

        for index in 0..NUM_SPRITES {
            let sprite_x = SPRITE[sprite_order[index]].x - pos_x;
            let sprite_y = SPRITE[sprite_order[index]].y - pos_y;

            let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);

            let transform_x = inv_det * (dir_y * sprite_x - dir_x * sprite_y);
            let transform_y = inv_det * (-plane_y * sprite_x + plane_x * sprite_y);

            let sprite_screen_x = ((RENDER_SCREEN_WIDTH / 2) as f64 * (1.0 + transform_x / transform_y)) as isize;

            const U_DIV: isize = 1;
            const V_DIV: isize = 1;
            const V_MOVE: f64 = 0.0;
            let v_move_screen = ((V_MOVE / transform_y) + screen_pitch + pos_z / transform_y) as isize;

            let sprite_height = ((RENDER_SCREEN_HEIGHT as f64 / (transform_y)).abs()) as isize / V_DIV;
            let mut draw_start_y = -sprite_height / 2 + RENDER_SCREEN_HEIGHT as isize / 2 + v_move_screen;
            if draw_start_y < 0 {
                draw_start_y = 0
            };
            let mut draw_end_y = sprite_height / 2 + RENDER_SCREEN_HEIGHT as isize / 2 + v_move_screen;
            if draw_end_y >= RENDER_SCREEN_HEIGHT as isize {
                draw_end_y = RENDER_SCREEN_HEIGHT as isize - 1
            };

            let sprite_width = ((RENDER_SCREEN_HEIGHT as f64 / (transform_y)).abs()) as isize / U_DIV;
            let mut draw_start_x = -sprite_width / 2 + sprite_screen_x;
            if draw_start_x < 0 {
                draw_start_x = 0
            };
            let mut draw_end_x = sprite_width / 2 + sprite_screen_x;
            if draw_end_x > RENDER_SCREEN_WIDTH as isize {
                draw_end_x = RENDER_SCREEN_WIDTH as isize
            };

            if x as isize >= draw_start_x && (x as isize) < draw_end_x {
                let stripe = x as isize;
                let tex_x = (256 * (stripe - (-sprite_width / 2 + sprite_screen_x)) * TEX_WIDTH as isize / sprite_width) / 256;
                if transform_y > 0.0 && transform_y < z_buffer[stripe as usize] {
                    for y in draw_start_y..draw_end_y {
                        let d = (y - v_move_screen) * 256 - RENDER_SCREEN_HEIGHT as isize * 128 + sprite_height * 128;
                        let tex_y = ((d * TEX_HEIGHT as isize) / sprite_height) / 256;
                        let color = texture[SPRITE[sprite_order[index]].texture][(TEX_WIDTH as isize * tex_y + tex_x) as usize];
                        if (color & 0x00FFFFFF) != 0 {
                            column_buffer[y as usize] = color;
                        }
                    }
                }
            }
        }

        column_buffer
    }).collect();
    sprite_buffer
}

fn wall_buffer(dir_x: f64, plane_x: f64, dir_y: f64, plane_y: f64, pos_x: f64, pos_y: f64, screen_pitch: f64, pos_z: f64, texture: [[u32; 4096]; 11]) -> (Vec<Vec<u32>>, Vec<f64>) {
    let (wall_buffer, z_buffer): (Vec<Vec<u32>>, Vec<f64>) = (0..RENDER_SCREEN_WIDTH).into_par_iter().map(|x| {
        let camera_x: f64 = 2.0 * x as f64 / RENDER_SCREEN_WIDTH as f64 - 1.0;
        let ray_dir_x: f64 = dir_x + plane_x * camera_x;
        let ray_dir_y: f64 = dir_y + plane_y * camera_x;

        let (mut map_x, mut map_y) = (pos_x as isize, pos_y as isize);

        let (mut side_dist_x, mut side_dist_y): (f64, f64);

        let delta_dist_x = (1.0 / ray_dir_x).abs();
        let delta_dist_y = (1.0 / ray_dir_y).abs();

        let perp_wall_dist: f64;

        let (step_x, step_y): (isize, isize);

        let mut wall_hit: bool = false;

        let mut side_wall: bool = false;

        if ray_dir_x < 0.0 {
            step_x = -1;
            side_dist_x = (pos_x - map_x as f64) * delta_dist_x;
        } else {
            step_x = 1;
            side_dist_x = (map_x as f64 + 1.0 - pos_x) * delta_dist_x;
        }
        if ray_dir_y < 0.0 {
            step_y = -1;
            side_dist_y = (pos_y - map_y as f64) * delta_dist_y;
        } else {
            step_y = 1;
            side_dist_y = (map_y as f64 + 1.0 - pos_y) * delta_dist_y;
        }

        while !wall_hit {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                side_wall = false;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                side_wall = true;
            }
            if WORLD_MAP[map_x as usize][map_y as usize] > 0 {
                wall_hit = true;
            }
        }
        if !side_wall {
            perp_wall_dist = side_dist_x - delta_dist_x;
        } else {
            perp_wall_dist = side_dist_y - delta_dist_y;
        };

        let line_height = RENDER_SCREEN_HEIGHT as f64 / perp_wall_dist;

        let mut draw_start = -line_height as i32 / 2 + RENDER_SCREEN_HEIGHT as i32 / 2 + screen_pitch as i32 + (pos_z.clone() / perp_wall_dist) as i32;
        if draw_start < 0 {
            draw_start = 0;
        };
        let mut draw_end = line_height as i32 / 2 + RENDER_SCREEN_HEIGHT as i32 / 2 + screen_pitch as i32 + (pos_z.clone() / perp_wall_dist) as i32;
        if draw_end >= RENDER_SCREEN_HEIGHT as i32 {
            draw_end = RENDER_SCREEN_HEIGHT as i32 - 1;
        };

        let tex_num = WORLD_MAP[map_x as usize][map_y as usize] - 1;

        let mut wall_x: f64;
        if side_wall == false {
            wall_x = pos_y + perp_wall_dist * ray_dir_y;
        } else {
            wall_x = pos_x + perp_wall_dist * ray_dir_x;
        }
        wall_x -= wall_x.floor();

        let mut tex_x = (wall_x * (TEX_WIDTH) as f64) as isize;
        if side_wall == false && ray_dir_x > 0.0 {
    
            tex_x = (TEX_WIDTH as isize) - tex_x - 1;
        }
        if side_wall == true && ray_dir_y < 0.0 {
            tex_x = (TEX_WIDTH as isize) - tex_x - 1;
        }

        let step = 1.0 * (TEX_HEIGHT as f64) / line_height;
        let mut tex_pos = ((draw_start as f64) - screen_pitch - (pos_z.clone() / perp_wall_dist) - (RENDER_SCREEN_HEIGHT as f64) / 2.0 + line_height / 2.0) * step;

        let mut col_buffer: [u32; RENDER_SCREEN_HEIGHT as usize] = [0; RENDER_SCREEN_HEIGHT as usize];

        for position_y in (draw_start as u32)..(draw_end as u32) {
            let tex_y = (tex_pos as isize) & ((TEX_HEIGHT as isize) - 1);
            tex_pos += step;
            let color = texture[tex_num as usize][((TEX_HEIGHT as isize) * tex_y + tex_x) as usize];
            if side_wall == true {
                /*let temp_color = Color::from_u32(&PixelFormat::try_from(PixelFormatEnum::RGBA8888).unwrap(), color);
                            let (r, g, b, a) = temp_color.rgba();
                            color = Color::to_u32(Color::RGBA(r / 2, g / 2, b /2, a), &PixelFormat::try_from(PixelFormatEnum::RGBA8888).unwrap());
                            */
            
            }
            col_buffer[position_y as usize] = color;
        }
        (col_buffer.to_vec(), perp_wall_dist)
    }).collect();
    (wall_buffer, z_buffer)
}

fn ceiling_buffer(screen_pitch: f64, half_screen_height: f64, pos_z: f64, ray_dir_x1: f64, ray_dir_x0: f64, ray_dir_y1: f64, ray_dir_y0: f64, pos_x: f64, pos_y: f64, texture: [[u32; 4096]; 11]) -> Vec<Vec<u32>> {
    let ceiling_floor_buffer: Vec<Vec<u32>> = (0..RENDER_SCREEN_HEIGHT).into_par_iter().map(|y| {
        let is_floor = y as isize > RENDER_SCREEN_HEIGHT as isize / 2 + screen_pitch as isize;
        let p  = if is_floor {
            (y as isize) - (RENDER_SCREEN_HEIGHT as isize) / 2 - screen_pitch as isize
        } else {
            RENDER_SCREEN_HEIGHT as isize / 2 - y as isize + screen_pitch as isize
        };
        let cam_z = if is_floor { half_screen_height + pos_z.clone() } else { half_screen_height - pos_z.clone()};
        let row_distance = cam_z / (p as f64);
        let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / (RENDER_SCREEN_WIDTH as f64);
        let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / (RENDER_SCREEN_WIDTH as f64);
        let mut floor_x = pos_x + row_distance * ray_dir_x0;
        let mut floor_y = pos_y + row_distance * ray_dir_y0;

        (0..RENDER_SCREEN_WIDTH).map(|_x| {
            let tx = ((TEX_WIDTH as f64) * floor_x.fract()) as isize & (TEX_WIDTH - 1) as isize;
            let ty = ((TEX_HEIGHT as f64) * floor_y.fract()) as isize & (TEX_HEIGHT - 1) as isize;
            floor_x += floor_step_x;
            floor_y += floor_step_y;
            if is_floor {
                // floor
                texture[7][(TEX_WIDTH as isize * ty + tx) as usize]
            } else {
                //ceiling
                texture[1][(TEX_WIDTH as isize * ty + tx) as usize]
            }
        }).collect()
    }).collect();
    ceiling_floor_buffer
}
