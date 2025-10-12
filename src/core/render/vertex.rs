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

const CUBE_VERTICES: &[Vertex] = &[ // Note: tex_coords is not used
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

pub fn generate_voxel_face(pos: Vector3<f32>, color: [f32; 3], normal: Vector3<f32>, texture_id: u16) -> (Vec<Vertex>, Vec<u32>) {
    const ATLAS_WIDTH: f32 = 1024.0;
    const ATLAS_HEIGHT: f32 = 512.0;
    const TEXTURE_SIZE: f32 = 16.0;
    const TEXTURES_PER_ROW: f32 = ATLAS_WIDTH / TEXTURE_SIZE;
    const TEXTURES_PER_COL: f32 = ATLAS_HEIGHT / TEXTURE_SIZE;
    const TEX_SIZE_U: f32 = TEXTURE_SIZE / ATLAS_WIDTH; 
    const TEX_SIZE_V: f32 = TEXTURE_SIZE / ATLAS_HEIGHT;
    debug_assert!((texture_id as f32) < TEXTURES_PER_COL * TEXTURES_PER_ROW);
    
    const HALF: f32 = 0.5; // FIXME: should be 0.5 for no gaps
    
    let tex_x = (texture_id as f32 % TEXTURES_PER_ROW) * TEX_SIZE_U;
    let tex_y = (texture_id as f32 / TEXTURES_PER_COL) * TEX_SIZE_V;

    let uvs = [
        [tex_x, tex_y],
        [tex_x + TEX_SIZE_U, tex_y],
        [tex_x + TEX_SIZE_U, tex_y + TEX_SIZE_V],
        [tex_x, tex_y + TEX_SIZE_V],
    ];
    
    let faces = match normal {
        Vector3 { x: 1.0, y: 0.0, z: 0.0 } => (
            [
                Vector3::new(HALF, -HALF, -HALF),  // bottom-right-back
                Vector3::new(HALF, HALF, -HALF),   // bottom-right-front
                Vector3::new(HALF, HALF, HALF),    // top-right-front
                Vector3::new(HALF, -HALF, HALF),   // top-right-back
            ],
            [0, 1, 2, 2, 3, 0]
        ),
        Vector3 { x: -1.0, y: 0.0, z: 0.0 } => (
            [
                Vector3::new(-HALF, HALF, -HALF),  // bottom-left-front
                Vector3::new(-HALF, -HALF, -HALF), // bottom-left-back
                Vector3::new(-HALF, -HALF, HALF),  // top-left-back
                Vector3::new(-HALF, HALF, HALF),   // top-left-front
            ],
            [0, 1, 2, 2, 3, 0]
        ),
        Vector3 { x: 0.0, y: 1.0, z: 0.0 } => (
            [
                Vector3::new(-HALF, HALF, -HALF),  // bottom-front-left
                Vector3::new(HALF, HALF, -HALF),   // bottom-front-right
                Vector3::new(HALF, HALF, HALF),    // top-front-right
                Vector3::new(-HALF, HALF, HALF),   // top-front-left
            ],
            [2, 1, 0, 3, 2, 0]
        ),
        Vector3 { x: 0.0, y: -1.0, z: 0.0 } => (
            [
                Vector3::new(HALF, -HALF, -HALF),  // bottom-back-right
                Vector3::new(-HALF, -HALF, -HALF), // bottom-back-left
                Vector3::new(-HALF, -HALF, HALF),  // top-back-left
                Vector3::new(HALF, -HALF, HALF),   // top-back-right
            ],
            [2, 1, 0, 0, 3, 2]
        ),
        Vector3 { x: 0.0, y: 0.0, z: 1.0 } => (
            [
                Vector3::new(-HALF, -HALF, HALF),  // back-left-top
                Vector3::new(HALF, -HALF, HALF),   // back-right-top
                Vector3::new(HALF, HALF, HALF),    // front-right-top
                Vector3::new(-HALF, HALF, HALF),   // front-left-top
            ],
            [0, 1, 2, 2, 3, 0]
        ),
        Vector3 { x: 0.0, y: 0.0, z: -1.0 } => (
            [
                Vector3::new(-HALF, HALF, -HALF),  // front-left-bottom
                Vector3::new(HALF, HALF, -HALF),   // front-right-bottom
                Vector3::new(HALF, -HALF, -HALF),  // back-right-bottom
                Vector3::new(-HALF, -HALF, -HALF), // back-left-bottom
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