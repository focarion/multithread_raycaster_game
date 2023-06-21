use fontdue::layout::{Layout, CoordinateSystem, TextStyle};
#[cfg(feature = "debug")]
use profiling::{tracy_client, scope};

use rayon::prelude::*;
use std::{time::Duration, collections::HashSet, ptr, num::NonZeroU32};
use winit::{
    event::{Event, DeviceEvent, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, CursorGrabMode}, dpi::PhysicalSize,
};
use image::{Pixel, GenericImageView};
use std::io::{self, Write};
use std::thread::available_parallelism;
use packed_simd::{u32x8};
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
const BRICK_WALLS_BYTES: &'static [u8] = include_bytes!("../assets/textures/brick_walls.png");
//const BIG_BRICKS_BYTES: &'static [u8] = include_bytes!("../assets/textures/big_bricks.png");
//const BRICKS_BYTES: &'static [u8] = include_bytes!("../assets/textures/bricks.png");
const RED_BRICKS_BYTES: &'static [u8] = include_bytes!("../assets/textures/red_bricks.png");
const CEILING_BYTES: &'static [u8] = include_bytes!("../assets/textures/ceiling.png");
const RED_FLOORING_BYTES: &'static [u8] = include_bytes!("../assets/textures/red_flooring.png");
const BARREL_BYTES: &'static [u8] = include_bytes!("../assets/textures/barrel.png");
const LIGHT_BYTES: &'static [u8] = include_bytes!("../assets/textures/light.png");
const PILLAR_BYTES: &'static [u8] = include_bytes!("../assets/textures/pillar.png");
fn main() {
    let event_loop = EventLoop::new();
    let monitor_resolution = event_loop.primary_monitor().unwrap().size();
    let (monitor_width, monitor_height) = (monitor_resolution.width, monitor_resolution.height);
    let (mut render_screen_width, mut render_screen_height) = (1280, 720);
    let mut resolution_input = String::new();
    let mut supersample_factor = 1;
        
    println!("{}x{} is your monitor current resolution", monitor_width, monitor_height);
    println!("Select the resolution:");
    println!("1 - 960x540");
    println!("2 - 1024x576");
    println!("3 - 1280x720");
    println!("4 - 1366x768");
    println!("5 - 1600x900");
    println!("6 - 1920x1080");
    println!("7 - 2560x1440");
    println!("8 - 3200x1800");
    println!("9 - 3840x2160");
    println!("Enter for default: 1280x720");

    io::stdout().flush().unwrap();
    match io::stdin().read_line(&mut resolution_input) {
        Ok(_) => {
            match resolution_input.trim().parse::<usize>(){
                Ok(value) => {
                    match value {
                        1 => {
                            render_screen_width = 960;
                            render_screen_height = 540;
                        },
                        2 => {
                            render_screen_width = 1024;
                            render_screen_height = 576;
                        },
                        3 => {
                            render_screen_width = 1280;
                            render_screen_height = 720;
                        },
                        4 => {
                            render_screen_width = 1366;
                            render_screen_height = 768;
                        },
                        5 => {
                            render_screen_width = 1600;
                            render_screen_height = 900;
                        },
                        6 => {
                            render_screen_width = 1920;
                            render_screen_height = 1080;
                        },
                        7 => {
                            render_screen_width = 2560;
                            render_screen_height = 1440;
                        },
                        8 => {
                            render_screen_width = 3200;
                            render_screen_height = 1800;
                        },
                        9 => {
                            render_screen_width = 3840;
                            render_screen_height = 2160;
                        },
                        _ => {
                            println!("No resolution specified, using default");
                            return
                        }
                    }
                }, Err(_) => {
                    println!("Your input was invalid");
                }
            }

        }
        Err(error) => {
            panic!("Failed to read line: {}", error);
        }
    }
    {
        let thread_amount = available_parallelism().unwrap().get();
        println!("You have {} Threads Available", thread_amount);
    }
    let mut thread_amount = String::new();
    println!("Select the amount of Threads:");
    println!("I personally recommend 80% of the cpu threads up to 12 (Sometimes more is better but the overhead makes it slower)");
    println!("1 - 1 Thread");
    println!("2 - 2 Threads");
    println!("3 - 4 Threads");
    println!("4 - 6 Threads");
    println!("5 - 8 Threads");
    println!("6 - 12 Threads");
    println!("7 - 16 Threads");
    println!("8 - 24 Threads");
    println!("9 - 32 Threads");
    println!("10 - 64 Threads");
    println!("11 - All Threads");
    println!("Enter For default: 80% or 4 if lower than 5");
    io::stdout().flush().unwrap();
    match io::stdin().read_line(&mut thread_amount) {
        Ok(_) => {
            match thread_amount.trim().parse::<usize>(){
                Ok(value) => {
                    match value {
                        1 => {
                            rayon::ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();
                        },
                        2 => {
                            rayon::ThreadPoolBuilder::new().num_threads(2).build_global().unwrap();
                        },
                        3 => {
                            rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();
                        },
                        4 => {
                            rayon::ThreadPoolBuilder::new().num_threads(6).build_global().unwrap();
                        },
                        5 => {
                            rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();
                        },
                        6 => {
                            rayon::ThreadPoolBuilder::new().num_threads(12).build_global().unwrap();
                        },
                        7 => {
                            rayon::ThreadPoolBuilder::new().num_threads(16).build_global().unwrap();
                        },
                        8 => {
                            rayon::ThreadPoolBuilder::new().num_threads(24).build_global().unwrap();
                        },
                        9 => {
                            rayon::ThreadPoolBuilder::new().num_threads(32).build_global().unwrap();
                        },
                        10 => {
                            rayon::ThreadPoolBuilder::new().num_threads(64).build_global().unwrap();
                        },
                        11 => {
                            let thread_amount = available_parallelism().unwrap().get();
                            rayon::ThreadPoolBuilder::new().num_threads(thread_amount).build_global().unwrap();
                        },
                        _ => {
                            println!("No thread amount specified, using default");
                            let thread_amount = {
                                let threads = available_parallelism().unwrap().get();
                                if threads < 5 {
                                    4
                                } else if threads > 12 {
                                    12
                                } else {
                                (threads as f64 * 0.8) as usize
                                }
                            };
                            
                            rayon::ThreadPoolBuilder::new().num_threads(thread_amount).build_global().unwrap();
                        }
                    }
                }, Err(_) => {
                    println!("Your input was invalid");
                }
            }
        }
        Err(error) => {
            panic!("Failed to read line: {}", error);
        }
    }
    println!("Anti-Aliasing settings:");
    println!("1 - No Anti-Aliasing");
    println!("2 - 2x SSAA");
    println!("3 - 4x SSAA");
    println!("4 - 8x SSAA");
    println!("5 - 16x SSAA (Masochist mode)");
    println!("Enter For default: No Anti-Aliasing");
    let mut aa_input = String::new();
    match io::stdin().read_line(&mut aa_input) {
        Ok(_) => {
            match aa_input.trim().parse::<usize>(){
                Ok(value) => {
                    match value {
                        1 => {
                            supersample_factor = 1
                        },
                        2 => {
                            supersample_factor = 2
                        },
                        3 => {
                            supersample_factor = 4
                        },
                        4 => {
                            supersample_factor = 8
                        },
                        5 => {
                            supersample_factor = 16
                        },
                        _ => {
                            supersample_factor = 1
                        }
                    }
                }, Err(_) => {
                    println!("Your input was invalid");
                }
            }
        }
        Err(error) => {
            panic!("Failed to read line: {}", error);
        }
    }
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
    let font = include_bytes!("../assets/fonts/hud.otf") as &[u8];
    let game_text_font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    let display_size = PhysicalSize::new(render_screen_width as u32, render_screen_height as u32);
    let window = WindowBuilder::new().with_title(format!("Multithread Raycaster Game Version: {}", clap::crate_version!())).with_inner_size(display_size).with_resizable(false).build(&event_loop).unwrap();
    let graphics_context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&graphics_context, &window) }.unwrap();
    surface
        .resize(
            NonZeroU32::new(render_screen_width as u32).unwrap(),
            NonZeroU32::new(render_screen_height as u32).unwrap(),
        )
        .unwrap();
    let start_time = std::time::Instant::now();
    let mut frames = 0;
    let mut last_fps_print_time = start_time;
    let mut previous_frame_time = std::time::Instant::now();
    
    let mut mouse_lock = false;
    let mut init = 0;
    let mut texture: Vec<Vec<u32>> = vec![vec![0; (TEX_WIDTH * TEX_HEIGHT) as usize]; 20];
    {

        let bricks_texture = image::load_from_memory(BRICK_WALLS_BYTES).unwrap();
        let  buffer: Vec<u32> = bricks_texture.pixels()
        .map(|(_, _, pixel)| {
            let coverted_rgb = pixel.to_rgb();
            let rgb = coverted_rgb.channels();
            let color: u32;
            color= ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            color
        })
        .collect();
        texture[0] = buffer.clone();
        texture[2] = buffer.clone();
        texture[3] = buffer;
    }
    {
        let red_bricks_texture = image::load_from_memory(RED_BRICKS_BYTES).unwrap();
        let  buffer: Vec<u32> = red_bricks_texture.pixels()
        .map(|(_, _, pixel)| {
            let coverted_rgb = pixel.to_rgb();
            let rgb = coverted_rgb.channels();
            let color: u32;
            color= ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            color
        })
        .collect();
        texture[4] = buffer.clone();
        texture[5] = buffer.clone();
        texture[6] = buffer;
    }
    {
        let red_flooring_texture = image::load_from_memory(RED_FLOORING_BYTES).unwrap();
        let  buffer: Vec<u32> = red_flooring_texture.pixels()
        .map(|(_, _, pixel)| {
            let coverted_rgb = pixel.to_rgb();
            let rgb = coverted_rgb.channels();
            let color: u32;
            color= ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            color
        })
        .collect();
        texture[7] = buffer;
    }
    {
        let ceiling_texture = image::load_from_memory(CEILING_BYTES).unwrap();
        let  buffer: Vec<u32> = ceiling_texture.pixels()
        .map(|(_, _, pixel)| {
            let coverted_rgb = pixel.to_rgb();
            let rgb = coverted_rgb.channels();
            let color: u32;
            color= ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            color
        })
        .collect();
        texture[1] = buffer;
    }
    {
        let barrel_texture = image::load_from_memory(BARREL_BYTES).unwrap();
        let  buffer: Vec<u32> = barrel_texture.pixels()
        .map(|(_, _, pixel)| {
            let coverted_rgb = pixel.to_rgb();
            let rgb = coverted_rgb.channels();
            let color: u32;
            color= ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            color
        })
        .collect();
        texture[8] = buffer;
    }
    {
        let pillar_texture = image::load_from_memory(PILLAR_BYTES).unwrap();
        let  buffer: Vec<u32> = pillar_texture.pixels()
        .map(|(_, _, pixel)| {
            let coverted_rgb = pixel.to_rgb();
            let rgb = coverted_rgb.channels();
            let color: u32;
            color= ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            color
        })
        .collect();
        texture[9] = buffer;
    }
    {
        let light_texture = image::load_from_memory(LIGHT_BYTES).unwrap();
        let  buffer: Vec<u32> = light_texture.pixels()
        .map(|(_, _, pixel)| {
            let coverted_rgb = pixel.to_rgb();
            let rgb = coverted_rgb.channels();
            let color: u32;
            color= ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            color
        })
        .collect();
        texture[10] = buffer.clone();
    }
    let (mut pos_x, mut pos_y) = (22.0, 11.5); 
    let (mut dir_x, mut dir_y) = (-1.0, 0.0);
    let (mut plane_x, mut plane_y) = (0.0, 0.66);
    #[allow(unused_assignments)]
    let mut movespeed = 0.05;
    let mut screen_pitch = 0.0;
    let mut pos_z = 0.0;
    let mut is_jumping = false;
    let mut is_crouching = false;
    let mut is_crouched = false;
    let mut vertical_velocity = 0.0;
    let mut last_update = std::time::Instant::now();
    let mut movement_cooldown = std::time::Instant::now();
    
    // rayon::ThreadPoolBuilder::new().num_threads(12).build_global().unwrap();
    
    let mut pressed_keys = HashSet::new();
    #[cfg(feature = "debug")]
    scope!("Main Loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let current_frame_time = std::time::Instant::now();
        let frame_duration = current_frame_time.duration_since(previous_frame_time);
        previous_frame_time = current_frame_time;
        let frame_time_ms = frame_duration.as_millis();
        
        movespeed = (frame_time_ms as f64 / 1000.0) * 3.5;   
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                #[cfg(feature = "debug")]
                profiling::scope!("Redraw");
                {
                    frames += 1;
                    if last_fps_print_time.elapsed() > Duration::from_secs(1) {
                        println!("FPS: {:.1}", frames as f64 / last_fps_print_time.elapsed().as_secs_f64());
                        last_fps_print_time = std::time::Instant::now();
                        frames = 0;
                    }
                    let half_screen_height = render_screen_height as f64 / 2.0;
                    let ray_dir_x0 = dir_x - plane_x;
                    let ray_dir_y0 = dir_y - plane_y;
                    let ray_dir_x1 = dir_x + plane_x;
                    let ray_dir_y1 = dir_y + plane_y;

                    let ceiling_floor_buffer = ceiling_buffer(render_screen_width, render_screen_height, screen_pitch, half_screen_height, pos_z, ray_dir_x1, ray_dir_x0, ray_dir_y1, ray_dir_y0, pos_x, pos_y, texture.clone(), supersample_factor);

                    let (wall_buffer, z_buffer) = wall_buffer(render_screen_width, render_screen_height, dir_x, plane_x, dir_y, plane_y, pos_x, pos_y, screen_pitch, pos_z, texture.clone(), supersample_factor);
                    
                    let mut transposed_wall_buffer: Vec<Vec<u32>> = vec![vec![0; render_screen_width]; render_screen_height];
                    #[cfg(feature = "debug")]
                    profiling::scope!("Transpose Wall");
                    {
                        transposed_wall_buffer
                        .par_iter_mut()
                        .enumerate()
                        .for_each(|(y, row)| {
                            let row_ptr = row.as_mut_ptr() as *mut u32x8;
                            let mut x = 0;
                            while x < render_screen_width {
                                unsafe {
                                    let data = u32x8::new(
                                        *wall_buffer[x as usize].get_unchecked(y),
                                        *wall_buffer[(x + 1) as usize].get_unchecked(y),
                                        *wall_buffer[(x + 2) as usize].get_unchecked(y),
                                        *wall_buffer[(x + 3) as usize].get_unchecked(y),
                                        *wall_buffer[(x + 4) as usize].get_unchecked(y),
                                        *wall_buffer[(x + 5) as usize].get_unchecked(y),
                                        *wall_buffer[(x + 6) as usize].get_unchecked(y),
                                        *wall_buffer[(x + 7) as usize].get_unchecked(y),
                                    );
                                    ptr::write(row_ptr.add(x / 8), data);
                                }
                                x += 8;
                            }
                        });
                    }

                    let sprite_data: Vec<(usize, f64)> = (0..NUM_SPRITES).into_par_iter().map(|index| {
                        let order = index;
                        let dist = (pos_x - SPRITE[index].x) * (pos_x - SPRITE[index].x) + (pos_y - SPRITE[index].y) * (pos_y - SPRITE[index].y);
                        (order, dist)
                    }).collect();

                    let mut sprite_data = sprite_data;
                    sprite_data.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                    let sprite_order: Vec<usize> = sprite_data.into_iter().map(|(order, _)| order).collect();

                    let sprite_buffer = sprite_buffer(render_screen_width, render_screen_height, sprite_order, pos_x, pos_y, plane_x, dir_y, dir_x, plane_y, screen_pitch, pos_z, z_buffer, texture.clone(), supersample_factor);

                    let mut transposed_sprite_buffer: Vec<Vec<u32>> = vec![vec![0; render_screen_width]; render_screen_height];
                    #[cfg(feature = "debug")]
                    profiling::scope!("Transpose Sprite");
                    {
                        transposed_sprite_buffer
                        .par_iter_mut()
                        .enumerate()
                        .for_each(|(y, row)| {
                            let row_ptr = row.as_mut_ptr() as *mut u32x8;
                            let mut x = 0;
                            while x < render_screen_width {
                                unsafe {
                                    let data = u32x8::new(
                                        *sprite_buffer[x as usize].get_unchecked(y),
                                        *sprite_buffer[(x + 1) as usize].get_unchecked(y),
                                        *sprite_buffer[(x + 2) as usize].get_unchecked(y),
                                        *sprite_buffer[(x + 3) as usize].get_unchecked(y),
                                        *sprite_buffer[(x + 4) as usize].get_unchecked(y),
                                        *sprite_buffer[(x + 5) as usize].get_unchecked(y),
                                        *sprite_buffer[(x + 6) as usize].get_unchecked(y),
                                        *sprite_buffer[(x + 7) as usize].get_unchecked(y),
                                    );
                                    ptr::write(row_ptr.add(x / 8), data);
                                }
                                x += 8;
                            }
                        });
                        //info!("Finish Transpose Sprite Buffer");
                    }



                    let final_buffer = create_final_buffer(transposed_wall_buffer, ceiling_floor_buffer, transposed_sprite_buffer);
                    #[cfg(feature = "debug")]
                    profiling::scope!("Draw Frame");
                    {
                        #[cfg(feature = "debug")]
                        profiling::scope!("Draw to Winit Frame");
                        {
                            
                            let mut buffer: Vec<u32> = final_buffer.into_par_iter().flatten_iter().collect();
                            
                            //render_text_to_buffer(255, 255, 255, "Text",10.0, game_text_font.clone(), 0.0, render_screen_height as f32 / 1.1,&mut buffer,  render_screen_width, render_screen_height, );
                            //render_text_to_buffer(255, 0, 0, "Another Text",12.0, game_text_font.clone(), 0.0, render_screen_height as f32 / 1.8,&mut buffer,  render_screen_width, render_screen_height, );
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
                        if init < 3 {
                            init += 1
                        } else if init == 3 {
                            window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                            window.set_cursor_visible(false);
                            mouse_lock = true;
                            init = 4;
                            // *control_flow = ControlFlow::Exit
                        }
                        if is_jumping && !is_crouching && !is_crouched {
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
                        } else if !is_jumping && is_crouching {
                            let now = std::time::Instant::now();
                            let delta_time = (now - last_update).as_secs_f64() * 3.0;
                            last_update = now;
                            if !is_crouched {
                                pos_z -= vertical_velocity * delta_time;
                                if pos_z <= -200.0 {
                                    pos_z = -200.0;
                                    vertical_velocity = 0.0;
                                    is_crouching = false;
                                    is_crouched = true
                                }
                                
                            } else {
                                pos_z += vertical_velocity * delta_time;
                                if pos_z >= 0.0 {
                                    pos_z = 0.0;
                                    vertical_velocity = 0.0;
                                    is_crouching = false;
                                    is_crouched = false
                                }
                                
                            }
                            


                        }
                        #[cfg(feature = "debug")]
                        profiling::finish_frame!();
                    }
                }
            }
            Event::MainEventsCleared {} =>{
                window.request_redraw();
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
                        let x = delta.0;
                        let sensitivity = render_screen_width as f64 / 5.0;
                        let old_dir_x = dir_x;
                        dir_x = dir_x * (-x / sensitivity).cos() - dir_y * (-x / sensitivity).sin();
                        dir_y = old_dir_x * (-x / sensitivity).sin() + dir_y * (-x / sensitivity).cos();
                        let old_plane_x = plane_x;
                        plane_x = plane_x * (-x / sensitivity).cos() - plane_y * (-x / sensitivity).sin();
                        plane_y = old_plane_x * (-x / sensitivity).sin() + plane_y * (-x / sensitivity).cos();
                        let y = delta.1;
                        screen_pitch -= y;
                        screen_pitch = f64::clamp(screen_pitch, -((render_screen_width as f64)*0.28), (render_screen_width as f64)*0.28);
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
                        if !is_jumping {
                            if movement_cooldown.elapsed() > Duration::from_millis(250) {
                                if !is_crouching && !is_crouched {
                                    is_crouching = true;
                                    vertical_velocity = 250.0;
                                } else if !is_crouching && is_crouched {
                                    is_crouching = true;
                                    vertical_velocity = 250.0
                                }
                                movement_cooldown = std::time::Instant::now();
                            }
                        }
                    },
                    VirtualKeyCode::Space => {
                        if !is_crouched {
                            if movement_cooldown.elapsed() > Duration::from_millis(250) {
                                if !is_jumping && !is_crouching {
                                    is_jumping = true;
                                    vertical_velocity = 300.0;
                                }
                                movement_cooldown = std::time::Instant::now();
                            } 
                        }
                    },
                    
                    
                    _ => continue
                }
            }
    });
}



