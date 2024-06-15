const MAX_DEPTH: usize = 16;
use ahash::AHashMap;
fn hash_function(x: usize, y: usize, z: usize) -> usize {
    const PRIME1: usize = 73856093;
    const PRIME2: usize = 19349663;
    const PRIME3: usize = 83492791;

    ((x).wrapping_mul(PRIME1) ^ 
     (y).wrapping_mul(PRIME2) ^ 
     (z).wrapping_mul(PRIME3)) & 0xFFFFFFFFFFFF
}

#[derive(Clone, Debug)]
pub struct Voxel {
    pub color: u32,
}
#[derive(Clone, Debug)]
struct SpatialHashTable {
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
struct OctreeNode {
    children: [Option<Box<OctreeNode>>; 8],
    voxel: Option<Voxel>,
}

impl OctreeNode {
    fn new() -> Self {
        OctreeNode {
            children: Default::default(),
            voxel: None,
        }
    }

    fn insert(&mut self, depth: usize, x: usize, y: usize, z: usize, voxel: Voxel) {
        if depth == MAX_DEPTH {
            self.voxel = Some(voxel);
        } else {
            let index = ((x >> (MAX_DEPTH - depth - 1)) & 1) << 2 |
                        ((y >> (MAX_DEPTH - depth - 1)) & 1) << 1 |
                        ((z >> (MAX_DEPTH - depth - 1)) & 1);

            if self.children[index].is_none() {
                self.children[index] = Some(Box::new(OctreeNode::new()));
            }
            self.children[index].as_mut().unwrap().insert(depth + 1, x, y, z, voxel);
        }
    }
}

#[derive(Clone, Debug)]
pub struct SparseVoxelOctree {
    root: OctreeNode,
    spatial_hash_table: SpatialHashTable,
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
}