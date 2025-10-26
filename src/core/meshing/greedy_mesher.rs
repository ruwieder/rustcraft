use crate::{
    core::{
        block::Block,
        chunk::{CHUNK_SIZE, Chunk},
        meshing::{Vertex, generate_face},
    },
    world::World,
};
use cgmath::Vector3;

const DIRECTIONS: [Vector3<f32>; 6] = [
    Vector3 { x:  1.0, y:  0.0, z:  0.0 },
    Vector3 { x: -1.0, y:  0.0, z:  0.0 },
    Vector3 { x:  0.0, y:  1.0, z:  0.0 },
    Vector3 { x:  0.0, y: -1.0, z:  0.0 },
    Vector3 { x:  0.0, y:  0.0, z:  1.0 },
    Vector3 { x:  0.0, y:  0.0, z: -1.0 },
];

const fn get_block(chunk: &Chunk, x: usize, y: usize, z: usize) -> Block {
    chunk.get(x * SCALE, y * SCALE, z * SCALE)
}

const SCALE: usize = 4;
const MACRO_COUNT: usize = CHUNK_SIZE / SCALE;

#[inline(always)]
const fn face_index(x: usize, y: usize, z: usize, dir: usize) -> usize {
    dir + 6 * (x * MACRO_COUNT * MACRO_COUNT + y * MACRO_COUNT + z)
}

pub struct GreedyMesher;

impl GreedyMesher {
    pub fn build_mesh(chunk: &Chunk, world: &World) -> (Vec<Vertex>, Vec<u32>) {
        if Self::is_only_air_fast(chunk) && Self::is_only_air(chunk) {
            return (Vec::new(), Vec::new());
        }

        // Precompute exposed faces for the entire chunk to avoid repeated world lookups
        let exposed_cache = Self::build_exposed_cache(chunk, world);

        let direction_results: Vec<(Vec<Vertex>, Vec<u32>)> = DIRECTIONS
            .iter()
            .enumerate()
            .map(|(dir, &normal)| Self::greedy_mesh_direction(chunk, normal, dir, &exposed_cache))
            .collect();

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u32;

        for (dir_vertices, dir_indices) in direction_results {
            vertices.extend(dir_vertices);
            indices.extend(dir_indices.into_iter().map(|i| i + index_offset));
            index_offset = vertices.len() as u32;
        }

        (vertices, indices)
    }

