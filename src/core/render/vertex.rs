use cgmath::{InnerSpace, Vector3, Zero};
use wgpu::VertexFormat;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],    // keep for now for fallback
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub const fn new(pos: [f32; 3], tex_coord: [f32; 2]) -> Self {
        Self { pos, tex_coord, color: [1.0, 0.0, 1.0]}
    }
    
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout { 
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
            ]
        }
    }
}

const CUBE_VERTICES: &[Vertex] = &[ // FIXME: tex_coord
    Vertex::new([-0.5, -0.5, -0.5], [1.0, 1.0]),
    Vertex::new([0.5, -0.5, -0.5], [1.0, 1.0]),
    Vertex::new([0.5, -0.5, 0.5], [1.0, 1.0]),
    Vertex::new([-0.5, -0.5, 0.5], [1.0, 1.0]),
    Vertex::new([-0.5, 0.5, -0.5], [1.0, 1.0]),
    Vertex::new([0.5, 0.5, -0.5], [1.0, 1.0]),
    Vertex::new([0.5, 0.5, 0.5], [1.0, 1.0]),
    Vertex::new([-0.5, 0.5, 0.5], [1.0, 1.0]),
];

const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // Bottom
    4, 6, 5, 4, 7, 6, // Top
    0, 5, 1, 0, 4, 5, // Front
    3, 2, 6, 3, 6, 7, // Back
    0, 3, 7, 0, 7, 4, // Left
    1, 5, 6, 1, 6, 2, // Right
];

pub fn generate_voxel_mesh(voxel_pos: Vector3<f32>, tex_coord: [f32; 2]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(CUBE_VERTICES.len());
    let mut indices = Vec::with_capacity(CUBE_INDICES.len());
    // let indices = CUBE_INDICES.clone();
    
    for &v in CUBE_VERTICES {   // Offset cube vertices by voxel position
        let pos = [
            v.pos[0] + voxel_pos.x,
            v.pos[1] + voxel_pos.y,
            v.pos[2] + voxel_pos.z,
        ];
        vertices.push(Vertex::new(pos, tex_coord));
    }
    // Use same indices (they're relative to the 8 vertices)
    indices.extend_from_slice(CUBE_INDICES);
    (vertices, indices)
}

pub fn generate_voxel_face(pos: Vector3<f32>, color: [f32; 3], normal: Vector3<f32>, texture_id: u16) -> (Vec<Vertex>, Vec<u16>) {
    let half = 0.5;
    let texture_atlas_size = 32.0; // atlas as 32x32 textures
    let tex_size = 1.0 / texture_atlas_size;
    
    let tex_x = (texture_id % texture_atlas_size as u16) as f32 * tex_size;
    let tex_y = (texture_id / texture_atlas_size as u16) as f32 * tex_size;
    
    let uvs = [
        [tex_x, tex_y],
        [tex_x + tex_size, tex_y],
        [tex_x + tex_size, tex_y + tex_size],
        [tex_x, tex_y + tex_size],
    ];
    
    let faces = match normal {
        Vector3 { x: -1.0, y: 0.0, z: 0.0 } => ( // Left
            [
                Vector3::new(-half, -half, -half),
                Vector3::new(-half, -half, half),
                Vector3::new(-half, half, half),
                Vector3::new(-half, half, -half),
            ],
            [0, 1, 2, 2, 3, 0]
        ),
        Vector3 { x: 1.0, y: 0.0, z: 0.0 } => ( // Right
            [
                Vector3::new(half, -half, half),
                Vector3::new(half, -half, -half),
                Vector3::new(half, half, -half),
                Vector3::new(half, half, half),
            ],
            [0, 1, 2, 2, 3, 0]
        ),
        Vector3 { x: 0.0, y: -1.0, z: 0.0 } => ( // Bottom
            [
                Vector3::new(-half, -half, -half),
                Vector3::new(half, -half, -half),
                Vector3::new(half, -half, half),
                Vector3::new(-half, -half, half),
            ],
            [0, 1, 2, 2, 3, 0]
        ),
        Vector3 { x: 0.0, y: 1.0, z: 0.0 } => ( // Top
            [
                Vector3::new(-half, half, half),
                Vector3::new(half, half, half),
                Vector3::new(half, half, -half),
                Vector3::new(-half, half, -half),
            ],
            [0, 1, 2, 2, 3, 0]
        ),
        Vector3 { x: 0.0, y: 0.0, z: -1.0 } => ( // Back
            [
                Vector3::new(half, -half, -half),
                Vector3::new(-half, -half, -half),
                Vector3::new(-half, half, -half),
                Vector3::new(half, half, -half),
            ],
            [0, 1, 2, 2, 3, 0]
        ),
        Vector3 { x: 0.0, y: 0.0, z: 1.0 } => ( // Front
            [
                Vector3::new(-half, -half, half),
                Vector3::new(half, -half, half),
                Vector3::new(half, half, half),
                Vector3::new(-half, half, half),
            ],
            [0, 1, 2, 2, 3, 0]
        ),
        _ => ( [Vector3::zero(); 4], [0, 0, 0, 0, 0, 0] )
    };
    
    let (positions, indices) = faces;
    let mut vertices = Vec::new();
    
    for (i, &position) in positions.iter().enumerate() {
        vertices.push(Vertex {
            pos: [pos.x + position.x, pos.y + position.y, pos.z + position.z],
            tex_coord: uvs[i],
            color,
        });
    }
    
    (vertices, indices.to_vec())
}