use cgmath::Vector3;

use crate::core::render::vertex::Vertex;

pub fn generate_face(
    pos: Vector3<f32>,
    color: [f32; 3],
    normal: Vector3<f32>,
    texture_id: u16,
    width: f32,
    height: f32,
) -> (Vec<Vertex>, Vec<u32>) {
    // println!("{pos:?}; {normal:?}; {width:?}; {height:?}");
    const ATLAS_WIDTH: f32 = 1024.0;
    const ATLAS_HEIGHT: f32 = 512.0;
    const TEXTURE_SIZE: f32 = 16.0;
    const TEXTURES_PER_ROW: u16 = (ATLAS_WIDTH / TEXTURE_SIZE) as u16;
    const TEXTURES_PER_COL: u16 = (ATLAS_HEIGHT / TEXTURE_SIZE) as u16;
    const TEX_SIZE_U: f32 = TEXTURE_SIZE / ATLAS_WIDTH; 
    const TEX_SIZE_V: f32 = TEXTURE_SIZE / ATLAS_HEIGHT;
    const GAP: f32 = 0.01;
    
    let half_width = width * 0.5 - GAP;
    let half_height = height * 0.5 - GAP;
    
    let texture_x = texture_id % TEXTURES_PER_ROW;
    let texture_y = texture_id / TEXTURES_PER_ROW;
    
    let tex_start_u = texture_x as f32 * TEX_SIZE_U;
    let tex_start_v = texture_y as f32 * TEX_SIZE_V;
    
    let uvs = [
        [tex_start_u, tex_start_v],
        [tex_start_u + TEX_SIZE_U, tex_start_v],
        [tex_start_u + TEX_SIZE_U, tex_start_v + TEX_SIZE_V],
        [tex_start_u, tex_start_v + TEX_SIZE_V],
    ];

    let vertices = match normal {
        Vector3 { x: 1.0, .. } => {
            vec![
                Vertex { pos: [pos.x + 0.5, pos.y - half_width, pos.z - half_height], tex_coord: uvs[3], color },
                Vertex { pos: [pos.x + 0.5, pos.y + half_width, pos.z - half_height], tex_coord: uvs[2], color },
                Vertex { pos: [pos.x + 0.5, pos.y + half_width, pos.z + half_height], tex_coord: uvs[1], color },
                Vertex { pos: [pos.x + 0.5, pos.y - half_width, pos.z + half_height], tex_coord: uvs[0], color },
            ]
        },
        Vector3 { x: -1.0, .. } => {
            vec![
                Vertex { pos: [pos.x - 0.5, pos.y + half_width, pos.z - half_height], tex_coord: uvs[3], color },
                Vertex { pos: [pos.x - 0.5, pos.y - half_width, pos.z - half_height], tex_coord: uvs[2], color },
                Vertex { pos: [pos.x - 0.5, pos.y - half_width, pos.z + half_height], tex_coord: uvs[1], color },
                Vertex { pos: [pos.x - 0.5, pos.y + half_width, pos.z + half_height], tex_coord: uvs[0], color },
            ]
        },
        Vector3 { y: 1.0, .. } => {
            vec![
                Vertex { pos: [pos.x + half_width, pos.y + 0.5, pos.z - half_height], tex_coord: uvs[2], color },
                Vertex { pos: [pos.x - half_width, pos.y + 0.5, pos.z - half_height], tex_coord: uvs[3], color },
                Vertex { pos: [pos.x - half_width, pos.y + 0.5, pos.z + half_height], tex_coord: uvs[0], color },
                Vertex { pos: [pos.x + half_width, pos.y + 0.5, pos.z + half_height], tex_coord: uvs[1], color },
            ]
        },
        Vector3 { y: -1.0, .. } => {
            vec![
                Vertex { pos: [pos.x + half_width, pos.y - 0.5, pos.z + half_height], tex_coord: uvs[1], color },
                Vertex { pos: [pos.x - half_width, pos.y - 0.5, pos.z + half_height], tex_coord: uvs[0], color },
                Vertex { pos: [pos.x - half_width, pos.y - 0.5, pos.z - half_height], tex_coord: uvs[3], color },
                Vertex { pos: [pos.x + half_width, pos.y - 0.5, pos.z - half_height], tex_coord: uvs[2], color },
            ]
        },
        Vector3 { z: 1.0, .. } => {
            vec![
                Vertex { pos: [pos.x - half_width, pos.y - half_height, pos.z + 0.5], tex_coord: uvs[2], color },
                Vertex { pos: [pos.x + half_width, pos.y - half_height, pos.z + 0.5], tex_coord: uvs[1], color },
                Vertex { pos: [pos.x + half_width, pos.y + half_height, pos.z + 0.5], tex_coord: uvs[0], color },
                Vertex { pos: [pos.x - half_width, pos.y + half_height, pos.z + 0.5], tex_coord: uvs[3], color },
            ]
        },
        Vector3 { z: -1.0, .. } => {
            vec![
                Vertex { pos: [pos.x + half_width, pos.y - half_height, pos.z - 0.5], tex_coord: uvs[1], color },
                Vertex { pos: [pos.x - half_width, pos.y - half_height, pos.z - 0.5], tex_coord: uvs[2], color },
                Vertex { pos: [pos.x - half_width, pos.y + half_height, pos.z - 0.5], tex_coord: uvs[3], color },
                Vertex { pos: [pos.x + half_width, pos.y + half_height, pos.z - 0.5], tex_coord: uvs[0], color },
            ]
        },
        _ => Vec::new(),
    };

    let indices = vec![0, 1, 2, 2, 3, 0];

    (vertices, indices)
}