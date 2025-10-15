use cgmath::Vector3;
use fastnoise_lite::*;

use crate::core::{
    block::Block,
    chunk::{CHUNK_SIZE, CHUNK_VOLUME, Chunk},
};

pub struct TerrainGenerator;


impl TerrainGenerator {
    pub fn heightmap_advanced(
        world_pos: &Vector3<i64>,
        seed: u32,
        blocks: &mut [Block; CHUNK_VOLUME],
    ) {
        const BLOCK_ID: u16 = 0;
        const SCALE_XY: f32 = 0.001;
        const SCALE_Z: f32 = 24.0;
        if (world_pos.z * CHUNK_SIZE as i64) > SCALE_Z as i64 {
            return;
        }
        if (world_pos.z + 1 * CHUNK_SIZE as i64) < SCALE_Z as i64 {
            for i in 0..CHUNK_VOLUME {
                blocks[i].id = BLOCK_ID;
            }
            return;
        }
        let mut noise_gen = FastNoiseLite::with_seed(seed as i32);
        noise_gen.set_fractal_type(Some(FractalType::FBm));
        noise_gen.set_fractal_octaves(Some(1));
        noise_gen.set_frequency(Some(SCALE_XY));
        for x in 0..CHUNK_SIZE {for y in 0..CHUNK_SIZE {
            let height = noise_gen.get_noise_2d(
                (world_pos.x * CHUNK_SIZE as i64 + x as i64) as f32, 
                (world_pos.y * CHUNK_SIZE as i64 + y as i64) as f32
            );
            
            for z in 0..CHUNK_SIZE {
                let idx = Chunk::index(x, y, z);
                let z = z as i64 + world_pos.z * CHUNK_SIZE as i64;
                if (z as f32) / SCALE_Z < height {
                    blocks[idx] = Block{ id: BLOCK_ID };
                };
            }
        }}
    }
    
}
