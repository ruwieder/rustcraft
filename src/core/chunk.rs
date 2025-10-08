pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
use cgmath::Vector3;

use crate::core::Voxel;

pub struct Chunk {
    pub voxels: [Voxel; CHUNK_VOLUME],
    pub _pos: Vector3<i64>,
    // pub is_empty: bool
}

#[allow(dead_code)]
impl Chunk {
    pub fn new_fill(pos: Vector3<i64>, color: (u8, u8, u8)) -> Self {
        let voxels = [Voxel::new(color); CHUNK_VOLUME];
        Chunk { voxels, _pos: pos }
    }
    
    pub fn new_flat(pos: Vector3<i64>, color: (u8, u8, u8)) -> Self {
        let mut voxels = [Voxel::new((0, 0, 0)); CHUNK_VOLUME];
        for i in 0..(CHUNK_SIZE * CHUNK_SIZE) {
            voxels[i].color = (color.0 * (i%2) as u8, color.1 * ((i+1) % 2) as u8, color.2);
        }
        Chunk { voxels, _pos: pos }
    }
    
    pub const fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x
    }
    
    pub fn try_get(&self, x: usize, y: usize, z: usize) -> Option<Voxel> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            Some(self.voxels[Self::index(x, y, z)])
        } else { None }
    }
}