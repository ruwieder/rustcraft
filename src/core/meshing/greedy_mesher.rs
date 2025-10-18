use crate::{
    core::{
        block::Block,
        chunk::{CHUNK_SIZE, Chunk},
        meshing::{Vertex, generate_face},
    },
    world::World,
};
use cgmath::Vector3;

pub struct GreedyMesher;

impl GreedyMesher {
    pub fn build_mesh(chunk: &Chunk, world: &World) -> (Vec<Vertex>, Vec<u32>) {
        if Self::is_only_air_fast(chunk) && Self::is_only_air(chunk) {
            return (Vec::new(), Vec::new());
        }
        let normals = [
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, -1.0),
        ];
        // Precompute exposed faces for the entire chunk to avoid repeated world lookups
        let exposed_cache = Self::build_exposed_cache(chunk, world);

        let direction_results: Vec<(Vec<Vertex>, Vec<u32>)> = normals
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
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let idx = Chunk::index(x, y, z);
                    if !chunk.blocks[idx].is_transpose() {
                        return false;
                    }
                }
            }
        }
        true
    }

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

    fn build_exposed_cache(
        chunk: &Chunk,
        world: &World,
    ) -> [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] {
        let mut cache = [[[0u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let chunk_world_base = Vector3::new(
            chunk._pos.x * CHUNK_SIZE as i64,
            chunk._pos.y * CHUNK_SIZE as i64,
            chunk._pos.z * CHUNK_SIZE as i64,
        );

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let mut exposed_mask = 0u8;

                    let directions = [
                        (1, 0, 0),
                        (-1, 0, 0),
                        (0, 1, 0),
                        (0, -1, 0),
                        (0, 0, 1),
                        (0, 0, -1),
                    ];

                    for (dir, &(dx, dy, dz)) in directions.iter().enumerate() {
                        let nx = x as i64 + dx;
                        let ny = y as i64 + dy;
                        let nz = z as i64 + dz;

                        let exposed = if nx >= 0
                            && nx < CHUNK_SIZE as i64
                            && ny >= 0
                            && ny < CHUNK_SIZE as i64
                            && nz >= 0
                            && nz < CHUNK_SIZE as i64
                        {
                            let block = chunk.get(nx as usize, ny as usize, nz as usize);
                            block.is_transpose()
                        } else {
                            let world_pos = Vector3::new(
                                chunk_world_base.x + nx,
                                chunk_world_base.y + ny,
                                chunk_world_base.z + nz,
                            );
                            Self::is_face_exposed_new(world, world_pos)
                        };

                        if exposed {
                            exposed_mask |= 1 << dir;
                        }
                    }

                    cache[x][y][z] = exposed_mask;
                }
            }
        }

        cache
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
        direction: usize,
        exposed_cache: &[[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(1024);
        let mut indices = Vec::with_capacity(1024);
        let mut index_offset = 0u32;

        let mut visited = BitSet::new(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);

        let (u_axis, v_axis, depth_axis) = if normal.x.abs() > 0.5 {
            (
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(1.0, 0.0, 0.0),
            )
        } else if normal.y.abs() > 0.5 {
            (
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 1.0, 0.0),
            )
        } else {
            (
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            )
        };

        for depth in 0..CHUNK_SIZE {
            for u in 0..CHUNK_SIZE {
                for v in 0..CHUNK_SIZE {
                    let pos = Self::get_position(u_axis, v_axis, depth_axis, depth, u, v);
                    let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);
                    if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
                        continue;
                    }
                    let index = x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z;
                    if visited.get(index) {
                        continue;
                    }
                    let block = chunk.get(x, y, z);
                    if block.is_transpose() {
                        continue;
                    }
                    if (exposed_cache[x][y][z] & (1 << direction)) == 0 {
                        continue;
                    }
                    let (quad_width, quad_height) = Self::find_quad(
                        chunk,
                        depth,
                        u,
                        v,
                        block,
                        direction,
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
                                if qx < CHUNK_SIZE && qy < CHUNK_SIZE && qz < CHUNK_SIZE {
                                    let quad_index =
                                        qx * CHUNK_SIZE * CHUNK_SIZE + qy * CHUNK_SIZE + qz;
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
        direction: usize,
        u_axis: Vector3<f32>,
        v_axis: Vector3<f32>,
        depth_axis: Vector3<f32>,
        visited: &BitSet,
        exposed_cache: &[[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    ) -> (usize, usize) {
        let max_width = CHUNK_SIZE - start_u;
        let max_height = CHUNK_SIZE - start_v;
        let mut quad_width = 1;
        let mut quad_height = 1;

        // Expand horizontally with early break
        for w in 1..max_width {
            let mut valid = true;
            for h in 0..quad_height {
                let pos =
                    Self::get_position(u_axis, v_axis, depth_axis, depth, start_u + w, start_v + h);
                let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);

                if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
                    valid = false;
                    break;
                }

                let index = x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z;
                if visited.get(index) {
                    valid = false;
                    break;
                }

                let block = chunk.get(x, y, z);
                if block != target_block || block.is_transpose() {
                    valid = false;
                    break;
                }

                if (exposed_cache[x][y][z] & (1 << direction)) == 0 {
                    valid = false;
                    break;
                }
            }

            if !valid {
                break;
            }
            quad_width += 1;
        }

        // Expand vertically with early break
        for h in 1..max_height {
            let mut valid = true;
            for w in 0..quad_width {
                let pos =
                    Self::get_position(u_axis, v_axis, depth_axis, depth, start_u + w, start_v + h);
                let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);

                if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
                    valid = false;
                    break;
                }

                let index = x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z;
                if visited.get(index) {
                    valid = false;
                    break;
                }

                let block = chunk.get(x, y, z);
                if block != target_block || block.is_transpose() {
                    valid = false;
                    break;
                }

                if (exposed_cache[x][y][z] & (1 << direction)) == 0 {
                    valid = false;
                    break;
                }
            }

            if !valid {
                break;
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
        let base_pos = Self::get_position(u_axis, v_axis, depth_axis, depth, u, v);

        // Adjust position to be the center of the quad
        let center_offset_u = (quad_width as f32 - 1.0) * 0.5;
        let center_offset_v = (quad_height as f32 - 1.0) * 0.5;

        let center_pos = Vector3::new(
            base_pos.x + u_axis.x * center_offset_u + v_axis.x * center_offset_v,
            base_pos.y + u_axis.y * center_offset_u + v_axis.y * center_offset_v,
            base_pos.z + u_axis.z * center_offset_u + v_axis.z * center_offset_v,
        );

        let (quad_vertices, quad_indices) = generate_face(
            center_pos,
            normal,
            block.id,
            quad_width as f32,
            quad_height as f32,
        );
        for vertex in quad_vertices {
            vertices.push(vertex);
        }
        for mut index in quad_indices {
            index += *index_offset;
            indices.push(index);
        }
        *index_offset += 4;
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

// Simple bit set for efficient visited tracking
struct BitSet {
    data: Vec<u64>,
    size: usize,
}

impl BitSet {
    fn new(size: usize) -> Self {
        let array_size = size.div_ceil(64);
        Self {
            data: vec![0; array_size],
            size,
        }
    }

    fn get(&self, index: usize) -> bool {
        if index >= self.size {
            return false;
        }
        let word = index / 64;
        let bit = index % 64;
        (self.data[word] & (1 << bit)) != 0
    }

    fn set(&mut self, index: usize) {
        if index >= self.size {
            return;
        }
        let word = index / 64;
        let bit = index % 64;
        self.data[word] |= 1 << bit;
    }
}