fn create_final_buffer(
    mut transposed_wall_buffer: Vec<Vec<u32>>,
    ceiling_floor_buffer: Vec<Vec<u32>>,
    mut transposed_sprite_buffer: Vec<Vec<u32>>
) -> Vec<Vec<u32>> {
    #[cfg(feature = "debug")]
    profiling::scope!("Final Buffer");
    {
        transposed_wall_buffer.par_iter_mut().zip(ceiling_floor_buffer).for_each(|(wall_vec, floor_ceiling_vec)| {
            for (value_wall, value_floor) in wall_vec.iter_mut().zip(floor_ceiling_vec) {
                if *value_wall == 0 {
                    *value_wall = value_floor;
                }
            }
        });
        transposed_sprite_buffer.par_iter_mut().zip(transposed_wall_buffer).for_each(|(sprite_vec, wall_floor_vec)| {
            for (value_sprite, value_wall) in sprite_vec.iter_mut().zip(wall_floor_vec) {
                if *value_sprite == 0 {
                    *value_sprite = value_wall;
                }
            }
        });
        transposed_sprite_buffer
    }
}


fn sprite_buffer(
    render_screen_width: usize,
    render_screen_height: usize,
    sprite_order: Vec<usize>,
    pos_x: f64,
    pos_y: f64,
    plane_x: f64,
    dir_y: f64,
    dir_x: f64,
    plane_y: f64,
    screen_pitch: f64,
    pos_z: f64,
    z_buffer: Vec<f64>,
    texture: Vec<Vec<u32>>,
    supersample_factor: usize
) -> Vec<Vec<u32>> {
    #[cfg(feature = "debug")]
    profiling::scope!("Sprite Buffer");
    {
        let render_screen_width_ss = render_screen_width * supersample_factor;
        let render_screen_height_ss = render_screen_height * supersample_factor;
        let sprite_buffer: Vec<Vec<u32>> = (0..render_screen_width_ss).into_par_iter().map(|x| {
            let mut column_buffer = vec![0; render_screen_height_ss];

            for index in 0..NUM_SPRITES {
                let sprite_x = SPRITE[sprite_order[index]].x - pos_x;
                let sprite_y = SPRITE[sprite_order[index]].y - pos_y;

                let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);

                let transform_x = inv_det * (dir_y * sprite_x - dir_x * sprite_y);
                let transform_y = inv_det * (-plane_y * sprite_x + plane_x * sprite_y);

                let sprite_screen_x_ns = ((render_screen_width / 2) as f64 * (1.0 + transform_x / transform_y)) as isize;

                const U_DIV: isize = 1;
                const V_DIV: isize = 1;
                const V_MOVE: f64 = 0.0;
                let v_move_screen = ((V_MOVE / transform_y) + screen_pitch + pos_z / transform_y) as isize;

                let sprite_height_ns = ((render_screen_height as f64 / (transform_y)).abs()) as isize / V_DIV;
                let mut draw_start_y = -sprite_height_ns / 2 + render_screen_height as isize / 2 + v_move_screen;
                if draw_start_y < 0 {
                    draw_start_y = 0
                };
                draw_start_y *= supersample_factor as isize;
                let mut draw_end_y = sprite_height_ns / 2 + render_screen_height as isize / 2 + v_move_screen;
                if draw_end_y >= render_screen_height as isize {
                    draw_end_y = render_screen_height as isize - 1
                };
                draw_end_y *= supersample_factor as isize;

                let sprite_width_ns = ((render_screen_height as f64 / (transform_y)).abs()) as isize / U_DIV;
                let mut draw_start_x = -sprite_width_ns / 2 + sprite_screen_x_ns;
                if draw_start_x < 0 {
                    draw_start_x = 0
                };
                draw_start_x *= supersample_factor as isize;
                let mut draw_end_x = sprite_width_ns / 2 + sprite_screen_x_ns;
                if draw_end_x > render_screen_width as isize {
                    draw_end_x = render_screen_width as isize
                };
                draw_end_x *= supersample_factor as isize;

                if x as isize >= draw_start_x && (x as isize) < draw_end_x {
                    let stripe_ns = x as isize / supersample_factor as isize;
                    let tex_x = (256 * (stripe_ns - (-sprite_width_ns / 2 + sprite_screen_x_ns)) * TEX_WIDTH as isize / sprite_width_ns) / 256;
                    if transform_y > 0.0 && transform_y < z_buffer[stripe_ns as usize] {
                        for y in draw_start_y..draw_end_y {
                            let d = ((y / supersample_factor as isize) - v_move_screen) * 256 - render_screen_height as isize * 128 + sprite_height_ns * 128;
                            let tex_y = ((d * TEX_HEIGHT as isize) / sprite_height_ns) / 256;
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
        if supersample_factor > 1 {
            let final_buffer: Vec<Vec<u32>> = (0..render_screen_width).into_par_iter().map(|x| {
                (0..render_screen_height).map(|y| {
                    let mut r_sum = 0;
                    let mut g_sum = 0;
                    let mut b_sum = 0;
                    for sy in 0..supersample_factor {
                        for sx in 0..supersample_factor {
                            let color = sprite_buffer[x * supersample_factor + sx][y * supersample_factor + sy];
                            let r = (color >> 16) & 0xFF;
                            let g = (color >> 8) & 0xFF;
                            let b = color & 0xFF;

                            r_sum += r;
                            g_sum += g;
                            b_sum += b;
                        }
                    }

                    let count = (supersample_factor * supersample_factor) as u32;
                    let r_avg = (r_sum / count) as u32;
                    let g_avg = (g_sum / count) as u32;
                    let b_avg = (b_sum / count) as u32;

                    (r_avg << 16) | (g_avg << 8) | b_avg
                }).collect::<Vec<u32>>()
            }).collect::<Vec<Vec<u32>>>();
            
            
            return final_buffer
        }
        sprite_buffer
    }
}
fn wall_buffer(
    render_screen_width: usize,
    render_screen_height: usize,
    dir_x: f64,
    plane_x: f64,
    dir_y: f64,
    plane_y: f64,
    pos_x: f64,
    pos_y: f64,
    screen_pitch: f64,
    pos_z: f64,
    texture: Vec<Vec<u32>>,
    supersample_factor: usize
) -> (Vec<Vec<u32>>, Vec<f64>) {
    #[cfg(feature = "debug")]
    profiling::scope!("Wall Buffer");
    {
        let screen_pitch = screen_pitch * supersample_factor as f64;
        let render_screen_width_ss = render_screen_width * supersample_factor;
        let render_screen_height_ss = render_screen_height * supersample_factor;
        let (wall_buffer, z_buffer): (Vec<Vec<u32>>, Vec<f64>) = (0..render_screen_width_ss).into_par_iter().map(|x| {
            let camera_x: f64 = 2.0 * x as f64 / (render_screen_width_ss as f64) - 1.0;
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
                perp_wall_dist = (map_x as f64 - pos_x + (1.0 - step_x as f64) / 2.0) / ray_dir_x;
            } else {
                perp_wall_dist = (map_y as f64 - pos_y + (1.0 - step_y as f64) / 2.0) / ray_dir_y;
            };
            

            let line_height = (render_screen_height_ss as f64 / perp_wall_dist).abs();


            let mut draw_start = -line_height as i32 / 2 + (render_screen_height_ss) as i32 / 2 + screen_pitch as i32 + (pos_z.clone() / perp_wall_dist) as i32;
            if draw_start < 0 {
                draw_start = 0;
            };
            let mut draw_end = line_height as i32 / 2 + (render_screen_height_ss) as i32 / 2 + screen_pitch as i32 + (pos_z.clone() / perp_wall_dist) as i32;
            if draw_end >= (render_screen_height_ss) as i32 {
                draw_end = (render_screen_height_ss) as i32 - 1;
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
            let mut tex_pos = ((draw_start as f64) - screen_pitch - (pos_z.clone() / perp_wall_dist) - (render_screen_height_ss as f64) / 2.0 + line_height / 2.0) * step;


            let mut col_buffer: Vec<u32> = vec![0; render_screen_height_ss];
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
        if supersample_factor > 1 {
            let final_buffer: Vec<Vec<u32>> = (0..render_screen_width).into_par_iter().map(|x| {
                (0..render_screen_height).map(|y| {
                    let mut r_sum = 0;
                    let mut g_sum = 0;
                    let mut b_sum = 0;
                    for sy in 0..supersample_factor {
                        for sx in 0..supersample_factor {
                            let color = wall_buffer[x * supersample_factor + sx][y * supersample_factor + sy];
                            let r = (color >> 16) & 0xFF;
                            let g = (color >> 8) & 0xFF;
                            let b = color & 0xFF;

                            r_sum += r;
                            g_sum += g;
                            b_sum += b;
                        }
                    }

                    let count = (supersample_factor * supersample_factor) as u32;
                    let r_avg = (r_sum / count) as u32;
                    let g_avg = (g_sum / count) as u32;
                    let b_avg = (b_sum / count) as u32;

                    (r_avg << 16) | (g_avg << 8) | b_avg
                }).collect::<Vec<u32>>()
            }).collect::<Vec<Vec<u32>>>();
            let final_zbuffer: Vec<f64> = (0..render_screen_width).into_par_iter().map(|x| {
                let mut min_z = f64::MAX;
                for sx in 0..supersample_factor {
                    let z = z_buffer[x * supersample_factor + sx];
                    if z < min_z {
                        min_z = z;
                    }
                }
                min_z
            }).collect();
            
            
            return (final_buffer, final_zbuffer)
        }

        (wall_buffer, z_buffer)
    }
}    
fn ceiling_buffer(
    render_screen_width: usize,
    render_screen_height: usize,
    screen_pitch: f64,
    half_screen_height: f64,
    pos_z: f64,
    ray_dir_x1: f64,
    ray_dir_x0: f64,
    ray_dir_y1: f64,
    ray_dir_y0: f64,
    pos_x: f64,
    pos_y: f64,
    texture: Vec<Vec<u32>>,
    supersample_factor: usize
) -> Vec<Vec<u32>> {
    #[cfg(feature = "debug")]
    profiling::scope!("Ceiling Buffer");
    {
        let ceiling_floor_buffer: Vec<Vec<u32>> = (0..render_screen_height*supersample_factor).into_par_iter().map(|y| {
            let is_floor = y as isize > (render_screen_height * supersample_factor) as isize / 2 + (screen_pitch * supersample_factor as f64) as isize;
            let p  = if is_floor {
                (y as isize) - (render_screen_height * supersample_factor) as isize / 2 - (screen_pitch * supersample_factor as f64) as isize
            } else {
                (render_screen_height * supersample_factor) as isize / 2 - y as isize + (screen_pitch * supersample_factor as f64) as isize
            };
            let cam_z = if is_floor { half_screen_height + pos_z } else { half_screen_height - pos_z};
            let row_distance = cam_z / (p as f64 / supersample_factor as f64);
            let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / (render_screen_width as f64 * supersample_factor as f64);
            let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / (render_screen_width as f64 * supersample_factor as f64);

            let mut floor_x = pos_x + row_distance * ray_dir_x0;
            let mut floor_y = pos_y + row_distance * ray_dir_y0;
            let mut x_buffer: Vec<u32> = Vec::with_capacity(render_screen_width);
            (0..render_screen_width*supersample_factor).for_each(|_| {
                let tx = ((TEX_WIDTH as f64) * floor_x.fract()) as isize & (TEX_WIDTH - 1) as isize;
                let ty = ((TEX_HEIGHT as f64) * floor_y.fract()) as isize & (TEX_HEIGHT - 1) as isize;
                floor_x += floor_step_x;
                floor_y += floor_step_y;
                if is_floor {
                    // floor
                    x_buffer.push(texture[7][(TEX_WIDTH as isize * ty + tx) as usize]);
                } else {
                    //ceiling
                    x_buffer.push(texture[1][(TEX_WIDTH as isize * ty + tx) as usize]);
                }
            });
            x_buffer
        }).collect();
        if supersample_factor > 1 {
                // Averaging pixels to reduce to original size.
            let final_buffer: Vec<Vec<u32>> = (0..render_screen_height).into_par_iter().map(|y| {
                (0..render_screen_width).map(|x| {
                    let mut r_sum = 0;
                    let mut g_sum = 0;
                    let mut b_sum = 0;
                    for sy in 0..supersample_factor {
                        for sx in 0..supersample_factor {
                            let color = ceiling_floor_buffer[y * supersample_factor + sy][x * supersample_factor + sx];
                            let r = (color >> 16) & 0xFF;
                            let g = (color >> 8) & 0xFF;
                            let b = color & 0xFF;

                            r_sum += r;
                            g_sum += g;
                            b_sum += b;
                        }
                    }

                    let count = (supersample_factor * supersample_factor) as u32;
                    let r_avg = (r_sum / count) as u32;
                    let g_avg = (g_sum / count) as u32;
                    let b_avg = (b_sum / count) as u32;

                    (r_avg << 16) | (g_avg << 8) | b_avg
                }).collect::<Vec<u32>>()
            }).collect::<Vec<Vec<u32>>>();

            return final_buffer

        }
        ceiling_floor_buffer
    }
}
fn render_text_to_buffer(
    r: u32,
    g: u32,
    b: u32,
    text: &str,
    size: f32,
    font: fontdue::Font,
    pos_x: f32,
    pos_y: f32,
    buffer: &mut [u32],
    width: usize,
    height: usize
) {
    let fonts = &[font.clone()];
    let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
    let font_scale = 0.75 * size;
    let scale = (height as f32 / font_scale).round();  // Scale the font to the height as per original function
    layout.append(fonts, &TextStyle::new(text, scale, 0));
    let glyph_vec = layout.glyphs();
    let (mut x, y) = (pos_x, pos_y);
    for glyph in glyph_vec {
        let (metrics, bitmap) = font.rasterize(glyph.parent, scale);
        for (i, pixel) in bitmap.iter().enumerate() {
            let bitmap_x = i % metrics.width;
            let bitmap_y = i / metrics.width;

            let buffer_x = x as usize + bitmap_x;
            let buffer_y = if y as usize > metrics.height as usize { 
                (y as usize - metrics.height as usize) + bitmap_y 
            } else { 
                bitmap_y 
            };

            if buffer_x < width && buffer_y < height {
                let buffer_index = buffer_y * width + buffer_x;

                // RGBA conversion with provided colors.
                let alpha = *pixel as u32;
                let red = r * alpha / 0xFF;
                let green = g * alpha / 0xFF;
                let blue = b * alpha / 0xFF;
                let rgba = (red << 16) | (green << 8) | blue;
                if rgba != 0 {
                buffer[buffer_index] = rgba;
                }
            }
        }
        x += metrics.advance_width;
    }
}