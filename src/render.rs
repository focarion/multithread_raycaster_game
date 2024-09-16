use crate::voxel::{SparseVoxelOctree, VoxelColor, MAX_DEPTH};
use rayon::prelude::*;

#[derive(Clone)]
pub struct Renderer {
    pub buffer: Vec<u32>,
    pub render_width: usize,
    pub render_height: usize,

}


impl Renderer {
    pub fn new(render_width: usize, render_height: usize) -> Self {
        Self {
            buffer: vec![0; render_width * render_height],
            render_width,
            render_height,
        }
    }
    
    pub fn render_voxels(renderer: &mut Renderer, svo: &SparseVoxelOctree, threads: usize) {
        let render_width = renderer.render_width;
        let render_height = renderer.render_height;
        let buffer_size = render_width * render_height;
        let block_size = (buffer_size + threads - 1) / threads;
    
        // Use par_chunks_mut to process the buffer in parallel
        puffin::profile_scope!("par_chunk");
        renderer.buffer.par_chunks_mut(block_size).enumerate().for_each(|(i, buffer_slice)| {
            let start_pixel = i * block_size;
            let start_row = start_pixel / render_width;
    
            // Each thread processes its buffer slice
            if is_x86_feature_detected!("avx512f") {
                puffin::profile_scope!("render_simd");
                Avx512::render_simd(
                    buffer_slice,
                    svo,
                    render_width,
                    render_height,
                    start_row,
                );
            } else if is_x86_feature_detected!("avx2") {
                Avx2::render_simd(
                    buffer_slice,
                    svo,
                    render_width,
                    render_height,
                    start_row,
                );
            } else if is_x86_feature_detected!("sse2") {
                Sse2::render_simd(
                    buffer_slice,
                    svo,
                    render_width,
                    render_height,
                    start_row,
                );
            } else {
                render_default(
                    buffer_slice,
                    svo,
                    render_width,
                    render_height,
                    start_row,
                );
            }
        });
    }    
}

trait RenderSimd {
    const LANES: usize;

    fn render_simd(
        buffer: &mut [u32],
        svo: &SparseVoxelOctree,
        screen_width: usize,
        block_height: usize,
        start_row: usize,
    );
}

struct Avx512;
struct Avx2;
struct Sse2;

impl RenderSimd for Avx512 {
    const LANES: usize = 16;

