use std::io::{self, Write};
use std::num::IntErrorKind;
use std::thread::available_parallelism;
use fontdue::layout::{Layout, CoordinateSystem, TextStyle};
use rayon::prelude::*;
pub fn render_text_to_buffer(
    r: u32,
    g: u32,
    b: u32,
    text: &str,
    size: f32,
    font: fontdue::Font,
    pos: (f32, f32),
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
    let (mut x, y) = (pos.0, pos.1);
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
pub fn rescale_buffer(
    src_buffer: &Vec<Vec<u32>>,
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize
) -> Vec<Vec<u32>> {
    let scale_x = src_width as f64 / dst_width as f64;
    let scale_y = src_height as f64 / dst_height as f64;
    let dst_buffer: Vec<Vec<u32>> = (0..dst_height).into_par_iter().map(|dst_y| {
        (0..dst_width).map(|dst_x| {
            let src_x = (dst_x as f64 * scale_x).min(src_width as f64 - 1.0);
            let src_y = (dst_y as f64 * scale_y).min(src_height as f64 - 1.0);


    
            let x1 = src_x.floor() as usize;
            let y1 = src_y.floor() as usize;
            let x2 = (x1 + 1).min(src_width - 1);
            let y2 = (y1 + 1).min(src_height - 1);
            let frac_x = src_x - src_x.floor();
            let frac_y = src_y - src_y.floor();


    
            let a = src_buffer[y1][x1];
            let b = src_buffer[y1][x2];
            let c = src_buffer[y2][x1];
            let d = src_buffer[y2][x2];
            
            
    
            let r = ((1.0 - frac_x) * (1.0 - frac_y) * ((a >> 16) & 0xFF) as f64
            + frac_x * (1.0 - frac_y) * ((b >> 16) & 0xFF) as f64
            + (1.0 - frac_x) * frac_y * ((c >> 16) & 0xFF) as f64
            + frac_x * frac_y * ((d >> 16) & 0xFF) as f64) as u32;
        
        let g = ((1.0 - frac_x) * (1.0 - frac_y) * ((a >> 8) & 0xFF) as f64
            + frac_x * (1.0 - frac_y) * ((b >> 8) & 0xFF) as f64
            + (1.0 - frac_x) * frac_y * ((c >> 8) & 0xFF) as f64
            + frac_x * frac_y * ((d >> 8) & 0xFF) as f64) as u32;
        
        let b = ((1.0 - frac_x) * (1.0 - frac_y) * (a & 0xFF) as f64
            + frac_x * (1.0 - frac_y) * (b & 0xFF) as f64
            + (1.0 - frac_x) * frac_y * (c & 0xFF) as f64
            + frac_x * frac_y * (d & 0xFF) as f64) as u32;
            (r << 16) | (g << 8) | b
        }).collect()
    }).collect();


    dst_buffer
}
pub fn game_setup(
    monitor_width: u32,
    monitor_height: u32
) -> (usize, usize, usize, usize, u8, usize, bool) {
    let (mut render_screen_width, mut render_screen_height): (usize, usize) = (1280, 720);
    let mut resolution_input = String::new();
    let mut supersample_factor = 1;
    let (scale_width, scale_height);
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
                            println!("Option Unavailable, using default");
                        }
                    }
                }, Err(error) => {
                    match error {
                        int_error => {
                            match *int_error.kind() {
                                IntErrorKind::Empty => {
                                    println!("No resolution specified, using default");
                                },
                                IntErrorKind::InvalidDigit => {
                                    println!("This is not a number so using default");
                                },
                                IntErrorKind::Zero => {
                                    println!("Zero is invalid so using default");
                                }
                                _ => {
                                    println!("Overflowed int... Good job! Using default");
                                }
                            }
                        }
                    }
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
    let threads_amount: u8;
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
                            threads_amount = 1;
                        },
                        2 => {
                            threads_amount = 2
                        },
                        3 => {
                            threads_amount = 4
                        },
                        4 => {
                            threads_amount = 6
                        },
                        5 => {
                            threads_amount = 8
                        },
                        6 => {
                            threads_amount = 12
                        },
                        7 => {
                            threads_amount = 16
                        },
                        8 => {
                            threads_amount = 24
                        },
                        9 => {
                            threads_amount = 32
                        },
                        10 => {
                            threads_amount = 64
                        },
                        11 => {
                            let thread_amount = available_parallelism().unwrap().get();
                            threads_amount = thread_amount as u8
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
                            threads_amount = thread_amount as u8
                            
                        }
                    }
                }, Err(error) => {
                    match error {
                        int_error => {
                            match *int_error.kind() {
                                IntErrorKind::Empty => {
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
                                    
                                    threads_amount = thread_amount as u8
                                },
                                IntErrorKind::InvalidDigit => {
                                    println!("This is not a number so using default");
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
                                    
                                    threads_amount = thread_amount as u8
                                },
                                IntErrorKind::Zero => {
                                    println!("Zero is invalid so using default");
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
                                    
                                    threads_amount = thread_amount as u8
                                }
                                _ => {
                                    println!("Overflowed int... Good job! Using default");
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
                                    
                                    threads_amount = thread_amount as u8
                                }
                            }
                        }
                    }
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
                            println!("No anti-aliasing technique specified, using default");
                            
                        }
                    }
                }, Err(error) => {
                    match error {
                        int_error => {
                            match *int_error.kind() {
                                IntErrorKind::Empty => {
                                    println!("No anti-aliasing specified, using default");
                                },
                                IntErrorKind::InvalidDigit => {
                                    println!("This is not a number so using default");
                                },
                                IntErrorKind::Zero => {
                                    println!("Zero is invalid so using default");
                                }
                                _ => {
                                    println!("Overflowed int... Good job! Using default");
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(error) => {
            panic!("Failed to read line: {}", error);
        }
    }
    println!("Do you want to scale to another resolution?");
    println!("Select the resolution:");
    println!("1 - 1024x576");
    println!("2 - 1280x720");
    println!("3 - 1366x768");
    println!("4 - 1600x900");
    println!("5 - 1920x1080");
    println!("6 - 2560x1440");
    println!("7 - 3200x1800");
    println!("8 - 3840x2160");
    let mut scale_input = String::new();
    io::stdout().flush().unwrap();
    match io::stdin().read_line(&mut scale_input) {
        Ok(_) => {
            match scale_input.trim().parse::<usize>(){
                Ok(value) => {
                    match value {
                        1 => {
                            scale_width = 1024;
                            scale_height = 576;
                        },
                        2 => {
                            scale_width = 1280;
                            scale_height = 720;
                        },
                        3 => {
                            scale_width = 1366;
                            scale_height = 768;
                        },
                        4 => {
                            scale_width = 1600;
                            scale_height = 900;
                        },
                        5 => {
                            scale_width = 1920;
                            scale_height = 1080;
                        },
                        6 => {
                            scale_width = 2560;
                            scale_height = 1440;
                        },
                        7 => {
                            scale_width = 3200;
                            scale_height = 1800;
                        },
                        8 => {
                            scale_width = 3840;
                            scale_height = 2160;
                        },
                        _ => {
                            println!("Option Unavailable, using default");
                            scale_width = render_screen_width;
                            scale_height = render_screen_height;
                        }
                    }
                }, Err(error) => {
                    match error {
                        int_error => {
                            match *int_error.kind() {
                                IntErrorKind::Empty => {
                                    println!("No resolution specified, using default");
                                    scale_width = render_screen_width;
                                    scale_height = render_screen_height;
                                },
                                IntErrorKind::InvalidDigit => {
                                    println!("This is not a number so using default");
                                    scale_width = render_screen_width;
                                    scale_height = render_screen_height;
                                },
                                IntErrorKind::Zero => {
                                    println!("Zero is invalid so using default");
                                    scale_width = render_screen_width;
                                    scale_height = render_screen_height;
                                }
                                _ => {
                                    println!("Overflowed int... Good job! Using default");
                                    scale_width = render_screen_width;
                                    scale_height = render_screen_height;
                                }
                            }
                        }
                    }
                }
            }

        }
        Err(error) => {
            panic!("Failed to read line: {}", error);
        }
    }
    let mut is_fullscreen = false;
    let mut fscreen_input = String::new();
    println!("Fullscreen?");
    println!("1 - Yes");
    println!("2 - No");
    match io::stdin().read_line(&mut fscreen_input) {
        Ok(_) => {
            match fscreen_input.trim().parse::<usize>(){
                Ok(value) => {
                    match value {
                        1 => {
                            is_fullscreen = true
                        },
                        2 => {
                            is_fullscreen = false
                        },
                        _ => {
                            println!("Option Unavailable, using default");
                        }
                    }
                }, Err(error) => {
                    match error {
                        int_error => {
                            match *int_error.kind() {
                                IntErrorKind::Empty => {
                                    println!("Not specified, using default");
                                },
                                IntErrorKind::InvalidDigit => {
                                    println!("This is not a number so using default");
                                },
                                IntErrorKind::Zero => {
                                    println!("Zero is invalid so using default");
                                }
                                _ => {
                                    println!("Overflowed int... Good job! Using default");
                                }
                            }
                        }
                    }
                }
            }

        }
        Err(error) => {
            panic!("Failed to read line: {}", error);
        }
    }
    (render_screen_width, render_screen_height, scale_width, scale_height, threads_amount, supersample_factor, is_fullscreen)
}