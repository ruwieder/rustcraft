use std::u16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    pub id: u16,
}

#[allow(dead_code)]
impl Block {
    pub fn new(block_id: u16) -> Self {
        Self{ id: block_id }
    }
    
    pub fn air() -> Self {
        Self {id: u16::MAX}
    }
    
    #[inline(always)]
    pub fn is_transpose(&self) -> bool {
        self.id == Self::air().id
    }
}
