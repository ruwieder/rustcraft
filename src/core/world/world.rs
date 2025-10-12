use std::{collections::HashMap, ops::Range};
use cgmath::Vector3;
use crate::core::{render::vertex::generate_voxel_face, *};

const DEFAULT_COLOR: (u8, u8, u8) = (255, 100, 100);
const DEFAULT_TEX_ID: u16 = 25;

pub struct World {
    pub chunks: HashMap<(i64, i64, i64), Chunk>
}

impl World {
    pub fn new() -> Self {
        let mut world = Self{
            chunks: HashMap::new(),
        };
        for x in -3..=3 {
            for z in 0..=0 {
                world.load_chunks(x, 0, z);
            }
        }
        world
    }
    
    pub fn load_chunks(&mut self, x: i64, y: i64, z: i64) {
        let key = (x, y, z);
        if !self.chunks.contains_key(&key) {
            self.chunks.insert(key, 
                Chunk::new_flat(Vector3::new(x, y, z), DEFAULT_COLOR, DEFAULT_TEX_ID)
                // Chunk::new_fill(Vector3::new(x, y, z), DEFAULT_COLOR, DEFAULT_TEX_ID)
            );
        };
    }
    
    pub fn get_chunk(&self, world_pos: &Vector3<i64>) -> Option<&Chunk> {
        let chunk_idx = (
            world_pos.x / CHUNK_SIZE as i64,
            world_pos.y / CHUNK_SIZE as i64,
            world_pos.z / CHUNK_SIZE as i64,
        );
        self.chunks.get(&chunk_idx)
    }
    
    pub fn get_block(&self, world_pos: Vector3<i64>) -> Option<Block> {
        let chunk = self.get_chunk(&world_pos);
        
        if let Some(chunk) = chunk {
            chunk.get_from_world_pos(world_pos)
        } else { None }
    }
    
    pub fn build_mesh(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u32;

        for (chunk_pos, chunk) in &self.chunks {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let idx = Chunk::index(x, y, z);
                        let voxel = chunk.blocks[idx];
                        
                        if voxel.color == (0, 0, 0) { 
                            continue; 
                        }
                        let color = [
                            voxel.color.0 as f32 / 255.0,
                            voxel.color.1 as f32 / 255.0,
                            voxel.color.2 as f32 / 255.0,
                        ];
                        
                        let world_pos = Vector3::new(
                            (chunk_pos.0 * CHUNK_SIZE as i64 + x as i64) as f32,
                            (chunk_pos.1 * CHUNK_SIZE as i64 + y as i64) as f32, 
                            (chunk_pos.2 * CHUNK_SIZE as i64 + z as i64) as f32,
                        );
                        
                        let directions = [
                            Vector3::new(-1.0, 0.0, 0.0), // left
                            Vector3::new(1.0, 0.0, 0.0),  // right
                            Vector3::new(0.0, -1.0, 0.0), // bottom
                            Vector3::new(0.0, 1.0, 0.0),  // top
                            Vector3::new(0.0, 0.0, -1.0), // back
                            Vector3::new(0.0, 0.0, 1.0),  // front
                        ];
                        
                        for dir in directions {
                            if self.is_face_exposed(world_pos, dir) {
                                let (v, mut i) = generate_voxel_face(
                                    world_pos, color, dir, voxel.tex_id
                                );
                                
                                for idx in &mut i {
                                    *idx += index_offset as u16;
                                }
                                
                                index_offset += v.len() as u32;
                                vertices.extend(v);
                                indices.extend(i);
                            }
                        }
                    }
                }
            }
        }
        
        (vertices, indices)
    }
    
    fn is_face_exposed(&self, pos: Vector3<f32>, dir: Vector3<f32>) -> bool {
        // true
        let neighbor = Vector3::new(
            (pos.x + dir.x) as i64,
            (pos.y + dir.y) as i64,
            (pos.z + dir.z) as i64,
        );
        let chunk = self.get_chunk(&neighbor);
        if let Some(chunk) = chunk {
            let block = chunk.get_from_world_pos(neighbor);
            // !chunk.is_rendered || block.is_none() || block.unwrap().is_transpose()
            block.unwrap_or(Block::air()).is_transpose()
        } else { true }
    }
}