use crate::voxel::{SparseVoxelOctree, VoxelColor, MAX_DEPTH};
use crossbeam::scope;

#[derive(Clone)]
pub struct Renderer {
    pub buffer: Vec<Vec<u32>>,
    pub draw_buffer: Vec<u32>,
    pub render_width: usize,
    pub render_height: usize,
    pub block_height: usize,
}

impl Renderer {
    pub fn new(render_width: usize, render_height: usize, threads: usize) -> Self {
        Self {
            draw_buffer: vec![0; render_width * render_height],
            buffer: vec![vec![0; render_width]; render_height],
            render_width,
            render_height,
            block_height: render_height / threads,
        }
    }

    pub fn render_voxels(renderer: &mut Renderer, svo: &SparseVoxelOctree, threads: usize) {
        for row in &mut renderer.buffer {
            row.fill(0);
        }

        if threads > 1 {
            let render_height = renderer.render_height;
            let block_height = renderer.block_height;
            let render_width = renderer.render_width;

            // Split the buffer into non-overlapping slices
            let buffer_slices: Vec<&mut [Vec<u32>]> = renderer.buffer.chunks_mut(block_height).collect();

            // Use scoped threads to render each slice
            scope(|s| {
                for (i, buffer_slice) in buffer_slices.into_iter().enumerate() {
                    let slice_start_row = i * block_height;
                    s.spawn(move |_| {
                        if is_x86_feature_detected!("avx512f") {
                            Avx512::render_simd(buffer_slice, svo, render_width, block_height, slice_start_row);
                        } else if is_x86_feature_detected!("avx2") {
                            Avx2::render_simd(buffer_slice, svo, render_width, block_height, slice_start_row);
                        } else if is_x86_feature_detected!("sse2") {
                            Sse2::render_simd(buffer_slice, svo, render_width, block_height, slice_start_row);
                        } else {
                            render_default(buffer_slice, svo, render_width, block_height, slice_start_row);
                        }
                    });
                }
            }).unwrap();
        } else {
            // Use single-threaded rendering
            if is_x86_feature_detected!("avx512f") {
                Avx512::render_simd(&mut renderer.buffer, svo, renderer.render_width, renderer.render_height, 0);
            } else if is_x86_feature_detected!("avx2") {
                Avx2::render_simd(&mut renderer.buffer, svo, renderer.render_width, renderer.render_height, 0);
            } else if is_x86_feature_detected!("sse2") {
                Sse2::render_simd(&mut renderer.buffer, svo, renderer.render_width, renderer.render_height, 0);
            } else {
                render_default(&mut renderer.buffer, svo, renderer.render_width, renderer.render_height, 0);
            }
        }
    }
}

trait RenderSimd {
    const LANES: usize;

    fn render_simd(buffer: &mut [Vec<u32>], svo: &SparseVoxelOctree, screen_width: usize, block_height: usize, start_row: usize);
}

struct Avx512;
struct Avx2;
struct Sse2;

impl RenderSimd for Avx512 {
    const LANES: usize = 16;