    fn render_simd(
        buffer_slice: &mut [u32],
        svo: &SparseVoxelOctree,
        screen_width: usize,
        screen_height: usize,
        start_row: usize,
    ) {
        let voxel_range = 32;
        let voxel_size = screen_width.min(screen_height) as f32 / voxel_range as f32;
        let scale = screen_width.min(screen_height) as f32 / voxel_range as f32;

        let mut voxel_cache: Vec<Option<VoxelColor>> =
            vec![None; voxel_range * voxel_range * voxel_range];

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
                                let screen_x = (x as f32 * scale) as usize;
                                let screen_y = (y as f32 * scale) as usize;

                                let min_x = screen_x;
                                let max_x = (screen_x + voxel_size as usize).min(screen_width);
                                let min_y = screen_y;
                                let max_y = (screen_y + voxel_size as usize).min(screen_height);

                                let color_val = (color.a as u32) << 24
                                    | (color.r as u32) << 16
                                    | (color.g as u32) << 8
                                    | (color.b as u32);

                                for y in min_y..max_y {
                                    if y < start_row || y >= start_row + buffer_slice.len() / screen_width {
                                        continue;
                                    }
                                    let row_offset = (y - start_row) * screen_width;
                                    for x in min_x..max_x {
                                        let index = row_offset + x;
                                        if index < buffer_slice.len() {
                                            buffer_slice[index] = color_val;
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

    fn render_simd(
        buffer_slice: &mut [u32],
        svo: &SparseVoxelOctree,
        screen_width: usize,
        screen_height: usize,
        start_row: usize,
    ) {
        let voxel_range = 32;
        let voxel_size = screen_width.min(screen_height) as f32 / voxel_range as f32;
        let scale = screen_width.min(screen_height) as f32 / voxel_range as f32;

        let mut voxel_cache: Vec<Option<VoxelColor>> =
            vec![None; voxel_range * voxel_range * voxel_range];

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
                                let screen_x = (x as f32 * scale) as usize;
                                let screen_y = (y as f32 * scale) as usize;

                                let min_x = screen_x;
                                let max_x = (screen_x + voxel_size as usize).min(screen_width);
                                let min_y = screen_y;
                                let max_y = (screen_y + voxel_size as usize).min(screen_height);

                                let color_val = (color.a as u32) << 24
                                    | (color.r as u32) << 16
                                    | (color.g as u32) << 8
                                    | (color.b as u32);

                                for y in min_y..max_y {
                                    if y < start_row || y >= start_row + buffer_slice.len() / screen_width {
                                        continue;
                                    }
                                    let row_offset = (y - start_row) * screen_width;
                                    for x in min_x..max_x {
                                        let index = row_offset + x;
                                        if index < buffer_slice.len() {
                                            buffer_slice[index] = color_val;
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
    fn render_simd(
        buffer_slice: &mut [u32],
        svo: &SparseVoxelOctree,
        screen_width: usize,
        screen_height: usize,
        start_row: usize,
    ) {
        let voxel_range = 32;
        let voxel_size = screen_width.min(screen_height) as f32 / voxel_range as f32;
        let scale = screen_width.min(screen_height) as f32 / voxel_range as f32;

        let mut voxel_cache: Vec<Option<VoxelColor>> =
            vec![None; voxel_range * voxel_range * voxel_range];

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
                                let screen_x = (x as f32 * scale) as usize;
                                let screen_y = (y as f32 * scale) as usize;

                                let min_x = screen_x;
                                let max_x = (screen_x + voxel_size as usize).min(screen_width);
                                let min_y = screen_y;
                                let max_y = (screen_y + voxel_size as usize).min(screen_height);

                                let color_val = (color.a as u32) << 24
                                    | (color.r as u32) << 16
                                    | (color.g as u32) << 8
                                    | (color.b as u32);

                                for y in min_y..max_y {
                                    if y < start_row || y >= start_row + buffer_slice.len() / screen_width {
                                        continue;
                                    }
                                    let row_offset = (y - start_row) * screen_width;
                                    for x in min_x..max_x {
                                        let index = row_offset + x;
                                        if index < buffer_slice.len() {
                                            buffer_slice[index] = color_val;
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
fn render_default(
    buffer_slice: &mut [u32],
    svo: &SparseVoxelOctree,
    screen_width: usize,
    screen_height: usize,
    start_row: usize,
) {
    let voxel_range = 32;
    let voxel_size = screen_width.min(screen_height) as f32 / voxel_range as f32;
    let scale = screen_width.min(screen_height) as f32 / voxel_range as f32;

    let mut voxel_cache: Vec<Option<VoxelColor>> =
        vec![None; voxel_range * voxel_range * voxel_range];

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
                            let screen_x = (x as f32 * scale) as usize;
                            let screen_y = (y as f32 * scale) as usize;

                            let min_x = screen_x;
                            let max_x = (screen_x + voxel_size as usize).min(screen_width);
                            let min_y = screen_y;
                            let max_y = (screen_y + voxel_size as usize).min(screen_height);

                            let color_val = (color.a as u32) << 24
                                | (color.r as u32) << 16
                                | (color.g as u32) << 8
                                | (color.b as u32);

                            for y in min_y..max_y {
                                if y < start_row || y >= start_row + buffer_slice.len() / screen_width {
                                    continue;
                                }
                                let row_offset = (y - start_row) * screen_width;
                                for x in min_x..max_x {
                                    let index = row_offset + x;
                                    if index < buffer_slice.len() {
                                        buffer_slice[index] = color_val;
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

