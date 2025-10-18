use cgmath::Vector3;
use fastnoise_lite::*;

use crate::core::{
    block::{Block, BlockType},
    chunk::{CHUNK_SIZE, CHUNK_VOLUME, Chunk},
};

pub struct TerrainGenerator;

impl TerrainGenerator {
    pub fn heightmap_advanced(
        world_pos: &Vector3<i64>,
        seed: u32,
        blocks: &mut [Block; CHUNK_VOLUME],
    ) {
        const FRACTAL_SCALE_XY: f32 = 0.0005;
        const WARP_SCALE_XY: f32 = 0.0015;
        const SCALE_Z: f32 = 150.0;
        if (world_pos.z * CHUNK_SIZE as i64) > SCALE_Z as i64 {
            return;
        }
        if ((world_pos.z + 1) * CHUNK_SIZE as i64) < -SCALE_Z as i64 {
            for i in 0..CHUNK_VOLUME {
                blocks[i] = Block::new(BlockType::Stone)
            }
            return;
        }

        let mut warp_gen = FastNoiseLite::with_seed(seed as i32);
        warp_gen.set_frequency(Some(WARP_SCALE_XY));
        warp_gen.set_domain_warp_amp(Some(600.0));

        let mut noise_gen = FastNoiseLite::with_seed(seed as i32);
        noise_gen.set_fractal_type(Some(FractalType::FBm));
        noise_gen.set_fractal_octaves(Some(7));
        noise_gen.set_frequency(Some(FRACTAL_SCALE_XY));

        let mut surface_get = FastNoiseLite::with_seed(seed as i32);
        surface_get.set_noise_type(Some(NoiseType::OpenSimplex2));
        surface_get.set_frequency(Some(0.01));

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let (x_warp, y_warp) = warp_gen.domain_warp_2d(
                    (world_pos.x * CHUNK_SIZE as i64 + x as i64) as f32,
                    (world_pos.y * CHUNK_SIZE as i64 + y as i64) as f32,
                );
                // let (x_, y_) = (
                //     (world_pos.x * CHUNK_SIZE as i64 + x as i64) as f32,
                //     (world_pos.y * CHUNK_SIZE as i64 + y as i64) as f32
                // );
                let noise_value = noise_gen.get_noise_2d(x_warp, y_warp);
                let height = noise_value;
                let surfact_value = surface_get.get_noise_2d(x_warp, y_warp);

                for z in 0..CHUNK_SIZE {
                    let idx = Chunk::index(x, y, z);
                    let z = z as i64 + world_pos.z * CHUNK_SIZE as i64;
                    if (z as f32) / SCALE_Z < height {
                        if surfact_value < -0.95 {
                            blocks[idx] = Block::new(BlockType::Stone)
                        } else if surfact_value < -0.90 {
                            blocks[idx] = Block::new(BlockType::Dirt)
                        } else {
                            blocks[idx] = Block::new(BlockType::Grass)
                        }
                    };
                }
            }
        }
    }
}
