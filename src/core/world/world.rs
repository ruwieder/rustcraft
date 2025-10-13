use crate::core::{
    render::{face_gen::generate_face, greedy_mesher::GreedyMesher},
    *,
};
use cgmath::Vector3;
use rand::RngCore;
use std::{collections::HashMap, time::Instant};

const DEFAULT_BLOCK_ID: u16 = 20;
const FACE_CULLING: bool = true;

pub struct World {
    pub chunks: HashMap<(i64, i64, i64), Chunk>,
    pub seed: u32,
}

impl World {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut world = Self {
            chunks: HashMap::new(),
            seed: rng.next_u32(),
        };
        let time = Instant::now();
        for x in -80..=80 {
            for y in -80..=80 {
                for z in 0..=2 {
                    world.load_chunks(x, y, z);
                }
            }
        }
        println!("chunk loading: {:.2} seconds", (Instant::now() - time).as_secs_f32());
        // world.load_chunks(0, 0, 0);
        // world.load_chunks(1, 0, 0);
        // world.load_chunks(-1, -1, 0);
        // world.load_chunks(1, -1, 0);
        // world.load_chunks(-1, 1, 0);
        // world.load_chunks(1, 1, 0);
        world
    }

    pub fn load_chunks(&mut self, x: i64, y: i64, z: i64) {
        let key = (x, y, z);
        self.chunks.entry(key).or_insert_with(
            // || Chunk::new_flat(Vector3::new(x, y, z), DEFAULT_COLOR, DEFAULT_BLOCK_ID)
            || Chunk::terrain_gen(Vector3::new(x, y, z), self.seed),
        );
    }

    pub fn get_chunk(&self, world_pos: &Vector3<i64>) -> Option<&Chunk> {
        let chunk_idx = (
            world_pos.x.div_euclid(CHUNK_SIZE as i64),
            world_pos.y.div_euclid(CHUNK_SIZE as i64),
            world_pos.z.div_euclid(CHUNK_SIZE as i64),
        );
        self.chunks.get(&chunk_idx)
    }

    pub fn get_block(&self, world_pos: Vector3<i64>) -> Option<Block> {
        let chunk = self.get_chunk(&world_pos);

        if let Some(chunk) = chunk {
            chunk.get_from_world_pos(world_pos)
        } else {
            None
        }
    }

    pub fn build_mesh_naive(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut index_offset = 0u32;

        for (chunk_pos, chunk) in &self.chunks {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let idx = Chunk::index(x, y, z);
                        let block = chunk.blocks[idx];

                        if block.is_transpose() {
                            continue;
                        }

                        let world_pos = Vector3::new(
                            (chunk_pos.0 * CHUNK_SIZE as i64 + x as i64) as f32,
                            (chunk_pos.1 * CHUNK_SIZE as i64 + y as i64) as f32,
                            (chunk_pos.2 * CHUNK_SIZE as i64 + z as i64) as f32,
                        );
                        let directions = [
                            Vector3::new(1.0, 0.0, 0.0),  // Front
                            Vector3::new(-1.0, 0.0, 0.0), // Back
                            Vector3::new(0.0, 1.0, 0.0),  // Right
                            Vector3::new(0.0, -1.0, 0.0), // Left
                            Vector3::new(0.0, 0.0, 1.0),  // Top
                            Vector3::new(0.0, 0.0, -1.0), // Bottom
                        ];
                        for dir in directions {
                            if !FACE_CULLING || self.is_face_exposed(world_pos, dir) {
                                let (v, mut i) =
                                    generate_face(world_pos, dir, block.id, 1.0, 1.0);

                                for idx in &mut i {
                                    *idx += index_offset;
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

    pub fn build_mesh_greedy(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u32;
        let time = Instant::now();
        for (chunk_pos, chunk) in &self.chunks {
            if !chunk.is_rendered {
                continue;
            }
            let chunk_vertices = GreedyMesher::build_mesh(chunk, self);
            let vertex_count = chunk_vertices.0.len();
            // Transform vertices to world coordinates
            for mut vertex in chunk_vertices.0 {
                vertex.pos[0] += chunk_pos.0 as f32 * CHUNK_SIZE as f32;
                vertex.pos[1] += chunk_pos.1 as f32 * CHUNK_SIZE as f32;
                vertex.pos[2] += chunk_pos.2 as f32 * CHUNK_SIZE as f32;
                vertices.push(vertex);
            }

            for index in chunk_vertices.1 {
                indices.push(index + index_offset);
            }

            index_offset += vertex_count as u32;
        }
        println!("generated mesh in {:.2} seconds", (Instant::now() - time).as_secs_f32());
        (vertices, indices)
    }

    pub fn is_face_exposed(&self, pos: Vector3<f32>, dir: Vector3<f32>) -> bool {
        let neighbor = Vector3::new(
            (pos.x + dir.x) as i64,
            (pos.y + dir.y) as i64,
            (pos.z + dir.z) as i64,
        );
        let chunk = self.get_chunk(&neighbor);
        if let Some(chunk) = chunk
            && chunk.is_rendered
        {
            let block = chunk.get_from_world_pos(neighbor);
            block.unwrap_or(Block::air()).is_transpose()
        } else {
            true
        }
    }
}
