type Color = (u8, u8, u8);

#[derive(Debug, Clone, Copy)]
pub struct Voxel {
    pub color: Color,
    pub tex_id: u16,
}

impl Voxel {
    pub fn new(color: Color, tex_id: u16) -> Self {
        Self{ color, tex_id }
    }
    
    pub fn air() -> Self {
        Self {color: (0, 0, 0), tex_id: 0}
    }
    
    pub fn is_transpose(&self) -> bool {
        self.tex_id == 0
    }
}
