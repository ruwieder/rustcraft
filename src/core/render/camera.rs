use cgmath::{InnerSpace, Matrix4, Point3, Vector3, Deg, PerspectiveFov};

pub struct Camera {
    pub pos: Vector3<f32>,
    pub rot: Vector3<f32>,
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
    pub fn new(pos: Vector3<f32>, rot: Vector3<f32>, aspect: f32) -> Self {
        Self {
            pos,
            rot,
            fov: 90.0,
            aspect,
            near: 0.1,
            far: 1000.0
        }
    }
    
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let pitch = self.rot.x;
        let yaw = self.rot.y;
        let up = Vector3::new(0.0, 1.0, 0.0);
        let forward = Vector3::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            -yaw.cos() * pitch.cos(),
        ).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward).normalize();
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
        // uniform.camera_pos = self.pos.into();
        uniform.camera_pos = [self.pos.x, self.pos.y, self.pos.z, 1.0];
    }
}