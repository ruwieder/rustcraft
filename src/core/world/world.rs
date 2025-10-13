use crate::core::{ mesh::Mesh, * };
use cgmath::Vector3;
use rand::RngCore;
use std::{collections::{BTreeMap, HashMap, HashSet, VecDeque}, time::{Duration, Instant}};

pub struct World {
    pub chunks: BTreeMap<(i64, i64, i64), Chunk>,
    pub meshes: BTreeMap<(i64, i64, i64), Mesh>,
    pub seed: u32,
    pub dirty_chunks: HashSet<(i64, i64, i64)>,
    pub need_to_load: VecDeque<(i64, i64, i64)>,
}

impl World {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut world = Self {
            chunks: BTreeMap::new(),
            meshes: BTreeMap::new(),
            seed: rng.next_u32(),
            dirty_chunks: HashSet::new(),
            need_to_load: VecDeque::new(),
        };
        const LOAD_AREA: usize = 100;
        const LOAD_DEPTH: usize = 3;
        for x in 0..LOAD_AREA {
            for y in 0..LOAD_AREA {
                for z in 0..LOAD_DEPTH {
                    world.need_to_load.push_back((x as i64, y as i64, z as i64));
                    world.need_to_load.push_back((-(x as i64), y as i64, z as i64));
                    world.need_to_load.push_back((-(x as i64), -(y as i64), z as i64));
                    world.need_to_load.push_back(((x as i64), -(y as i64), z as i64));
                }
            }
        }
        world
    }
    
    pub fn update(&mut self, has_time: Duration) {
        const LOAD_RATIO: f64 = 0.5;
        self.load_new(
            Duration::from_secs_f64(has_time.as_secs_f64() * LOAD_RATIO)
        );
        self.update_meshes();
    }
    
    pub fn load_new(&mut self, has_time: Duration) {
        let started = Instant::now();
        const BATCH_SIZE: usize = 10;
        const NEIGHBOR_OFFSETS: [(i64, i64, i64); 6] = [
            (1, 0, 0), (-1, 0, 0), 
            (0, 1, 0), (0, -1, 0), 
            (0, 0, 1), (0, 0, -1)
        ];
        
        while started.elapsed() < has_time && !self.need_to_load.is_empty() {
            let mut processed = 0;
            while let Some(k) = self.need_to_load.pop_front() {
                self.load_chunks(k.0, k.1, k.2);
                let mut neighbors_to_check = Vec::with_capacity(6);
                for offset in &NEIGHBOR_OFFSETS {
                    neighbors_to_check.push((k.0 + offset.0, k.1 + offset.1, k.2 + offset.2));
                }
                for neighbor in neighbors_to_check {
                    if let Some(chunk) = self.chunks.get_mut(&neighbor) {
                        if !chunk.is_dirty {
                            chunk.is_dirty = true;
                            self.dirty_chunks.insert(neighbor);
                        }
                    }
                }
                if let Some(chunk) = self.chunks.get_mut(&k) {
                    chunk.is_dirty = true;
                }
                self.dirty_chunks.insert(k);
                processed += 1;
                if processed >= BATCH_SIZE {
                    break;
                }
            }
            if processed == 0 {
                break;
            }
        }
    }
    
    pub fn _update_dirty_set(&mut self) {
        for (key, chunk) in &self.chunks {
            if chunk.is_dirty {
                self.dirty_chunks.insert(*key);
            }
        }
    }
    
    fn update_meshes(&mut self) {
        let durty_chunks = std::mem::take(&mut self.dirty_chunks);
        for key in durty_chunks {
            let chunk = self.chunks.get(&key).unwrap();
            let (vertices, indices) = chunk.generate_mesh(&self);
            
            if self.meshes.contains_key(&key) {
                let mesh = self.meshes.get_mut(&key).unwrap();
                mesh.update(vertices, indices);
            } else {
                let mesh = Mesh::new(vertices, indices);
                self.meshes.insert(key, mesh);
            }
            
            self.chunks.get_mut(&key).unwrap().is_dirty = false;
        };
    }
    
    pub fn load_chunks(&mut self, x: i64, y: i64, z: i64) {
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
            chunk.get_from_world_pos(world_pos)
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
        // if self.need_to_load.contains(&(neighbor.x, neighbor.y, neighbor.z)) {
        //     return false;
        // }
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
