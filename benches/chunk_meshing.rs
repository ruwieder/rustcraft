use std::time::Duration;

use cgmath::Vector3;
use criterion::{Criterion, criterion_group, criterion_main};
use rustcraft::{core::chunk::Chunk, world::World};

pub fn single(c: &mut Criterion) {
    let chunk = Chunk::terrain_gen(Vector3::new(0, 0, 0), 6969);
    let mut world = World::new(6969);
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                world.need_to_load.push_back((x, y, z));
            }
        }
    }
    world.load_new(Duration::from_secs(1000));

    c.bench_function("Generating mesh of single chunk 32", |b| {
        b.iter(|| {
            let chunk_copy = Chunk {
                blocks: chunk.blocks.clone(),
                ..chunk
            };
            let _ = chunk_copy.generate_mesh(&world);
        })
    });
}

criterion_group!(benches, single);
criterion_main!(benches);