    fn is_only_air(chunk: &Chunk) -> bool {
        for x in 0..MACRO_COUNT {
            for y in 0..MACRO_COUNT {
                for z in 0..MACRO_COUNT {
                    // if !chunk.get(x, y, z).is_transpose() {
                    if !get_block(chunk, x, y, z).is_transpose() {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// check corner blocks for early exit
    #[inline]
    fn is_only_air_fast(chunk: &Chunk) -> bool {
        const IDX_1: usize = Chunk::index(0, 0, 0);
        const IDX_2: usize = Chunk::index(0, 0, CHUNK_SIZE - 1);
        const IDX_3: usize = Chunk::index(0, CHUNK_SIZE - 1, 0);
        const IDX_4: usize = Chunk::index(0, CHUNK_SIZE - 1, CHUNK_SIZE - 1);
        const IDX_5: usize = Chunk::index(CHUNK_SIZE - 1, 0, 0);
        const IDX_6: usize = Chunk::index(CHUNK_SIZE - 1, 0, CHUNK_SIZE - 1);
        const IDX_7: usize = Chunk::index(CHUNK_SIZE - 1, CHUNK_SIZE - 1, 0);
        const IDX_8: usize = Chunk::index(CHUNK_SIZE - 1, CHUNK_SIZE - 1, CHUNK_SIZE - 1);
        chunk.blocks[IDX_1].is_transpose()
            && chunk.blocks[IDX_2].is_transpose()
            && chunk.blocks[IDX_3].is_transpose()
            && chunk.blocks[IDX_4].is_transpose()
            && chunk.blocks[IDX_5].is_transpose()
            && chunk.blocks[IDX_6].is_transpose()
            && chunk.blocks[IDX_7].is_transpose()
            && chunk.blocks[IDX_8].is_transpose()
    }

    fn build_exposed_cache(chunk: &Chunk, world: &World) -> BitSet {
        let mut cache = BitSet::new(6 * MACRO_COUNT * MACRO_COUNT * MACRO_COUNT);
        let chunk_world_base = Vector3::new(
            chunk.pos.x * CHUNK_SIZE as i64,
            chunk.pos.y * CHUNK_SIZE as i64,
            chunk.pos.z * CHUNK_SIZE as i64,
        );
    
        for x in 0..MACRO_COUNT {
            for y in 0..MACRO_COUNT {
                for z in 0..MACRO_COUNT {
                    for (dir, &normal) in DIRECTIONS.iter().enumerate() {
                        let normal_axis = if normal.x != 0.0 { 0 }
                            else if normal.y != 0.0 { 1 }
                            else { 2 };
                        let is_positive = normal[normal_axis] > 0.0;
                        let other_axes = [0, 1, 2].iter().filter(|&&a| a != normal_axis).copied().collect::<Vec<_>>();
                        let axis0 = other_axes[0];
                        let axis1 = other_axes[1];
                        let normal_start = if is_positive {
                            [x, y, z][normal_axis] * SCALE + SCALE - 1
                        } else {
                            [x, y, z][normal_axis] * SCALE
                        };
                        
                        let mut exposed = false;
                        for a0 in 0..SCALE {
                            for a1 in 0..SCALE {
                                let mut pos = [x * SCALE, y * SCALE, z * SCALE];
                                pos[normal_axis] = normal_start;
                                pos[axis0] += a0;
                                pos[axis1] += a1;
                                
                                if Self::is_face_exposed(world, chunk, chunk_world_base, pos[0] as i64, pos[1] as i64, pos[2] as i64, normal) {
                                    exposed = true;
                                    break;
                                }
                            }
                            if exposed { break; }
                        }
                        
                        cache.set_if(face_index(x, y, z, dir), exposed);
                    }
                }
            }
        }
    
        cache
    }
    
    fn is_face_exposed(
        world: &World,
        chunk: &Chunk,
        chunk_world_base: Vector3<i64>,
        px: i64,
        py: i64,
        pz: i64,
        normal: Vector3<f32>,
    ) -> bool {
        let nx = px + normal.x as i64;
        let ny = py + normal.y as i64;
        let nz = pz + normal.z as i64;
    
        if nx >= 0 && nx < CHUNK_SIZE as i64 &&
           ny >= 0 && ny < CHUNK_SIZE as i64 &&
           nz >= 0 && nz < CHUNK_SIZE as i64 {
            let block = chunk.get(nx as usize, ny as usize, nz as usize);
            block.is_transpose()
        } else {
            let world_pos = Vector3::new(
                chunk_world_base.x + nx,
                chunk_world_base.y + ny,
                chunk_world_base.z + nz,
            );
            Self::is_face_exposed_new(world, world_pos)
        }
    }
    
    fn is_face_exposed_new(world: &World, pos: Vector3<i64>) -> bool {
        if let Some(chunk) = world.get_chunk(&pos) {
            let block = chunk.get_from_world_pos(pos);
            block.is_transpose()
        } else {
            false
        }
    }

    fn greedy_mesh_direction(
        chunk: &Chunk,
        normal: Vector3<f32>,
        dir: usize,
        exposed_cache: &BitSet,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(MACRO_COUNT * MACRO_COUNT);
        let mut indices = Vec::with_capacity(MACRO_COUNT * MACRO_COUNT);
        let mut index_offset = 0u32;

        let mut visited = BitSet::new(MACRO_COUNT * MACRO_COUNT * MACRO_COUNT);
        let depth_axis = normal.map(|v| v.abs());
        let (u_axis, v_axis) = if normal.x.abs() > 0.5 {
            (Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0))
        } else if normal.y.abs() > 0.5 {
            (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0))
        } else {
            (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0))
        };

        for depth in 0..MACRO_COUNT {
            for u in 0..MACRO_COUNT {
                for v in 0..MACRO_COUNT {
                    let pos = Self::get_position(u_axis, v_axis, depth_axis, depth, u, v);
                    let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);
                    if x >= MACRO_COUNT || y >= MACRO_COUNT || z >= MACRO_COUNT {
                        continue;
                    }
                    let index = x * MACRO_COUNT * MACRO_COUNT + y * MACRO_COUNT + z;
                    if visited.get(index) {
                        continue;
                    }
                    // let block = chunk.get(x, y, z);
                    let block = get_block(chunk, x, y, z);
                    if block.is_transpose() {
                        continue;
                    }
                    if !exposed_cache.get(face_index(x, y, z, dir)) {
                        continue;
                    }
                    let (quad_width, quad_height) = Self::find_quad(
                        chunk,
                        depth,
                        u,
                        v,
                        block,
                        dir,
                        u_axis,
                        v_axis,
                        depth_axis,
                        &visited,
                        exposed_cache,
                    );

                    if quad_width > 0 && quad_height > 0 {
                        Self::create_greedy_quad(
                            normal,
                            depth,
                            u,
                            v,
                            quad_width,
                            quad_height,
                            &block,
                            u_axis,
                            v_axis,
                            depth_axis,
                            &mut vertices,
                            &mut indices,
                            &mut index_offset,
                        );

                        // Mark quad as visited
                        for du in 0..quad_width {
                            for dv in 0..quad_height {
                                let quad_pos = Self::get_position(
                                    u_axis,
                                    v_axis,
                                    depth_axis,
                                    depth,
                                    u + du,
                                    v + dv,
                                );
                                let (qx, qy, qz) = (
                                    quad_pos.x as usize,
                                    quad_pos.y as usize,
                                    quad_pos.z as usize,
                                );
                                if qx < MACRO_COUNT && qy < MACRO_COUNT && qz < MACRO_COUNT {
                                    let quad_index =
                                        qx * MACRO_COUNT * MACRO_COUNT + qy * MACRO_COUNT + qz;
                                    visited.set(quad_index);
                                }
                            }
                        }
                    }
                }
            }
        }

        (vertices, indices)
    }

    fn find_quad(
        chunk: &Chunk,
        depth: usize,
        start_u: usize,
        start_v: usize,
        target_block: Block,
        dir: usize,
        u_axis: Vector3<f32>,
        v_axis: Vector3<f32>,
        depth_axis: Vector3<f32>,
        visited: &BitSet,
        exposed_cache: &BitSet,
    ) -> (usize, usize) {
        let max_width = MACRO_COUNT - start_u;
        let max_height = MACRO_COUNT - start_v;
        let mut quad_width = 1;
        let mut quad_height = 1;

        // Expand horizontally
        'outer: for w in 1..max_width {
            for h in 0..quad_height {
                let pos =
                    Self::get_position(u_axis, v_axis, depth_axis, depth, start_u + w, start_v + h);
                let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);

                if x >= MACRO_COUNT || y >= MACRO_COUNT || z >= MACRO_COUNT {
                    panic!(); // FIXME
                }

                let index = x * MACRO_COUNT * MACRO_COUNT + y * MACRO_COUNT + z;
                if visited.get(index) {
                    break 'outer;
                }

                // let block = chunk.get(x, y, z);
                let block = get_block(chunk, x, y, z);
                if block != target_block || block.is_transpose() {
                    break 'outer;
                }

                if !exposed_cache.get(face_index(x, y, z, dir)) {
                    break 'outer;
                }
            }
            quad_width += 1;
        }

