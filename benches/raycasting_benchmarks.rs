use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::prelude::*;
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
fn ceiling_buffer(screen_pitch: f64, half_screen_height: f64, pos_z: f64, ray_dir_x1: f64, ray_dir_x0: f64, ray_dir_y1: f64, ray_dir_y0: f64, pos_x: f64, pos_y: f64, texture: [[u32; 4096]; 11]) -> Vec<Vec<u32>> {
    let ceiling_floor_buffer: Vec<Vec<u32>> = (0..RENDER_SCREEN_HEIGHT).into_par_iter().map(|y| {
        let is_floor = y as isize > RENDER_SCREEN_HEIGHT as isize / 2 + screen_pitch as isize;
        let p  = if is_floor {
            (y as isize) - (RENDER_SCREEN_HEIGHT as isize) / 2 - screen_pitch as isize
        } else {
            RENDER_SCREEN_HEIGHT as isize / 2 - y as isize + screen_pitch as isize
        };
        let cam_z = if is_floor { half_screen_height + pos_z } else { half_screen_height - pos_z};
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
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Raycasting");
    group.significance_level(0.1).sample_size(500).measurement_time(Duration::from_secs(10));
    use image::io::Reader as ImageReader;
    use image::Pixel;
    const TEX_WIDTH: u32 = 64;
    const TEX_HEIGHT: u32 = 64;
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
    group.bench_function("ceiling buffer", |b| b.iter(|| ceiling_buffer(black_box(0.0), black_box(1080 as f64 / 2.0), black_box(0.0), black_box(-1.0), black_box(-1.0), black_box(0.66), black_box(0.66), black_box(22.0), black_box(11.5), black_box(texture))));
    group.bench_function("wall buffer", |b| b.iter(|| wall_buffer(black_box(-1.0),  black_box(0.0), black_box(0.0), black_box(0.66), black_box(22.0), black_box(11.5), black_box(0.0), black_box(0.0), black_box(texture))));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);