#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    pub id: u32,
}

#[allow(dead_code)]
impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self::from_id(block_type as u32)
    }

    pub fn from_id(block_id: u32) -> Self {
        Self { id: block_id }
    }

    pub fn air() -> Self {
        Self::new(BlockType::Air)
    }

    #[inline(always)]
    pub fn is_transpose(&self) -> bool {
        self.id == Self::air().id
    }
}

#[repr(u32)]
pub enum BlockType {
    Air = 0,
    Stone = 1,
    Dirt = 2,
    Grass = 3,
}
