use std::time::{Duration, Instant};
use rayon::prelude::*;
use cgmath::{InnerSpace, Vector3};

use crate::core::{chunk::{Chunk, CHUNK_SIZE}, render::camera::Camera, world::world::World};

const LOAD_DISTANCE: i32 = 20;
const LOAD_DISTANCE_Z: i32 = 5;
const UNLOAD_DISTANCE: i32 = LOAD_DISTANCE * 2;

impl World {
    pub fn loader_update(&mut self, has_time: Duration, camera: &Camera) {
        let pos = camera.pos;
        let (forward, _, _) = camera.fru();
        let chunk_idx = Vector3::new(
            (pos.x as i64).div_euclid(CHUNK_SIZE as i64),
            (pos.y as i64).div_euclid(CHUNK_SIZE as i64),
            (pos.z as i64).div_euclid(CHUNK_SIZE as i64),
        ); 
        
        self.collect_around(chunk_idx, forward);
        self.unload_far(chunk_idx);
        
        const LOAD_RATIO: f64 = 0.7;
        let load_time = Duration::from_secs_f64(has_time.as_secs_f64() * LOAD_RATIO);
        
        self.load_new(load_time);
        // self.update_chunk_states(renderer);
    }
    
    
    pub fn load_new(&mut self, has_time: Duration) {
        let started = Instant::now();
        const BATCH_SIZE: usize = 30;
        
        while started.elapsed() < has_time && !self.need_to_load.is_empty() {
            let batch: Vec<_> = self.need_to_load.drain(..BATCH_SIZE.min(self.need_to_load.len())).collect();
            
            let new_chunks: Vec<((i64, i64, i64), Chunk)> = batch
                .par_iter()
                .map(|&(x, y, z)| {
                    let world_pos = Vector3 { x, y, z };
                    let chunk = Chunk::terrain_gen(world_pos, self.seed);
                    ((x, y, z), chunk)
                })
                .collect();
            
            for (key, chunk) in new_chunks {
                self.chunks.insert(key, chunk);
                self.mark_neighbors_dirty(key);
            }
        }
    }
    
    pub fn collect_around(&mut self, player_chunk: Vector3<i64>, camera_dir: Vector3<f32>) {
        use std::cmp::Reverse;
        let mut chunks_to_load = Vec::new();
        for x in -LOAD_DISTANCE..=LOAD_DISTANCE {
            for y in -LOAD_DISTANCE..=LOAD_DISTANCE {
                for z in -LOAD_DISTANCE_Z..=LOAD_DISTANCE_Z {
                    let chunk_pos = player_chunk + Vector3::new(x as i64, y as i64, z as i64);
                    let key = (chunk_pos.x, chunk_pos.y, chunk_pos.z);
                    
                    if !self.chunks.contains_key(&key) && !self.need_to_load.contains(&key) {
                        let priority = self.calculate_loading_priority(chunk_pos, player_chunk, camera_dir);
                        chunks_to_load.push((priority, key));
                    }
                }
            }
        }
        chunks_to_load.sort_by_key(|(priority, _)| Reverse(*priority));
        // self.need_to_load.extend(chunks_to_load.into_iter().map(|(_, key)| key));
        self.need_to_load = chunks_to_load.into_iter().map(|(_, key)| key).collect();
    }
    
    
    fn calculate_loading_priority(&self, chunk_pos: Vector3<i64>, player_chunk: Vector3<i64>, camera_dir: Vector3<f32>) -> u32 {
        let delta = chunk_pos - player_chunk;
        let distance_sq = delta.x * delta.x + delta.y * delta.y + delta.z * delta.z;
        // Base priority: closer chunks have higher priority
        let mut priority: u32 = (2000 - distance_sq).max(0) as u32;
        // Boost priority for chunks in camera direction
        let chunk_dir = Vector3::new(delta.x as f32, delta.y as f32, delta.z as f32).normalize();
        let dot = camera_dir.dot(chunk_dir);
        priority += (1000.0 * dot.max(0.0)) as u32;
        // Boost for chunks directly around player (neighbors)
        if delta.x.abs() <= 5 && delta.z.abs() <= 5 && delta.y.abs() <= 5 {
            priority += 1000;
        }
        priority
    }
    
    pub fn unload_far(&mut self, player_chunk: Vector3<i64>){
        let unload_dist_sq = (UNLOAD_DISTANCE * UNLOAD_DISTANCE) as i64;
        let unload_dist_z = LOAD_DISTANCE_Z as i64 + 10;
        
        let to_remove: Vec<_> = self.chunks
            .keys()
            .filter(|&&(x, y, z)| {
                let dx = x - player_chunk.x;
                let dy = y - player_chunk.y;
                let dz = z - player_chunk.z;
                (dx * dx + dy * dy) > unload_dist_sq || dz.abs() > unload_dist_z
            })
            .cloned()
            .collect();
            
        for key in &to_remove {
            self.drop_chunk(Vector3::new(key.0, key.1, key.2));
        };
    }
    
    pub fn mark_neighbors_dirty(&mut self, center: (i64, i64, i64)) {
        const NEIGHBOR_OFFSETS: [(i64, i64, i64); 6] = [
            (1, 0, 0), (-1, 0, 0), 
            (0, 1, 0), (0, -1, 0), 
            (0, 0, 1), (0, 0, -1)
        ];
        
        for offset in &NEIGHBOR_OFFSETS {
            let neighbor = (center.0 + offset.0, center.1 + offset.1, center.2 + offset.2);
            if let Some(chunk) = self.chunks.get_mut(&neighbor) {
                chunk.is_dirty = true;
                self.dirty_chunks.insert(neighbor);
            }
        }
    }
}