    fn render_simd(buffer: &mut [Vec<u32>], svo: &SparseVoxelOctree, screen_width: usize, block_height: usize, start_row: usize) {
        let voxel_range = 32;
        let voxel_size = screen_width.min(block_height) as f32 / voxel_range as f32;
        let scale = screen_width.min(block_height) as f32 / voxel_range as f32;

        let mut voxel_cache: Vec<Option<VoxelColor>> = vec![None; voxel_range * voxel_range * voxel_range];

        #[inline(always)]
        fn get_cache_index(x: usize, y: usize, z: usize) -> usize {
            x + y * 32 + z * 32 * 32
        }

        svo.dfs(&mut |node, depth, base_x, base_y, base_z| {
            if depth >= MAX_DEPTH || node.bitmask == 0 {
                for dx in 0..(1 << (MAX_DEPTH - depth)) {
                    for dy in 0..(1 << (MAX_DEPTH - depth)) {
                        for dz in 0..(1 << (MAX_DEPTH - depth)) {
                            let x = base_x + dx;
                            let y = base_y + dy;
                            let z = base_z + dz;

                            let cache_index = get_cache_index(x, y, z);
                            if voxel_cache[cache_index].is_none() {
                                if let Some(voxel) = svo.get(x, y, z) {
                                    voxel_cache[cache_index] = Some(voxel.color.clone());
                                }
                            }

                            if let Some(color) = &voxel_cache[cache_index] {
                                let screen_x = (x as f32 * scale) as isize;
                                let screen_y = ((y as f32 * scale) as isize) + start_row as isize;

                                let min_x = screen_x.max(0) as usize;
                                let min_y = screen_y.max(0) as usize;
                                let max_x = (screen_x + voxel_size as isize).min(screen_width as isize) as usize;
                                let max_y = (screen_y + voxel_size as isize).min(block_height as isize) as usize;

                                let color_val = (color.a as u32) << 24
                                                | (color.r as u32) << 16
                                                | (color.g as u32) << 8
                                                | (color.b as u32);

                                for final_y in min_y..max_y {
                                    for final_x in (min_x..max_x).step_by(Self::LANES) {
                                        if final_y < buffer.len() && final_x + Self::LANES <= buffer[final_y].len() {
                                            let slice = &mut buffer[final_y][final_x..final_x + Self::LANES];
                                            for i in 0..Self::LANES {
                                                if final_x + i < max_x {
                                                    slice[i] = color_val;
                                                }
                                            }
                                        } else {
                                            for i in 0..Self::LANES {
                                                if final_y < buffer.len() && final_x + i < max_x && final_x + i < buffer[final_y].len() {
                                                    buffer[final_y][final_x + i] = color_val;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

impl RenderSimd for Avx2 {
    const LANES: usize = 8;

    fn render_simd(buffer: &mut [Vec<u32>], svo: &SparseVoxelOctree, screen_width: usize, block_height: usize, start_row: usize) {
        let voxel_range = 32;
        let voxel_size = screen_width.min(block_height) as f32 / voxel_range as f32;
        let scale = screen_width.min(block_height) as f32 / voxel_range as f32;

        let mut voxel_cache: Vec<Option<VoxelColor>> = vec![None; voxel_range * voxel_range * voxel_range];

        #[inline(always)]
        fn get_cache_index(x: usize, y: usize, z: usize) -> usize {
            x + y * 32 + z * 32 * 32
        }

        svo.dfs(&mut |node, depth, base_x, base_y, base_z| {
            if depth >= MAX_DEPTH || node.bitmask == 0 {
                for dx in 0..(1 << (MAX_DEPTH - depth)) {
                    for dy in 0..(1 << (MAX_DEPTH - depth)) {
                        for dz in 0..(1 << (MAX_DEPTH - depth)) {
                            let x = base_x + dx;
                            let y = base_y + dy;
                            let z = base_z + dz;

                            let cache_index = get_cache_index(x, y, z);
                            if voxel_cache[cache_index].is_none() {
                                if let Some(voxel) = svo.get(x, y, z) {
                                    voxel_cache[cache_index] = Some(voxel.color.clone());
                                }
                            }

                            if let Some(color) = &voxel_cache[cache_index] {
                                let screen_x = (x as f32 * scale) as isize;
                                let screen_y = ((y as f32 * scale) as isize) + start_row as isize;

                                let min_x = screen_x.max(0) as usize;
                                let min_y = screen_y.max(0) as usize;
                                let max_x = (screen_x + voxel_size as isize).min(screen_width as isize) as usize;
                                let max_y = (screen_y + voxel_size as isize).min(block_height as isize) as usize;

                                let color_val = (color.a as u32) << 24
                                                | (color.r as u32) << 16
                                                | (color.g as u32) << 8
                                                | (color.b as u32);

                                for final_y in min_y..max_y {
                                    for final_x in (min_x..max_x).step_by(Self::LANES) {
                                        if final_y < buffer.len() && final_x + Self::LANES <= buffer[final_y].len() {
                                            let slice = &mut buffer[final_y][final_x..final_x + Self::LANES];
                                            for i in 0..Self::LANES {
                                                if final_x + i < max_x {
                                                    slice[i] = color_val;
                                                }
                                            }
                                        } else {
                                            for i in 0..Self::LANES {
                                                if final_y < buffer.len() && final_x + i < max_x && final_x + i < buffer[final_y].len() {
                                                    buffer[final_y][final_x + i] = color_val;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

impl RenderSimd for Sse2 {
    const LANES: usize = 4;
    fn render_simd(buffer: &mut [Vec<u32>], svo: &SparseVoxelOctree, screen_width: usize, block_height: usize, start_row: usize) {
        let voxel_range = 32;
        let voxel_size = screen_width.min(block_height) as f32 / voxel_range as f32;
        let scale = screen_width.min(block_height) as f32 / voxel_range as f32;

        let mut voxel_cache: Vec<Option<VoxelColor>> = vec![None; voxel_range * voxel_range * voxel_range];

        #[inline(always)]
        fn get_cache_index(x: usize, y: usize, z: usize) -> usize {
            x + y * 32 + z * 32 * 32
        }

        svo.dfs(&mut |node, depth, base_x, base_y, base_z| {
            if depth >= MAX_DEPTH || node.bitmask == 0 {
                for dx in 0..(1 << (MAX_DEPTH - depth)) {
                    for dy in 0..(1 << (MAX_DEPTH - depth)) {
                        for dz in 0..(1 << (MAX_DEPTH - depth)) {
                            let x = base_x + dx;
                            let y = base_y + dy;
                            let z = base_z + dz;

                            let cache_index = get_cache_index(x, y, z);
                            if voxel_cache[cache_index].is_none() {
                                if let Some(voxel) = svo.get(x, y, z) {
                                    voxel_cache[cache_index] = Some(voxel.color.clone());
                                }
                            }

                            if let Some(color) = &voxel_cache[cache_index] {
                                let screen_x = (x as f32 * scale) as isize;
                                let screen_y = ((y as f32 * scale) as isize) + start_row as isize;

                                let min_x = screen_x.max(0) as usize;
                                let min_y = screen_y.max(0) as usize;
                                let max_x = (screen_x + voxel_size as isize).min(screen_width as isize) as usize;
                                let max_y = (screen_y + voxel_size as isize).min(block_height as isize) as usize;

                                let color_val = (color.a as u32) << 24
                                                | (color.r as u32) << 16
                                                | (color.g as u32) << 8
                                                | (color.b as u32);

                                for final_y in min_y..max_y {
                                    for final_x in (min_x..max_x).step_by(Self::LANES) {
                                        if final_y < buffer.len() && final_x + Self::LANES <= buffer[final_y].len() {
                                            let slice = &mut buffer[final_y][final_x..final_x + Self::LANES];
                                            for i in 0..Self::LANES {
                                                if final_x + i < max_x {
                                                    slice[i] = color_val;
                                                }
                                            }
                                        } else {
                                            for i in 0..Self::LANES {
                                                if final_y < buffer.len() && final_x + i < max_x && final_x + i < buffer[final_y].len() {
                                                    buffer[final_y][final_x + i] = color_val;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
fn render_default(buffer: &mut [Vec<u32>], svo: &SparseVoxelOctree, screen_width: usize, block_height: usize, start_row: usize) {
    let voxel_range = 32;
    let voxel_size = screen_width.min(block_height) as f32 / voxel_range as f32;
    let scale = screen_width.min(block_height) as f32 / voxel_range as f32;

    let mut voxel_cache: Vec<Option<VoxelColor>> = vec![None; voxel_range * voxel_range * voxel_range];

        #[inline(always)]
        fn get_cache_index(x: usize, y: usize, z: usize) -> usize {
            x + y * 32 + z * 32 * 32
        }

    svo.dfs(&mut |node, depth, base_x, base_y, base_z| {
        if depth >= MAX_DEPTH || node.bitmask == 0 {
            for dx in 0..(1 << (MAX_DEPTH - depth)) {
                for dy in 0..(1 << (MAX_DEPTH - depth)) {
                    for dz in 0..(1 << (MAX_DEPTH - depth)) {
                        let x = base_x + dx;
                        let y = base_y + dy;
                        let z = base_z + dz;

                        let cache_index = get_cache_index(x, y, z);
                        if voxel_cache[cache_index].is_none() {
                            if let Some(voxel) = svo.get(x, y, z) {
                                voxel_cache[cache_index] = Some(voxel.color.clone());
                            }
                        }

                        if let Some(color) = &voxel_cache[cache_index] {
                            let screen_x = (x as f32 * scale) as isize;
                            let screen_y = ((y as f32 * scale) as isize) + start_row as isize;

                            let min_x = screen_x.max(0) as usize;
                            let min_y = screen_y.max(0) as usize;
                            let max_x = (screen_x + voxel_size as isize).min(screen_width as isize) as usize;
                            let max_y = (screen_y + voxel_size as isize).min(block_height as isize) as usize;

                            let color_val = (color.a as u32) << 24
                                            | (color.r as u32) << 16
                                            | (color.g as u32) << 8
                                            | (color.b as u32);

                            for final_y in min_y..max_y {
                                for final_x in min_x..max_x {
                                    if final_y < buffer.len() && final_x < buffer[final_y].len() {
                                        buffer[final_y][final_x] = color_val;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}
