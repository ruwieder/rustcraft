pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
use cgmath::Vector3;

use crate::{
    core::{
        block::Block,
        meshing::{GreedyMesher, Vertex},
    },
    world::{TerrainGenerator, World},
};

pub struct Chunk {
    pub blocks: [Block; CHUNK_VOLUME],
    pub pos: Vector3<i64>,
    pub is_rendered: bool,
    pub is_dirty: bool,
}

#[allow(dead_code)]
impl Chunk {
    pub fn new_empty(pos: Vector3<i64>) -> Self {
        let blocks = [Block::air(); CHUNK_VOLUME];
        Chunk {
            blocks,
            pos,
            is_rendered: true,
            is_dirty: true,
        }
    }

    pub fn terrain_gen(world_pos: Vector3<i64>, seed: u32) -> Self {
        let mut blocks = [Block::air(); CHUNK_VOLUME];
        TerrainGenerator::heightmap_advanced(&world_pos, seed, &mut blocks);
        Chunk {
            blocks,
            pos: world_pos,
            is_rendered: true,
            is_dirty: true,
        }
    }

    pub fn generate_mesh(&self, world: &World) -> (Vec<Vertex>, Vec<u32>) {
        if !self.is_rendered {
            return (Vec::new(), Vec::new());
        }
        let (mut vertices, indices) = GreedyMesher::build_mesh(self, world);
        for v in &mut vertices {
            v.pos[0] += self.pos.x as f32 * CHUNK_SIZE as f32;
            v.pos[1] += self.pos.y as f32 * CHUNK_SIZE as f32;
            v.pos[2] += self.pos.z as f32 * CHUNK_SIZE as f32;
        }
        (vertices, indices)
    }

    #[inline(always)]
    pub const fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline(always)]
    pub const fn from_index(i: usize) -> (usize, usize, usize) {
        (
            i % CHUNK_SIZE,
            (i / CHUNK_SIZE) % CHUNK_SIZE,
            i / (CHUNK_SIZE * CHUNK_SIZE),
        )
    }

    #[inline(always)]
    pub const fn get(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[Self::index(x, y, z)]
    }

    #[inline]
    pub const fn get_from_world_pos(&self, world_pos: Vector3<i64>) -> Block {
        let x = world_pos.x.rem_euclid(CHUNK_SIZE as i64) as usize;
        let y = world_pos.y.rem_euclid(CHUNK_SIZE as i64) as usize;
        let z = world_pos.z.rem_euclid(CHUNK_SIZE as i64) as usize;
        self.get(x, y, z)
    }
}
