pub const TEX_WIDTH: u32 = 64;
pub const TEX_HEIGHT: u32 = 64;
const BRICK_WALLS_BYTES: &'static [u8] = include_bytes!("../assets/textures/brick_walls.png");
//const BIG_BRICKS_BYTES: &'static [u8] = include_bytes!("../assets/textures/big_bricks.png");
//const BRICKS_BYTES: &'static [u8] = include_bytes!("../assets/textures/bricks.png");
const RED_BRICKS_BYTES: &'static [u8] = include_bytes!("../assets/textures/red_bricks.png");
const CEILING_BYTES: &'static [u8] = include_bytes!("../assets/textures/ceiling.png");
const RED_FLOORING_BYTES: &'static [u8] = include_bytes!("../assets/textures/red_flooring.png");
const BARREL_BYTES: &'static [u8] = include_bytes!("../assets/textures/barrel.png");
const LIGHT_BYTES: &'static [u8] = include_bytes!("../assets/textures/light.png");
const PILLAR_BYTES: &'static [u8] = include_bytes!("../assets/textures/pillar.png");

// const PILLAR_BYTES: &'static [u8] = include_bytes!("../assets/textures/pillar.png");
pub const GAME_MAIN_FONT: &[u8] = include_bytes!("../assets/fonts/hud.otf") as &[u8];
use fontdue::Font;
use image::{Pixel, GenericImageView};
pub struct Assets {
    pub textures: Vec<Vec<u32>>,
    pub two_dim_textures: Vec<Vec<Vec<u32>>>,
    pub fonts: Vec<Font>
}
impl Assets {
    pub fn new() -> Self {
        let mut textures: Vec<Vec<u32>> = vec![vec![0; (TEX_WIDTH * TEX_HEIGHT) as usize]; 20];
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
        textures[0] = buffer.clone();
        textures[2] = buffer.clone();
        textures[3] = buffer;
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
        textures[4] = buffer.clone();
        textures[5] = buffer.clone();
        textures[6] = buffer;
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
        textures[7] = buffer;
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
        textures[1] = buffer;
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
        textures[8] = buffer;
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
        textures[9] = buffer;
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
        textures[10] = buffer.clone();
    }
    // Two Dimensional Textures:
    let mut two_dim_textures: Vec<Vec<Vec<u32>>>= vec![vec![vec![0; TEX_WIDTH as usize]; TEX_HEIGHT as usize]; 20];
    {

    }
    let game_text_font = fontdue::Font::from_bytes(GAME_MAIN_FONT, fontdue::FontSettings::default()).unwrap();
    let fonts = vec![game_text_font];
    Self {
        textures,
        two_dim_textures,
        fonts
    }
    }
}
pub struct Sprite {
    pub x: f64,
    pub y: f64,
    pub textures: usize
}
pub const NUM_SPRITES: usize = 19;
pub const SPRITE: [Sprite; NUM_SPRITES] = [
    Sprite {x: 20.5, y: 11.5, textures: 10},
    Sprite {x: 18.5, y: 4.5, textures: 10},
    Sprite {x: 10.0, y: 4.5, textures: 10},
    Sprite {x: 10.0, y: 12.5, textures: 10},
    Sprite {x: 3.5, y: 6.5, textures: 10},
    Sprite {x: 3.5, y: 20.5, textures: 10},
    Sprite {x: 3.5, y: 14.5, textures: 10},
    Sprite {x: 14.5, y: 20.5, textures: 10},
    Sprite {x: 18.5, y: 10.5, textures: 9},
    Sprite {x: 18.5, y: 11.5, textures: 9},
    Sprite {x: 18.5, y: 12.5, textures: 9},
    Sprite {x: 21.5, y: 1.5, textures: 8},
    Sprite {x: 15.5, y: 1.5, textures: 8},
    Sprite {x: 16.0, y: 1.8, textures: 8},
    Sprite {x: 16.2, y: 1.2, textures: 8},
    Sprite {x: 3.5, y: 2.5, textures: 8},
    Sprite {x: 9.5, y: 15.5, textures: 8},
    Sprite {x: 10.0, y: 15.1, textures: 8},
    Sprite {x: 10.5, y: 15.8, textures: 8},
    ];
