#[allow(dead_code)]
#[allow(unused_imports)]

pub mod render;
pub mod world;
pub mod voxel;
pub mod chunk;
use voxel::Voxel;
use chunk::{Chunk, CHUNK_SIZE};
use render::vertex::{Vertex, generate_voxel_mesh};
use world::world::World;
