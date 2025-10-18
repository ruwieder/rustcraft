use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use rustcraft::world::World;

pub fn single(c: &mut Criterion) {
    let mut world = World::new(6969);
    c.bench_function("Generating and meshing single chunk 32", |b| {
        b.iter(|| {
            world.need_to_load.push_back((0, 0, 0));
            world.load_new(Duration::from_secs(100));
            world.chunks.get(&(0, 0, 0)).unwrap().generate_mesh(&world);
            world.chunks.clear();
        })
    });
}

pub fn thousand(c: &mut Criterion) {
    let mut world = World::new(6969);
    let mut keys = Vec::new();
    for x in -2..3 {
        for y in -2..3 {
            for z in -2..3 {
                keys.push((x, y, z));
            }
        }
    }
    c.bench_function("Generating and meshing of 5x5x5 chunks 32", |b| {
        b.iter(|| {
            for key in &keys {
                world.need_to_load.push_back(*key);
            }
            world.load_new(Duration::from_secs(1000));
            for key in &keys {
                world.chunks.get(key).unwrap().generate_mesh(&world);
            }
            world.chunks.clear();
        })
    });
}
criterion_group!(benches, single, thousand);
criterion_main!(benches);
