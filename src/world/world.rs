use crate::core::{
    block::Block,
    chunk::{CHUNK_SIZE, Chunk},
    meshing::{Mesh, Vertex},
    render::renderer::Renderer,
};
use cgmath::Vector3;
use hashbrown::HashMap;
use rayon::prelude::*;
use std::{
    collections::{HashSet, VecDeque},
    time::Duration,
};

pub struct World {
    pub chunks: HashMap<(i64, i64, i64), Chunk>,
    pub meshes: HashMap<(i64, i64, i64), Mesh>,
    pub seed: u32,
    pub dirty_chunks: HashSet<(i64, i64, i64)>,
    pub need_to_load: VecDeque<(i64, i64, i64)>,
}

impl World {
    pub fn new(seed: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            meshes: HashMap::new(),
            seed,
            dirty_chunks: HashSet::new(),
            need_to_load: VecDeque::new(),
        }
    }

    pub fn update(&mut self, has_time: Duration, renderer: &mut Renderer) {
        self.loader_update(has_time, &renderer.camera);
        renderer.cleanup_unused_meshes(&self.chunks);
        self.update_meshes(renderer);
    }

    fn update_meshes(&mut self, renderer: &mut Renderer) {
        let dirty_chunks = std::mem::take(&mut self.dirty_chunks);
        let mesh_updates: Vec<((i64, i64, i64), (Vec<Vertex>, Vec<u32>))> = dirty_chunks
            .par_iter()
            .filter_map(|key| {
                let chunk = self.chunks.get(key)?;
                let mesh_data = chunk.generate_mesh(&self);
                Some((*key, mesh_data))
            })
            .collect();
        for (key, (vertices, indices)) in mesh_updates {
            if let Some(mesh) = self.meshes.get_mut(&key) {
                mesh.update(vertices, indices);
            } else {
                let mesh = Mesh::new(vertices, indices);
                self.meshes.insert(key, mesh);
            }
            renderer.on_mesh_updated(key);
            if let Some(chunk) = self.chunks.get_mut(&key) {
                chunk.is_dirty = false;
                self.dirty_chunks.remove(&key);
            }
        }
    }

    pub fn load_chunk(&mut self, x: i64, y: i64, z: i64) {
        let key = (x, y, z);
        if !self.chunks.contains_key(&key) {
            let world_pos = Vector3 { x, y, z };
            let chunk = Chunk::terrain_gen(world_pos, self.seed);
            self.chunks.insert(key, chunk);
        }
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
            Some(chunk.get_from_world_pos(world_pos))
        } else {
            None
        }
    }

    pub fn drop_chunk(&mut self, world_pos: Vector3<i64>) {
        let key = (world_pos.x, world_pos.y, world_pos.z);
        self.chunks.remove(&key);
        self.meshes.remove(&key);
        self.dirty_chunks.remove(&key);
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
            block.is_transpose()
        } else {
            true
        }
    }
}
