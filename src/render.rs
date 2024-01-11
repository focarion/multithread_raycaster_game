use crate::{game_state::Player, assets::{Linedef, Assets/* , TextureId */}};

#[derive(Clone, Debug)]
pub enum BSPTree {
    Node {
        splitter: Linedef,
        front: Box<BSPTree>,
        back: Box<BSPTree>,
    },
    Leaf(Vec<Linedef>),
}
impl BSPTree {
    pub fn build(linedefs: Vec<Linedef>) -> Self {
        if linedefs.is_empty() {
            return BSPTree::Leaf(Vec::new());
        }
        let splitter = linedefs[0];
        let mut front_lines = Vec::new();
        let mut back_lines = Vec::new();
        
        front_lines.push(splitter);  // Include the first wall in the front_lines
        
        for line in &linedefs[1..] {
            let start_side = splitter.cross_product(&line.start);
            let end_side = splitter.cross_product(&line.end);
            if start_side >= 0.0 && end_side >= 0.0 {
                front_lines.push(*line);
            } else if start_side <= 0.0 && end_side <= 0.0 {
                back_lines.push(*line);
            } else {
                front_lines.push(*line);
                back_lines.push(*line);
            }
        }
        
        BSPTree::Node {
            splitter,
            front: Box::new(BSPTree::Leaf(front_lines)),
            back: Box::new(BSPTree::Leaf(back_lines)),
        }
    }
    pub fn traverse_and_render_polygons(&self, player: &Player, buffer: &mut Vec<u32>, zbuffer: &mut Vec<f64>, screen_width: usize, screen_height: usize, assets: &Assets) {
        match self {
            BSPTree::Node { splitter: _, front, back } => {
                front.traverse_and_render_polygons(player, buffer, zbuffer, screen_width, screen_height, assets);
                back.traverse_and_render_polygons(player, buffer, zbuffer, screen_width, screen_height, assets);
            },
            BSPTree::Leaf(linedefs) => {
                for x in 0..screen_width {
                    for linedef in linedefs {
                        if let Some((draw_start, draw_end, perp_wall_dist, intersection_x)) = Renderer::ray_wall_intersection(player, linedef, x, screen_width, screen_height) {
                            let wall_length = ((linedef.end.x - linedef.start.x).powi(2) + (linedef.end.y - linedef.start.y).powi(2)).sqrt();
                            let texture = &assets.textures[linedef.texture as usize];

                            let is_vertical_wall = linedef.start.x == linedef.end.x;
                
                            let tex_x = if is_vertical_wall {
                                let wall_span = (linedef.end.y - linedef.start.y).abs();
                                let relative_intersection_y = (intersection_x - linedef.start.y).abs() / wall_span;
                                let normalized_position = (relative_intersection_y % 1.0 + 1.0) % 1.0;
                                (normalized_position * texture.width as f64).floor() as u32
                            } else {
                                let relative_intersection_x = (intersection_x - linedef.start.x).abs() / wall_length;
                                let normalized_position = (relative_intersection_x % 1.0 + 1.0) % 1.0;
                                (normalized_position * texture.width as f64).floor() as u32
                            };
                
                            let tex_x = std::cmp::min(tex_x, texture.width - 1);
                            for y in (draw_start as usize).max(0)..(draw_end as usize).min(screen_height) {
                                let index = y * screen_width + x;
                                if perp_wall_dist < zbuffer[index] {
                                    
                                    let tex_y = ((y as f64 - draw_start) / (draw_end - draw_start) * linedef.height).fract() * texture.height as f64;
                                    
                                    let tex_index = (tex_y as u32 * texture.width + tex_x as u32) as usize;
                                    buffer[index] = texture.data[tex_index];
                                    
                                    zbuffer[index] = perp_wall_dist;
                                }
                            }
                        }
                    }
                }
            }
        }
    }    
    pub fn check_collision(&self, pos: (f64, f64, f64), radius: f64) -> bool {
        match self {
            BSPTree::Node { splitter, front, back } => {
                let distance = splitter.distance_to_point(pos);
                
                if distance.abs() < radius {
                    if pos.2 < splitter.height && splitter.is_within_boundaries(pos) {
                        return true;
                    }
                    return front.check_collision(pos, radius) || back.check_collision(pos, radius);
                }
                
                if distance > 0.0 {
                    front.check_collision(pos, radius)
                } else {
                    back.check_collision(pos, radius)
                }
            },
            BSPTree::Leaf(linedefs) => {
                for linedef in linedefs {
                    let distance = linedef.distance_to_point(pos);
                    if distance < radius && pos.2 < linedef.height && linedef.is_within_boundaries(pos) {
                        return true;
                    }
                }
                false
            }
        }
    }
    
    
}
pub struct Renderer {
    pub buffer: Vec<u32>,
    pub zbuffer: Vec<f64>,
    pub width: usize,
    pub height: usize,
}
impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            buffer: vec![0; width * height],
            zbuffer: vec![f64::INFINITY; width * height],
            width,
            height,
        }
    }

    pub fn raycast_and_render_polygons(&mut self, bsp_tree: &BSPTree, player: &Player, assets: &Assets, screen_width: usize, screen_height: usize) {
        self.zbuffer.fill(f64::INFINITY);
        self.buffer.fill(0);
        bsp_tree.traverse_and_render_polygons(player, &mut self.buffer, &mut self.zbuffer, screen_width, screen_height, assets);
    }
    fn ray_wall_intersection(player: &Player, linedef: &Linedef, x: usize, screen_width: usize, screen_height: usize) -> Option<(f64, f64, f64, f64)> {
        let is_vertical_wall = linedef.start.x == linedef.end.x;
        if is_vertical_wall {
            let camera_x = 2.0 * x as f64 / screen_width as f64 - 1.0;
            let ray_dir = (
                player.dir.0 + player.plane.0 * camera_x,
                player.dir.1 + player.plane.1 * camera_x,
            );
        
            if ray_dir.0.abs() < 1e-10 {
                return None;
            }

            let perp_wall_dist = (linedef.start.x - player.pos.0) / ray_dir.0;

            if perp_wall_dist < 0.0 {
                return None;
            }
            
            let intersection_y = player.pos.1 + perp_wall_dist * ray_dir.1;
            
            if intersection_y < linedef.start.y.min(linedef.end.y) || intersection_y > linedef.start.y.max(linedef.end.y) {
                return None;
            }
                
            let base_wall_height = (screen_width as f64 / perp_wall_dist).abs();
            let wall_height = base_wall_height * linedef.height;
            
            let effective_floor_level = linedef.floor_level - player.pos.2;
            let height_diff_scaled = ((player.height - effective_floor_level) / perp_wall_dist) * screen_width as f64;
        
            let draw_base = height_diff_scaled;
            let mut draw_end = draw_base;
            let mut draw_start = draw_base - wall_height;
        
            draw_start += player.screen_pitch;
            draw_end += player.screen_pitch;
    
            if draw_end < 0.0 || draw_start > screen_height as f64 {
                return None;
            }
            return Some((draw_start, draw_end, perp_wall_dist, intersection_y));
        }

        let camera_x = 2.0 * x as f64 / screen_width as f64 - 1.0;
        let ray_dir = (
            player.dir.0 + player.plane.0 * camera_x,
            player.dir.1 + player.plane.1 * camera_x,
        );
    
        let delta_x = linedef.end.x - linedef.start.x;
        let delta_y = linedef.end.y - linedef.start.y;
    
        let det = -ray_dir.1 * delta_x + ray_dir.0 * delta_y;
    
        if det.abs() < 1e-10 {
            return None;
        }
    
        let t = (delta_x * (player.pos.1 - linedef.start.y) - delta_y * (player.pos.0 - linedef.start.x)) / det;
        let u = (-ray_dir.1 * (player.pos.0 - linedef.start.x) + ray_dir.0 * (player.pos.1 - linedef.start.y)) / det;
    
        if t >= 0.0 && u >= 0.0 && u <= 1.0 {
            let perp_wall_dist = t;
    
            let base_wall_height = (screen_width as f64 / perp_wall_dist).abs();
            let wall_height = base_wall_height * linedef.height;
        
            let intersection_x = linedef.start.x + u * delta_x;

            let effective_floor_level = linedef.floor_level - player.pos.2;
            let height_diff_scaled = ((player.height - effective_floor_level) / perp_wall_dist) * screen_width as f64;
        
            let draw_base = height_diff_scaled;
            let mut draw_end = draw_base;
            let mut draw_start = draw_base - wall_height;
        
            draw_start += player.screen_pitch;
            draw_end += player.screen_pitch;
    
            if draw_end < 0.0 || draw_start > screen_height as f64 {
                return None;
            }
    
            Some((draw_start, draw_end, perp_wall_dist, intersection_x))
        } else {
            None
        }
    }
}