const BRICK_WALLS_BYTES: &'static [u8] = include_bytes!("../assets/textures/brick_walls.png");
const RED_BRICKS_BYTES: &'static [u8] = include_bytes!("../assets/textures/red_bricks.png");
const CEILING_BYTES: &'static [u8] = include_bytes!("../assets/textures/ceiling.png");
const RED_FLOORING_BYTES: &'static [u8] = include_bytes!("../assets/textures/red_flooring.png");
const BARREL_BYTES: &'static [u8] = include_bytes!("../assets/textures/barrel.png");
const LIGHT_BYTES: &'static [u8] = include_bytes!("../assets/textures/light.png");
const PILLAR_BYTES: &'static [u8] = include_bytes!("../assets/textures/pillar.png");
const CORRODED_COOPER_WALL_BYTES: &'static [u8] = include_bytes!("../assets/textures/corroded_cooper_wall.png");
const GREY_BRICKS_BYTES: &'static [u8] = include_bytes!("../assets/textures/grey_bricks.png");
pub const GAME_MAIN_FONT: &[u8] = include_bytes!("../assets/fonts/hud.otf") as &[u8];


use fontdue::Font;
use image::{Pixel, GenericImageView};
#[derive(Clone, Copy, Debug)]
pub enum TextureId {
    BrickWalls,
    RedBricks,
    Ceiling,
    RedFlooring,
    Barrel,
    Light,
    Pillar,
    CorrodedCooperWall,
    GreyBricks

}
#[derive(Clone, Debug)]
pub struct Texture {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u32>
}
pub struct Assets {
    pub textures: Vec<Texture>,
    pub fonts: Vec<Font>,
    pub sounds: [&'static [u8]; 1]
}
impl Assets {
    pub fn new() -> Self {
        let mut textures: Vec<Texture> = vec![];
        let texture_data_bytes = [
            (TextureId::BrickWalls, BRICK_WALLS_BYTES),
            (TextureId::RedBricks, RED_BRICKS_BYTES),
            (TextureId::Ceiling, CEILING_BYTES),
            (TextureId::RedFlooring, RED_FLOORING_BYTES),
            (TextureId::Barrel, BARREL_BYTES),
            (TextureId::Light, LIGHT_BYTES),
            (TextureId::Pillar, PILLAR_BYTES),
            (TextureId::CorrodedCooperWall, CORRODED_COOPER_WALL_BYTES),
            (TextureId::GreyBricks, GREY_BRICKS_BYTES)
            // Just add here new textures
        ];

        for (id, bytes) in texture_data_bytes.iter() {
            let texture = Self::load_texture(*id as u32, bytes);
            textures.push(texture);
        }
    let game_text_font = fontdue::Font::from_bytes(GAME_MAIN_FONT, fontdue::FontSettings::default()).unwrap();
    let fonts = vec![game_text_font];
    Self {
        textures,
        fonts,
        sounds: [include_bytes!("../assets/sounds/footsteps_concrete.mp3")],
    }
    }
    fn load_texture(id: u32, bytes: &[u8]) -> Texture {
        let img = image::load_from_memory(bytes).unwrap();
        let (width, height) = img.dimensions();
        let buffer: Vec<u32> = img.pixels()
            .map(|(_, _, pixel)| {
                let binding = pixel.to_rgb();
                let rgb = binding.channels();
                ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32)
            })
            .collect();
        
        Texture {
            id,
            width: width,
            height: height,
            data: buffer,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
}

impl Vertex {
    fn subtract(&self, other: &Vertex) -> Vertex {
        Vertex {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Linedef {
    pub start: Vertex,
    pub end: Vertex,
    pub height: f64,
    pub floor_level: f64,
    pub texture: TextureId,
}

impl Linedef {
    pub fn cross_product(&self, point: &Vertex) -> f64 {
        let line_vec = self.end.subtract(&self.start);
        let point_vec = point.subtract(&self.start);
        line_vec.x * point_vec.y - line_vec.y * point_vec.x
    }
    pub fn distance_to_point(&self, pos: (f64, f64, f64)) -> f64 {
        let px = pos.0;
        let py = pos.1;
        let x1 = self.start.x;
        let y1 = self.start.y;
        let x2 = self.end.x;
        let y2 = self.end.y;

        let numerator = ((x2 - x1) * (y1 - py) - (x1 - px) * (y2 - y1)).abs();
        let denominator = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();

        numerator / denominator
    }
    pub fn is_within_boundaries(&self, pos: (f64, f64, f64)) -> bool {
        let x = pos.0;
        let y = pos.1;
    
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;
    
        let t = ((x - self.start.x) * dx + (y - self.start.y) * dy) / (dx * dx + dy * dy);
    
        if t >= 0.0 && t <= 1.0 {
            let projected_x = self.start.x + t * dx;
            let projected_y = self.start.y + t * dy;
    
            let distance = ((projected_x - x).powi(2) + (projected_y - y).powi(2)).sqrt();
    
            if distance < 1e-2 {  // Adjust this threshold as needed
                return true;
            }
        }
        false
    }
    
    
}