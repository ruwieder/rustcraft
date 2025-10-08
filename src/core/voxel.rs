type Color = (u8, u8, u8);

#[derive(Debug, Clone, Copy)]
pub struct Voxel {
    pub color: Color,
}

impl Voxel {
    pub fn new(color: Color) -> Self {
        Self{ color }
    }
}
