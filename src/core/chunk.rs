pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
use cgmath::Vector3;

use crate::core::Block;

pub struct Chunk {
    pub blocks: [Block; CHUNK_VOLUME],
    pub _pos: Vector3<i64>,
    // pub is_empty: bool
    pub is_rendered: bool
}

#[allow(dead_code)]
impl Chunk {
    pub fn new_empty(pos: Vector3<i64>) -> Self {
        let blocks = [Block::air(); CHUNK_VOLUME];
        Chunk { blocks, _pos: pos, is_rendered: true }
    }
    
    pub fn new_fill(pos: Vector3<i64>, color: (u8, u8, u8), tex_id: u16) -> Self {
        let blocks = [Block::new(color, tex_id); CHUNK_VOLUME];
        Chunk{ blocks, _pos: pos, is_rendered: true }
    }
    
    pub fn new_flat(pos: Vector3<i64>, color: (u8, u8, u8), tex_id: u16) -> Self {
        let mut blocks = [Block::air(); CHUNK_VOLUME];
        for i in 0..(CHUNK_SIZE * CHUNK_SIZE) {
            blocks[i].color = (color.0 * (i%2) as u8, color.1 * ((i+1) % 2) as u8, color.2);
            blocks[i].tex_id = tex_id;
        }
        Chunk { blocks, _pos: pos, is_rendered: true }
    }
    
    pub const fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x
    }
    
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        Some(self.blocks[Self::index(x, y, z)])
    }
    
    pub fn get_from_world_pos(&self, world_pos: Vector3<i64>) -> Option<Block> {
        let x = world_pos.x.rem_euclid(CHUNK_SIZE as i64) as usize;
        let y = world_pos.y.rem_euclid(CHUNK_SIZE as i64) as usize;
        let z = world_pos.z.rem_euclid(CHUNK_SIZE as i64) as usize;
        self.get(x, y, z)
    }
}