        // Expand vertically
        'outer: for h in 1..max_height {
            for w in 0..quad_width {
                let pos =
                    Self::get_position(u_axis, v_axis, depth_axis, depth, start_u + w, start_v + h);
                let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);

                if x >= MACRO_COUNT || y >= MACRO_COUNT || z >= MACRO_COUNT {
                    panic!(); // FIXME
                }

                let index = x * MACRO_COUNT * MACRO_COUNT + y * MACRO_COUNT + z;
                if visited.get(index) {
                    break 'outer;
                }

                // let block = chunk.get(x, y, z);
                let block = get_block(chunk, x, y, z);
                if block != target_block || block.is_transpose() {
                    break 'outer;
                }

                if !exposed_cache.get(face_index(x, y, z, dir)) {
                    break 'outer;
                }
            }
            quad_height += 1;
        }

        (quad_width, quad_height)
    }

    fn create_greedy_quad(
        normal: Vector3<f32>,
        depth: usize,
        u: usize,
        v: usize,
        quad_width: usize,
        quad_height: usize,
        block: &Block,
        u_axis: Vector3<f32>,
        v_axis: Vector3<f32>,
        depth_axis: Vector3<f32>,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        index_offset: &mut u32,
    ) {
        let start_u = u * SCALE;
        let start_v = v * SCALE;
        let start_depth = depth * SCALE;
        
        let quad_width_f = (quad_width * SCALE) as f32;
        let quad_height_f = (quad_height * SCALE) as f32;
        
        let base_pos = Self::get_position(
            u_axis, v_axis, depth_axis, 
            start_depth, start_u, start_v
        );
    
        let mut center_pos = base_pos;
        
        center_pos += u_axis * (quad_width_f * 0.5);
        center_pos += v_axis * (quad_height_f * 0.5);
        
        if normal.x > 0.5 {
            center_pos.x += SCALE as f32 - 0.5;
        } else if normal.x < -0.5 {
            center_pos.x += 0.5;
        } else if normal.y > 0.5 {
            center_pos.y += SCALE as f32 - 0.5;
        } else if normal.y < -0.5 {
            center_pos.y += 0.5;
        } else if normal.z > 0.5 {
            center_pos.z += SCALE as f32 - 0.5;
        } else if normal.z < -0.5 {
            center_pos.z += 0.5;
        }
    
        let (quad_vertices, quad_indices) = generate_face(
            center_pos,
            normal,
            block.id as u32,
            quad_width_f,
            quad_height_f,
        );
        
        let vertex_count = quad_vertices.len();
        vertices.extend(quad_vertices);
        
        for index in quad_indices {
            indices.push(index + *index_offset);
        }
        
        *index_offset += vertex_count as u32;
    }
    

    #[inline]
    const fn get_position(
        u_axis: Vector3<f32>,
        v_axis: Vector3<f32>,
        depth_axis: Vector3<f32>,
        depth: usize,
        u: usize,
        v: usize,
    ) -> Vector3<f32> {
        Vector3::new(
            u_axis.x * u as f32 + v_axis.x * v as f32 + depth_axis.x * depth as f32,
            u_axis.y * u as f32 + v_axis.y * v as f32 + depth_axis.y * depth as f32,
            u_axis.z * u as f32 + v_axis.z * v as f32 + depth_axis.z * depth as f32,
        )
    }
}

struct BitSet {
    data: Vec<u64>,
}

impl BitSet {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size.div_ceil(64)],
        }
    }

    #[inline(always)]
    fn get(&self, index: usize) -> bool {
        (self.data[index / 64] & (1 << (index % 64))) != 0
    }

    #[inline(always)]
    fn set(&mut self, index: usize) {
        self.data[index / 64] |= 1 << (index % 64);
    }

    #[inline(always)]
    fn set_if(&mut self, index: usize, flag: bool) {
        self.data[index / 64] |= (flag as u64) << (index % 64);
    }
}
