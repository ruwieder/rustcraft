use std::collections::HashMap;
use cgmath::Vector3;
use crate::core::{render::vertex::generate_voxel_face, *};

pub struct World {
    pub chunks: HashMap<(i64, i64, i64), Chunk>
}

impl World {
    pub fn new() -> Self {
        let mut world = Self{
            chunks: HashMap::new(),
        };
        for x in -1..=1 {
            for z in -1..=1 {
                world.load_chunks(x, 0, z);
            }
        }
        world
    }
    
    pub fn load_chunks(&mut self, x: i64, y: i64, z: i64) {
        let key = (x, y, z);
        if !self.chunks.contains_key(&key) {
            self.chunks.insert(key, 
                Chunk::new_fill(Vector3::new(x, y, z), (250, 100, 100))
            );
        };
    }
    
    pub fn get_voxel(&self, world_pos: Vector3<i64>) -> Option<Voxel> {
        let chunk_x = world_pos.x / CHUNK_SIZE as i64;
        let chunk_y = world_pos.y / CHUNK_SIZE as i64;
        let chunk_z = world_pos.z / CHUNK_SIZE as i64;
    
        let local_x = (world_pos.x % CHUNK_SIZE as i64) as usize;
        let local_y = (world_pos.y % CHUNK_SIZE as i64) as usize;
        let local_z = (world_pos.z % CHUNK_SIZE as i64) as usize;
        
        let key = (chunk_x, chunk_y, chunk_z);
        if let Some(chunk) = self.chunks.get(&key) {
            chunk.try_get(local_x, local_y, local_z)
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
                        let voxel = chunk.voxels[idx];
                        
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
                                let (v, mut i) = generate_voxel_face(world_pos, color, dir);
                                
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
        let neighbor = Vector3::new(
            (pos.x + dir.x) as i64,
            (pos.y + dir.y) as i64,
            (pos.z + dir.z) as i64,
        );
        self.get_voxel(neighbor).is_none()
    }
}