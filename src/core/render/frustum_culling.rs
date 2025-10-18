use cgmath::{InnerSpace, Matrix4, Point3, Vector4};

use crate::core::chunk::CHUNK_SIZE;

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Vector4<f32>; 6],
}

impl Frustum {
    pub fn from_view_projection(view_proj: &Matrix4<f32>) -> Self {
        let m = view_proj;
        let mut planes = [Vector4::new(0.0, 0.0, 0.0, 0.0); 6];

        // Left plane
        planes[0] = Vector4::new(
            m[0][3] + m[0][0],
            m[1][3] + m[1][0],
            m[2][3] + m[2][0],
            m[3][3] + m[3][0],
        )
        .normalize();

        // Right plane
        planes[1] = Vector4::new(
            m[0][3] - m[0][0],
            m[1][3] - m[1][0],
            m[2][3] - m[2][0],
            m[3][3] - m[3][0],
        )
        .normalize();

        // Bottom plane
        planes[2] = Vector4::new(
            m[0][3] + m[0][1],
            m[1][3] + m[1][1],
            m[2][3] + m[2][1],
            m[3][3] + m[3][1],
        )
        .normalize();

        // Top plane
        planes[3] = Vector4::new(
            m[0][3] - m[0][1],
            m[1][3] - m[1][1],
            m[2][3] - m[2][1],
            m[3][3] - m[3][1],
        )
        .normalize();

        // Near plane
        planes[4] = Vector4::new(
            m[0][3] + m[0][2],
            m[1][3] + m[1][2],
            m[2][3] + m[2][2],
            m[3][3] + m[3][2],
        )
        .normalize();

        // Far plane
        planes[5] = Vector4::new(
            m[0][3] - m[0][2],
            m[1][3] - m[1][2],
            m[2][3] - m[2][2],
            m[3][3] - m[3][2],
        )
        .normalize();

        Self { planes }
    }

    pub fn intersects_aabb(&self, min: Point3<f32>, max: Point3<f32>) -> bool {
        for plane in &self.planes {
            let p = Point3::new(
                if plane.x >= 0.0 { max.x } else { min.x },
                if plane.y >= 0.0 { max.y } else { min.y },
                if plane.z >= 0.0 { max.z } else { min.z },
            );

            if plane.dot(Vector4::new(p.x, p.y, p.z, 1.0)) < 0.0 {
                return false;
            }
        }
        true
    }

    pub fn check(&self, world_pos: &(i64, i64, i64)) -> bool {
        let chunk_aabb_min = Point3::new(
            world_pos.0 as f32 * CHUNK_SIZE as f32,
            world_pos.1 as f32 * CHUNK_SIZE as f32,
            world_pos.2 as f32 * CHUNK_SIZE as f32,
        );
        let chunk_aabb_max = Point3::new(
            (world_pos.0 + 1) as f32 * CHUNK_SIZE as f32,
            (world_pos.1 + 1) as f32 * CHUNK_SIZE as f32,
            (world_pos.2 + 1) as f32 * CHUNK_SIZE as f32,
        );
        self.intersects_aabb(chunk_aabb_min, chunk_aabb_max)
    }
}
