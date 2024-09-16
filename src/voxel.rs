use ahash::AHashMap;
use std::collections::VecDeque;
pub const MAX_DEPTH: usize = 16;
fn hash_function(x: usize, y: usize, z: usize) -> usize {
    const PRIME1: usize = 73856093;
    const PRIME2: usize = 19349663;
    const PRIME3: usize = 83492791;

    ((x).wrapping_mul(PRIME1) ^ (y).wrapping_mul(PRIME2) ^ (z).wrapping_mul(PRIME3))
        & 0xFFFFFFFFFFFF
}

#[derive(Clone, Debug)]
pub struct Voxel {
    pub color: VoxelColor,
}
#[derive(Clone, Debug, Copy)]
pub struct VoxelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
#[derive(Clone, Debug)]
pub struct SpatialHashTable {
    table: AHashMap<usize, Voxel>,
}

impl SpatialHashTable {
    fn new() -> Self {
        SpatialHashTable {
            table: AHashMap::new(),
        }
    }

    fn insert(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let hash = hash_function(x, y, z);
        self.table.insert(hash, voxel);
    }

    fn get(&self, x: usize, y: usize, z: usize) -> Option<&Voxel> {
        let hash = hash_function(x, y, z);
        self.table.get(&hash)
    }

    fn remove(&mut self, x: usize, y: usize, z: usize) {
        let hash = hash_function(x, y, z);
        self.table.remove(&hash);
    }
}
#[derive(Clone, Debug)]
pub struct OctreeNode {
    pub children: [Option<Box<OctreeNode>>; 8],
    pub bitmask: u8,
    pub voxel: Option<Voxel>,
}

impl OctreeNode {
    fn new() -> Self {
        OctreeNode {
            children: Default::default(),
            bitmask: 0,
            voxel: None,
        }
    }

    fn insert(&mut self, depth: usize, x: usize, y: usize, z: usize, voxel: Voxel) {
        if depth >= MAX_DEPTH {
            return;
        }

        let shift = MAX_DEPTH - depth - 1;
        let index = ((x >> shift) & 1) << 2 | ((y >> shift) & 1) << 1 | ((z >> shift) & 1);

        let child = self.children[index].get_or_insert_with(|| {
            self.bitmask |= 1 << index;
            Box::new(OctreeNode::new())
        });
        child.insert(depth + 1, x, y, z, voxel);
    }

    fn oc_dfs(&self, visit: &mut dyn FnMut(&OctreeNode, usize, usize, usize, usize)) {
        let mut stack = vec![(self, 0, 0, 0, 0)]; // Node, depth, x, y, z

        while let Some((node, depth, x, y, z)) = stack.pop() {
            visit(node, depth, x, y, z);

            if depth < MAX_DEPTH {
                let shift = MAX_DEPTH - depth - 1;
                for i in (0..8).rev() {
                    if let Some(ref child) = node.children[i] {
                        let child_x = x | (((i & 4) >> 2) << shift);
                        let child_y = y | (((i & 2) >> 1) << shift);
                        let child_z = z | ((i & 1) << shift);
                        stack.push((child, depth + 1, child_x, child_y, child_z));
                    }
                }
            }
        }
    }
    fn oc_bfs(&self, visit: &mut dyn FnMut(&OctreeNode, usize)) {
        let mut queue: VecDeque<(&OctreeNode, usize)> = VecDeque::with_capacity(1024);
        queue.push_back((self, 0));

        while let Some((node, depth)) = queue.pop_front() {
            visit(node, depth);

            for child in &node.children {
                if let Some(ref child_node) = *child {
                    queue.push_back((child_node, depth + 1));
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct SparseVoxelOctree {
    pub root: OctreeNode,
    pub spatial_hash_table: SpatialHashTable,
}

impl SparseVoxelOctree {
    pub fn new() -> Self {
        SparseVoxelOctree {
            root: OctreeNode::new(),
            spatial_hash_table: SpatialHashTable::new(),
        }
    }

    pub fn insert(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        self.spatial_hash_table.insert(x, y, z, voxel.clone());
        self.root.insert(0, x, y, z, voxel);
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&Voxel> {
        self.spatial_hash_table.get(x, y, z)
    }

    pub fn remove(&mut self, x: usize, y: usize, z: usize) {
        self.spatial_hash_table.remove(x, y, z);
    }
    pub fn dfs(&self, visit: &mut dyn FnMut(&OctreeNode, usize, usize, usize, usize)) {
        self.root.oc_dfs(visit);
    }
    pub fn bfs(&self, visit: &mut dyn FnMut(&OctreeNode, usize)) {
        self.root.oc_bfs(visit);
    }
}