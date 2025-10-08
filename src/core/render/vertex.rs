use cgmath::Vector3;
use wgpu::VertexFormat;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub const fn new(pos: [f32; 3], color: [f32; 3]) -> Self {
        Self { pos, color }
    }
    
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout { 
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
            ]
        }
    }
}

const CUBE_VERTICES: &[Vertex] = &[
    Vertex::new([-0.5, -0.5, -0.5], [1.0, 1.0, 1.0]),
    Vertex::new([0.5, -0.5, -0.5], [1.0, 1.0, 1.0]),
    Vertex::new([0.5, -0.5, 0.5], [1.0, 1.0, 1.0]),
    Vertex::new([-0.5, -0.5, 0.5], [1.0, 1.0, 1.0]),
    Vertex::new([-0.5, 0.5, -0.5], [1.0, 1.0, 1.0]),
    Vertex::new([0.5, 0.5, -0.5], [1.0, 1.0, 1.0]),
    Vertex::new([0.5, 0.5, 0.5], [1.0, 1.0, 1.0]),
    Vertex::new([-0.5, 0.5, 0.5], [1.0, 1.0, 1.0]),
];

const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // Bottom
    4, 6, 5, 4, 7, 6, // Top
    0, 5, 1, 0, 4, 5, // Front
    3, 2, 6, 3, 6, 7, // Back
    0, 3, 7, 0, 7, 4, // Left
    1, 5, 6, 1, 6, 2, // Right
];

pub fn generate_voxel_mesh(voxel_pos: Vector3<f32>, color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(CUBE_VERTICES.len());
    let mut indices = Vec::with_capacity(CUBE_INDICES.len());
    // let indices = CUBE_INDICES.clone();
    
    for &v in CUBE_VERTICES {   // Offset cube vertices by voxel position
        let pos = [
            v.pos[0] + voxel_pos.x,
            v.pos[1] + voxel_pos.y,
            v.pos[2] + voxel_pos.z,
        ];
        vertices.push(Vertex::new(pos, color));
    }
    // Use same indices (they're relative to the 8 vertices)
    indices.extend_from_slice(CUBE_INDICES);
    (vertices, indices)
}