type Color = (u8, u8, u8);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    pub color: Color,
    pub id: u16,
}

#[allow(dead_code)]
impl Block {
    pub fn new(color: Color, block_id: u16) -> Self {
        Self{ color, id: block_id }
    }
    
    pub fn air() -> Self {
        Self {color: (0, 0, 0), id: 0}
    }
    
    pub fn is_transpose(&self) -> bool {
        self.id == 0
    }
}
