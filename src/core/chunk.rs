pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
use cgmath::Vector3;

use crate::core::{Block, world::terrain_generator::TerrainGenerator};

pub struct Chunk {
    pub blocks: [Block; CHUNK_VOLUME],
    pub _pos: Vector3<i64>,
    // pub is_empty: bool
    pub is_rendered: bool,
}

#[allow(dead_code)]
impl Chunk {
    pub fn new_empty(pos: Vector3<i64>) -> Self {
        let blocks = [Block::air(); CHUNK_VOLUME];
        Chunk {
            blocks,
            _pos: pos,
            is_rendered: true,
        }
    }

    pub fn new_fill(pos: Vector3<i64>, color: (u8, u8, u8), block_id: u16) -> Self {
        let blocks = [Block::new(color, block_id); CHUNK_VOLUME];
        Chunk {
            blocks,
            _pos: pos,
            is_rendered: true,
        }
    }

    pub fn new_flat(pos: Vector3<i64>, color: (u8, u8, u8), block_id: u16) -> Self {
        let mut blocks = [Block::air(); CHUNK_VOLUME];
        for block in blocks.iter_mut().take(CHUNK_SIZE * CHUNK_SIZE) {
            block.color = color;
            block.id = block_id;
        }
        blocks[CHUNK_SIZE * CHUNK_SIZE + 1].color = color;
        blocks[CHUNK_SIZE * CHUNK_SIZE + 1].id = block_id;
        Chunk {
            blocks,
            _pos: pos,
            is_rendered: true,
        }
    }

    pub fn terrain_gen(world_pos: Vector3<i64>, seed: u32) -> Self {
        let blocks: [Block; CHUNK_VOLUME] = TerrainGenerator::heightmap(&world_pos, seed);
        
        Chunk {
            blocks,
            _pos: world_pos,
            is_rendered: true,
        }
    }

    pub const fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }
    
    pub const fn from_index(i: usize) -> (usize, usize, usize) {
        (
            i % CHUNK_SIZE,
            (i / CHUNK_SIZE) % CHUNK_SIZE,
            i / (CHUNK_SIZE * CHUNK_SIZE)
        )
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
