use rayon::prelude::*;
use std::{time::Duration, collections::HashSet};
use softbuffer::GraphicsContext;
use winit::{
    event::{Event, DeviceEvent, WindowEvent, KeyboardInput, VirtualKeyCode /*, ModifiersState*/, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, CursorGrabMode}, dpi::PhysicalSize,
};
use image::io::Reader as ImageReader;
use image::Pixel;
use std::cmp::Ordering;

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

fn sort_sprites(mut order: [usize; NUM_SPRITES], mut dist: [f64; NUM_SPRITES]) {
    let amount = order.len();

    let mut sprites: Vec<(f64, usize)> = Vec::with_capacity(amount);
    for i in 0..amount {
        sprites.push((dist[i], order[i]));
    }

    sprites.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    // restore in reverse order to go from farthest to nearest
    for i in 0..amount {
        let sprite = &sprites[amount - i - 1];
        dist[i] = sprite.0;
        order[i] = sprite.1;
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let display_size = PhysicalSize::new(RENDER_SCREEN_WIDTH as u32, RENDER_SCREEN_HEIGHT as u32);
    let window = WindowBuilder::new().with_inner_size(display_size).with_resizable(true).build(&event_loop).unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();
    let start_time = std::time::Instant::now();
    let mut frames = 0;
    let mut last_fps_print_time = start_time;
    let mut previous_frame_time = std::time::Instant::now();
    
    //  modifiers = ModifiersState::default();
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
    let  pos_z = 0.0;
    let mut sprite_order = [0; NUM_SPRITES];
    let mut sprite_dist = [0.0; NUM_SPRITES];
    
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
                let ceiling_floor_buffer: Vec<Vec<u32>> = (0..RENDER_SCREEN_HEIGHT).into_par_iter().map(|y| {
                    let is_floor = y as isize > RENDER_SCREEN_HEIGHT as isize / 2 + screen_pitch as isize;
                    
                    
                    let ray_dir_x0 = dir_x - plane_x;
                    let ray_dir_y0 = dir_y - plane_y;
                    let ray_dir_x1 = dir_x + plane_x;
                    let ray_dir_y1 = dir_y + plane_y;
                
                    // Current y position compared to the center of the screen (the horizon)
                    let p  = if is_floor {
                        (y as isize) - (RENDER_SCREEN_HEIGHT as isize) / 2 - screen_pitch as isize
                    } else {
                        RENDER_SCREEN_HEIGHT as isize / 2 - y as isize + screen_pitch as isize
                    };
                
                    // Vertical position of the camera.
                    let pos_z = 0.5 * (RENDER_SCREEN_HEIGHT as f64);
                
                    // Horizontal distance from the camera to the floor for the current row.
                    // 0.5 is the z position exactly in the middle between floor and ceiling.
                    let row_distance = pos_z / (p as f64);
                
                    let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / (RENDER_SCREEN_WIDTH as f64);
                    let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / (RENDER_SCREEN_WIDTH as f64);
                
                    let mut floor_x = pos_x + row_distance * ray_dir_x0;
                    let mut floor_y = pos_y + row_distance * ray_dir_y0;
                
                
                    let mut row_buffer: [u32; RENDER_SCREEN_WIDTH as usize] = [0; RENDER_SCREEN_WIDTH as usize];
                
                    for x in 0..RENDER_SCREEN_WIDTH {
                        let tx = ((TEX_WIDTH as f64) * floor_x.fract()) as isize & (TEX_WIDTH - 1) as isize;
                        let ty = ((TEX_HEIGHT as f64) * floor_y.fract()) as isize & (TEX_HEIGHT - 1) as isize;
                
                        floor_x += floor_step_x;
                        floor_y += floor_step_y;
                
                        let floor_texture = 7;
                        let ceiling_texture = 1;
                        if is_floor {
                            // floor
                            let color = texture[floor_texture][(TEX_WIDTH as isize * ty + tx) as usize];
                            row_buffer[x as usize] = color;
                        } else {
                            //ceiling
                            let color = texture[ceiling_texture][(TEX_WIDTH as isize * ty + tx) as usize];
                            row_buffer[x as usize] = color;
                        }
                    }
                    row_buffer.to_vec()
                }).collect();
        
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
                
                    let mut draw_start = -line_height as i32 / 2 + RENDER_SCREEN_HEIGHT as i32 / 2 + screen_pitch as i32;
                    if draw_start < 0 {
                        draw_start = 0;
                    };
                    let mut draw_end = line_height as i32 / 2 + RENDER_SCREEN_HEIGHT as i32 / 2 + screen_pitch as i32;
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
                    let mut tex_pos = ((draw_start as f64) - screen_pitch - (RENDER_SCREEN_HEIGHT as f64) / 2.0 + line_height / 2.0) * step;
                
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
                let transposed_wall_buffer: Vec<Vec<u32>> = (0..RENDER_SCREEN_HEIGHT)
                .into_par_iter()
                .map(|y| {
                    let mut row = Vec::with_capacity(RENDER_SCREEN_WIDTH);
                    for x in 0..RENDER_SCREEN_WIDTH {
                        row.push(wall_buffer[x as usize][y as usize]);
                    }
                    row
                })
                .collect();
                (0..NUM_SPRITES).into_iter().for_each(|  index | {
                    sprite_order[index] = index;
                    sprite_dist[index] = (pos_x - SPRITE[index].x) * (pos_x - SPRITE[index].x) + (pos_y - SPRITE[index].y) * (pos_y - SPRITE[index].y);
                });
                sort_sprites(sprite_order, sprite_dist);

                let mut sprite_buffer: Vec<Vec<u32>> = (0..RENDER_SCREEN_WIDTH)
                    .map(|_| vec![0; RENDER_SCREEN_HEIGHT])
                    .collect();

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

                    for stripe in draw_start_x..draw_end_x {
                        let tex_x = (256 * (stripe - (-sprite_width / 2 + sprite_screen_x)) * TEX_WIDTH as isize / sprite_width) / 256;
                        if transform_y > 0.0 && transform_y < z_buffer[stripe as usize] {
                            for y in draw_start_y..draw_end_y {
                                let d = (y - v_move_screen) * 256 - RENDER_SCREEN_HEIGHT as isize * 128 + sprite_height * 128;
                                let tex_y = ((d * TEX_HEIGHT as isize) / sprite_height) / 256;
                                let color = texture[SPRITE[sprite_order[index]].texture][(TEX_WIDTH as isize * tex_y + tex_x) as usize];
                                if(color & 0x00FFFFFF) != 0 {sprite_buffer[stripe as usize][y as usize] = color};
                            }
                        }
                    }
                }
                let transposed_sprite_buffer: Vec<Vec<u32>> = (0..RENDER_SCREEN_HEIGHT)
                .into_par_iter()
                .map(|y| {
                    let mut row = Vec::with_capacity(RENDER_SCREEN_WIDTH);
                    for x in 0..RENDER_SCREEN_WIDTH {
                        row.push(sprite_buffer[x as usize][y as usize]);
                    }
                    row
                })
                .collect();




                let wall_floor_cel: Vec<Vec<u32>> = transposed_wall_buffer.into_par_iter().zip(ceiling_floor_buffer).map(|(wall_vec, floor_ceiling_vec)| {
                    let draw_buffer: Vec<u32> = wall_vec.into_iter().zip(floor_ceiling_vec.into_iter()).map(|(value_wall, value_floor)| {
                        if value_wall == 0 {
                            value_floor
                        } else {
                            value_wall
                        }
                    }).collect();
                    draw_buffer
                }).collect();
                let final_buffer: Vec<Vec<u32>> = transposed_sprite_buffer.into_par_iter().zip(wall_floor_cel).map(|(sprite_vec, wall_floor_cel_vec)| {
                    let draw_buffer: Vec<u32> = sprite_vec.into_iter().zip(wall_floor_cel_vec.into_iter()).map(|(value_sprite, value_wall_floor)| {
                        if value_sprite == 0 {
                            value_wall_floor
                        } else {
                            value_sprite
                        }
                    }).collect();
                    draw_buffer
                }).collect();
                let buffer: Vec<u32> = final_buffer.iter().flatten().cloned().collect();        
                graphics_context.set_buffer(&buffer, RENDER_SCREEN_WIDTH as u16, RENDER_SCREEN_HEIGHT as u16);
                if init < 3 {
                    init += 1
                } else if init == 3 {
                    window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                    window.set_cursor_visible(false);
                    init = 4;
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
                    
                    _ => continue
                }
            }
    });
}
