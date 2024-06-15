use std::{collections::VecDeque, time::Duration};
use ahash::AHashMap;
use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
const MAX_DEPTH: usize = 16;
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
pub struct OctreeNode {
    children: [Option<Box<OctreeNode>>; 8],
    bitmask: u8,
    voxel: Option<Voxel>,
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
        if depth == MAX_DEPTH {
            self.voxel = Some(voxel);
            return;
        }
        
        let shift = MAX_DEPTH - depth - 1;
        let index = ((x >> shift) & 1) << 2 |
                    ((y >> shift) & 1) << 1 |
                    ((z >> shift) & 1);
    
        let child = self.children[index].get_or_insert_with(|| {
            self.bitmask |= 1 << index;
            Box::new(OctreeNode::new())
        });
        child.insert(depth + 1, x, y, z, voxel);
    }
    
    fn oc_dfs(&self, visit: &mut dyn FnMut(&OctreeNode, usize)) {
        let mut stack = vec![(self, 0)]; // Stack of nodes and their depths

        while let Some((node, depth)) = stack.pop() {
            visit(node, depth);

            // Push children onto the stack in reverse order
            for i in (0..8).rev() {
                if let Some(ref child) = node.children[i] {
                    stack.push((child, depth + 1));
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
    pub fn dfs(&self, visit: &mut dyn FnMut(&OctreeNode, usize)) {
        self.root.oc_dfs(visit);
    }
    pub fn bfs(&self, visit: &mut dyn FnMut(&OctreeNode, usize)) {
        self.root.oc_bfs(visit);
    }
}
fn bench_svo(c: &mut Criterion) {
    let mut group = c.benchmark_group("SVO");
    group.measurement_time(Duration::from_secs(15)).confidence_level(0.99);
    let mut svo = SparseVoxelOctree::new();
    for i in [20usize, 21].iter() {
        svo.insert(10, *i, 30, Voxel { color: 0xFF0000 });
        group.bench_with_input(BenchmarkId::new("Get".to_string(), i), i, |b, i| {
            b.iter(|| {black_box(svo.get(10, *i, 30))})
        });
        group.bench_with_input(BenchmarkId::new("Insert", i), i, |b, i| {
            b.iter(|| {black_box(svo.insert(5, i + i, i + (i * i) , Voxel { color: 0xFF0000 }))})
        });
        group.bench_with_input(BenchmarkId::new("DFS", i), i, |b, _i| {
            b.iter(|| {black_box(svo.dfs(&mut |node, _depth| {
                if let Some(_voxel) = &node.voxel {
                }
            }
        ))})
        });
        group.bench_with_input(BenchmarkId::new("BFS", i), i, |b, _i| {
            b.iter(|| {black_box(svo.bfs(&mut |node, _depth| {
                if let Some(_voxel) = &node.voxel {
                }
            }
        ))})
        });
    }
    group.finish();
}
criterion_group!(benches, bench_svo);
criterion_main!(benches);