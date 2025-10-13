use cgmath::Vector3;
use noise::{NoiseFn, Simplex};
use rayon::prelude::*;
use crate::core::{block::Block, chunk::{Chunk, CHUNK_SIZE, CHUNK_VOLUME}};

pub struct TerrainGenerator;

const DEFAULT_COLOR: (u8, u8, u8) = (255, 255, 0);

const HEIGHTMAP_BLOCK_ID: u16 = 15;
const HEIGHTMAP_SCALE_XY: f64 = 50.0;
const HEIGHTMAP_SCALE_Z: f64 = (CHUNK_SIZE*1) as f64;
const HEIGHTMAP_MAX: f64 = 1.0;
const HEIGHTMAP_MIN: f64 = 0.2;

const NOISE3D_SCALE: f64 = 20.0;
const NOISE3D_VALUE: f64 = 0.4;

impl TerrainGenerator {
    pub fn noise_3d(world_pos: &Vector3<i64>, seed: u32) -> [Block; CHUNK_VOLUME] {
        let mut blocks = [Block::air(); CHUNK_VOLUME];
        let noise_gen = Simplex::new(seed);
        blocks.par_iter_mut().enumerate().for_each(|(i, block)| {
            let (x, y, z) = Chunk::from_index(i);
            let global_pos = Vector3::new(
                x as i64 + world_pos.x * CHUNK_SIZE as i64,
                y as i64 + world_pos.y * CHUNK_SIZE as i64, 
                z as i64 + world_pos.z * CHUNK_SIZE as i64
            );
            let value = (noise_gen.get([
                global_pos.x as f64 / NOISE3D_SCALE,
                global_pos.y as f64 / NOISE3D_SCALE,
                global_pos.z as f64 / NOISE3D_SCALE
            ]) + 1.0) / 2.0;
            
            if value > NOISE3D_VALUE {
                *block = Block {color: DEFAULT_COLOR, id: HEIGHTMAP_BLOCK_ID};
            };
        });
        blocks
    }
    
    pub fn heightmap(world_pos: &Vector3<i64>, seed: u32) -> [Block; CHUNK_VOLUME] {
        let mut blocks = [Block::air(); CHUNK_VOLUME];
        let noise_gen = Simplex::new(seed);
        blocks.par_iter_mut().enumerate().for_each(|(i, block)| {
            let (x, y, z) = Chunk::from_index(i);
            let global_pos = Vector3::new(
                x as i64 + world_pos.x * CHUNK_SIZE as i64,
                y as i64 + world_pos.y * CHUNK_SIZE as i64, 
                z as i64 + world_pos.z * CHUNK_SIZE as i64
            );
            let value = (noise_gen.get([
                global_pos.x as f64 / HEIGHTMAP_SCALE_XY,
                global_pos.y as f64 / HEIGHTMAP_SCALE_XY,
            ]) + 1.0) / 2.0 * HEIGHTMAP_MAX;
            
            if (global_pos.z as f64) < value * HEIGHTMAP_SCALE_Z + HEIGHTMAP_MIN {
                *block = Block {color: DEFAULT_COLOR, id: HEIGHTMAP_BLOCK_ID};
            };
        });
        blocks
    }
}