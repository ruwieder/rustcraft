use cgmath::{Deg, InnerSpace, Matrix4, PerspectiveFov, Point3, Vector2, Vector3};
use std::f32::consts::PI;
pub struct Camera {
    pub pos: Vector3<f32>,
    pub rot: Vector2<f32>,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UniformBuffer {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4], // 4 to keep this 80 bytes long
}

impl Default for UniformBuffer {
    fn default() -> Self {
        Self {
            view_proj: [[0.0; 4]; 4],
            camera_pos: [0.0; 4],
        }
    }
}

impl Camera {
    pub fn new(pos: Vector3<f32>, rot: Vector2<f32>, aspect: f32) -> Self {
        Self {
            pos,
            rot,
            fov: 90.0,
            aspect,
            near: 0.1,
            far: 1000.0
        }
    }
    
    pub fn update(&mut self, dt: f64, movement: (f32, f32, f32), mouse_delta: (f32, f32)) {
        let speed = 8.0 * dt as f32;
        let rot_speed = 0.15 * dt as f32;
        
        self.rot.y += mouse_delta.0 * rot_speed; // Yaw (left-right)
        self.rot.x += mouse_delta.1 * rot_speed; // Pitch (up-down)
        self.rot.x = self.rot.x.clamp(-PI / 2.0, PI / 2.0);
        self.rot.y = self.rot.y.rem_euclid(PI * 2.0);
        
        let (forward, right, up) = self.fru();
        self.pos += forward * movement.0 * speed;
        self.pos += right * movement.1 * speed;
        self.pos += up * movement.2 * speed;
    }
    
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let (forward, _, up) = self.fru();
        let pos = Point3::new(self.pos.x, self.pos.y, self.pos.z);
        Matrix4::look_at_rh(pos, pos + forward, up)
    }
    
    pub fn proj_matrix(&self) -> Matrix4<f32> {
        let perspective = PerspectiveFov {
                fovy: Deg(self.fov).into(),
                aspect: self.aspect,
                near: 0.1,
                far: 1000.0,
            };
            Matrix4::from(perspective)
    }
    
    pub fn update_uniform(&self, uniform: &mut UniformBuffer) {
        let view = self.view_matrix();
        let proj = self.proj_matrix();
        uniform.view_proj = (proj * view).into();
        uniform.camera_pos = [self.pos.x, self.pos.y, self.pos.z, 69.0];
    }
    
    pub fn fru(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        // forward - right - up
        let (sin_yaw, cos_yaw) = (self.rot.y.sin(), self.rot.y.cos());
        let (sin_pitch, cos_pitch) = (self.rot.x.sin(), self.rot.x.cos());
        let forward = Vector3::new(
            cos_pitch * cos_yaw,
            cos_pitch * sin_yaw, 
            sin_pitch
        ).normalize();
        let right = Vector3::new(
            -sin_yaw,
            cos_yaw,
            0.0
        ).normalize();
        let up = forward.cross(right).normalize();
        (forward, right, up)
    }
}