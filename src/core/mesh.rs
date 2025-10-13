use crate::core::render::vertex::Vertex;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_count: u32,
    pub is_dirty: bool
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self{
        Self {
            index_count: indices.len() as u32,
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
            is_dirty: true,
        }
    }
}