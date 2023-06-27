use std::{cmp::{min, max}, ptr};
use crate::{TEX_HEIGHT, TEX_WIDTH, SPRITE, NUM_SPRITES};
use packed_simd::u32x8;
use rayon::prelude::*;
pub fn create_final_buffer(
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


pub fn sprite_buffer(
    render_screen_width: usize,
    render_screen_height: usize,
    sprite_order: Vec<usize>,
    pos: (f64, f64, f64),
    plane: (f64, f64),
    dir: (f64, f64),
    screen_pitch: f64,
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
                let sprite_x = SPRITE[sprite_order[index]].x - pos.0;
                let sprite_y = SPRITE[sprite_order[index]].y - pos.1;

                let inv_det = 1.0 / (plane.0 * dir.1 - dir.0 * plane.1);

                let transform_x = inv_det * (dir.1 * sprite_x - dir.0 * sprite_y);
                let transform_y = inv_det * (-plane.1 * sprite_x + plane.0 * sprite_y);

                let sprite_screen_x_ns = ((render_screen_width / 2) as f64 * (1.0 + transform_x / transform_y)) as isize;

                const U_DIV: isize = 1;
                const V_DIV: isize = 1;
                const V_MOVE: f64 = 0.0;
                let v_move_screen = ((V_MOVE / transform_y) + screen_pitch + pos.2 / transform_y) as isize;

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
                            let color = texture[SPRITE[sprite_order[index]].textures][(TEX_WIDTH as isize * tex_y + tex_x) as usize];
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
        transposed_sprite_buffer
    }
}
pub fn wall_buffer(
    render_screen_width: usize,
    render_screen_height: usize,
    plane: (f64, f64),
    dir: (f64, f64),
    pos: (f64, f64, f64),
    map: Vec<Vec<usize>>,
    screen_pitch: f64,
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
            let ray_dir_x: f64 = dir.0 + plane.0 * camera_x;
            let ray_dir_y: f64 = dir.1 + plane.1 * camera_x;

            let (mut map_x, mut map_y) = (pos.0 as isize, pos.1 as isize);

            let (mut side_dist_x, mut side_dist_y): (f64, f64);

            let delta_dist_x = (1.0 / ray_dir_x).abs();
            let delta_dist_y = (1.0 / ray_dir_y).abs();

            let perp_wall_dist: f64;

            let (step_x, step_y): (isize, isize);

            let mut wall_hit: bool = false;

            let mut side_wall: bool = false;

            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (pos.0 - map_x as f64) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f64 + 1.0 - pos.0) * delta_dist_x;
            }
            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (pos.1 - map_y as f64) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f64 + 1.0 - pos.1) * delta_dist_y;
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
                if map[map_x as usize][map_y as usize] > 0 {
                    wall_hit = true;
                }
            }
            if !side_wall {
                perp_wall_dist = (map_x as f64 - pos.0 + (1.0 - step_x as f64) / 2.0) / ray_dir_x;
            } else {
                perp_wall_dist = (map_y as f64 - pos.1 + (1.0 - step_y as f64) / 2.0) / ray_dir_y;
            };
            

            let line_height = (render_screen_height_ss as f64 / perp_wall_dist).abs();


            let mut draw_start = max(0, -line_height as i32 / 2 + (render_screen_height_ss) as i32 / 2 + screen_pitch as i32 + (pos.2.clone() / perp_wall_dist) as i32);
            if draw_start < 0 {
                draw_start = 0;
            };
            let mut draw_end = min(render_screen_height_ss as i32 - 1, line_height as i32 / 2 + (render_screen_height_ss) as i32 / 2 + screen_pitch as i32 + (pos.2.clone() / perp_wall_dist) as i32);
            if draw_end >= (render_screen_height_ss) as i32 {
                draw_end = (render_screen_height_ss) as i32 - 1;
            };
            if draw_end < draw_start {
                draw_end = draw_start;
            }

            let tex_num = map[map_x as usize][map_y as usize] - 1;

            let mut wall_x: f64;
            if side_wall == false {
                wall_x = pos.1 + perp_wall_dist * ray_dir_y;
            } else {
                wall_x = pos.0 + perp_wall_dist * ray_dir_x;
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
            let mut tex_pos = ((draw_start as f64) - screen_pitch - (pos.2.clone() / perp_wall_dist) - (render_screen_height_ss as f64) / 2.0 + line_height / 2.0) * step;


            let mut col_buffer: Vec<u32> = vec![0; render_screen_height_ss];
            let draw_start_u32 = draw_start as u32;
            let draw_end_u32 = draw_end as u32;
            for position_y in draw_start_u32..draw_end_u32 {
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
        (transposed_wall_buffer, z_buffer)
    }
}    
pub fn ceiling_buffer(
    render_screen_width: usize,
    render_screen_height: usize,
    screen_pitch: f64,
    half_screen_height: f64,
    ray_dir_x1: f64,
    ray_dir_x0: f64,
    ray_dir_y1: f64,
    ray_dir_y0: f64,
    pos: (f64, f64, f64),
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
            let cam_z = if is_floor { half_screen_height + pos.2 } else { half_screen_height - pos.2};
            let row_distance = cam_z / (p as f64 / supersample_factor as f64);
            let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / (render_screen_width as f64 * supersample_factor as f64);
            let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / (render_screen_width as f64 * supersample_factor as f64);

            let mut floor_x = pos.0 + row_distance * ray_dir_x0;
            let mut floor_y = pos.1 + row_distance * ray_dir_y0;
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