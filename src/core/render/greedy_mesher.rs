use cgmath::Vector3;
use crate::core::{block::Block, chunk::{Chunk, CHUNK_SIZE}, render::{face_gen::generate_face, vertex::Vertex}, world::world::World};

pub struct GreedyMesher;

impl GreedyMesher {
    pub fn build_mesh(chunk: &Chunk, world: &World) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u32;
        
        let normals = [
            Vector3::new(1.0, 0.0, 0.0), 
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0), 
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0), 
            Vector3::new(0.0, 0.0, -1.0),
        ];
        
        for &normal in &normals {
            Self::greedy_mesh_direction(chunk, world, normal, &mut vertices, &mut indices, &mut index_offset);
        }

        (vertices, indices)
    }

    fn greedy_mesh_direction(
        chunk: &Chunk,
        world: &World,
        normal: Vector3<f32>,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        index_offset: &mut u32,
    ) {
        let mut visited = [[[false; 16]; 16]; 16];

        // Determine which dimension is the "depth" (the one we're looking at)
        let (u_axis, v_axis, depth_axis) = if normal.x.abs() > 0.5 {
            // X-facing: depth = x, plane = yz
            (Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0))
        } else if normal.y.abs() > 0.5 {
            // Y-facing: depth = y, plane = xz
            (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0))
        } else {
            // Z-facing: depth = z, plane = xy
            (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0))
        };

        for depth in 0..16 {
            for u in 0..16 {
                for v in 0..16 {
                    let pos = Self::get_position(u_axis, v_axis, depth_axis, depth, u, v);
                    let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);
                    
                    if x >= 16 || y >= 16 || z >= 16 || visited[x][y][z] {
                        continue;
                    }

                    let block = chunk.get(x, y, z).unwrap_or(Block::air());
                    
                    if block.is_transpose() {
                        continue;
                    }

                    let world_pos = Vector3::new(
                        pos.x + chunk._pos.x as f32 * 16.0,
                        pos.y + chunk._pos.y as f32 * 16.0,
                        pos.z + chunk._pos.z as f32 * 16.0,
                    );
                    if !world.is_face_exposed(world_pos, normal) {
                        continue;
                    }

                    // Find the largest quad
                    let (quad_width, quad_height) = Self::find_quad(
                        chunk, world, normal, depth, u, v, &block, u_axis, v_axis, depth_axis, &mut visited
                    );

                    if quad_width > 0 && quad_height > 0 {
                        Self::create_greedy_quad(
                            chunk,
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
                            vertices,
                            indices,
                            index_offset,
                        );

                        // Mark the entire quad as visited
                        for du in 0..quad_width {
                            for dv in 0..quad_height {
                                let quad_pos = Self::get_position(u_axis, v_axis, depth_axis, depth, u + du, v + dv);
                                let (qx, qy, qz) = (quad_pos.x as usize, quad_pos.y as usize, quad_pos.z as usize);
                                if qx < 16 && qy < 16 && qz < 16 {
                                    visited[qx][qy][qz] = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn find_quad(
        chunk: &Chunk,
        world: &World,
        normal: Vector3<f32>,
        depth: usize,
        start_u: usize,
        start_v: usize,
        target_block: &Block,
        u_axis: Vector3<f32>,
        v_axis: Vector3<f32>,
        depth_axis: Vector3<f32>,
        visited: &mut [[[bool; 16]; 16]; 16],
    ) -> (usize, usize) {
        let max_width = 16 - start_u;
        let max_height = 16 - start_v;
        let mut quad_width = 1;
        let mut quad_height = 1;

        // Expand horizontally
        'width_loop: for w in 1..max_width {
            for h in 0..quad_height {
                let pos = Self::get_position(u_axis, v_axis, depth_axis, depth, start_u + w, start_v + h);
                let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);
                
                if x >= 16 || y >= 16 || z >= 16 || visited[x][y][z] {
                    break 'width_loop;
                }

                let block = chunk.get(x, y, z).unwrap_or(Block::air());
                
                if block != *target_block || block.is_transpose() {
                    break 'width_loop;
                }

                let world_pos = Vector3::new(
                    pos.x + chunk._pos.x as f32 * 16.0,
                    pos.y + chunk._pos.y as f32 * 16.0,
                    pos.z + chunk._pos.z as f32 * 16.0,
                );
                if !world.is_face_exposed(world_pos, normal) {
                    break 'width_loop;
                }
            }
            quad_width += 1;
        }

        // Expand vertically
        'height_loop: for h in 1..max_height {
            for w in 0..quad_width {
                let pos = Self::get_position(u_axis, v_axis, depth_axis, depth, start_u + w, start_v + h);
                let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);
                
                if x >= 16 || y >= 16 || z >= 16 || visited[x][y][z] {
                    break 'height_loop;
                }

                let block = chunk.get(x, y, z).unwrap_or(Block::air());
                
                if block != *target_block || block.is_transpose() {
                    break 'height_loop;
                }

                let world_pos = Vector3::new(
                    pos.x + chunk._pos.x as f32 * 16.0,
                    pos.y + chunk._pos.y as f32 * 16.0,
                    pos.z + chunk._pos.z as f32 * 16.0,
                );
                if !world.is_face_exposed(world_pos, normal) {
                    break 'height_loop;
                }
            }
            quad_height += 1;
        }

        (quad_width, quad_height)
    }

    fn create_greedy_quad(
        chunk: &Chunk,
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
        let color = [
            block.color.0 as f32 / 255.0,
            block.color.1 as f32 / 255.0,
            block.color.2 as f32 / 255.0,
        ];

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
            color,
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

    fn get_position(u_axis: Vector3<f32>, v_axis: Vector3<f32>, depth_axis: Vector3<f32>, depth: usize, u: usize, v: usize) -> Vector3<f32> {
        Vector3::new(
            u_axis.x * u as f32 + v_axis.x * v as f32 + depth_axis.x * depth as f32,
            u_axis.y * u as f32 + v_axis.y * v as f32 + depth_axis.y * depth as f32,
            u_axis.z * u as f32 + v_axis.z * v as f32 + depth_axis.z * depth as f32,
        )
    }
}