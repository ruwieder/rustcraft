use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use rustcraft::world::World;

pub fn terrain_generation_single(c: &mut Criterion) {
    c.bench_function("Generating single chunk", |b| {
        b.iter(|| {
            let mut w = World::new(6969);
            w.need_to_load.push_back((0, 0, 0));
            w.load_new(Duration::from_secs(1000));
        })
    });
}
pub fn terrain_generation_5x5x5(c: &mut Criterion) {
    c.bench_function("Generating 5x5x5 chunks 32", |b| {
        b.iter(|| {
            let mut w = World::new(6969);
            for x in -2..3 {
                for y in -2..3 {
                    for z in -2..3 {
                        w.need_to_load.push_back((x, y, z));
                    }
                }
            }
            w.load_new(Duration::from_secs(1000));
        })
    });
}
criterion_group!(benches, terrain_generation_single, terrain_generation_5x5x5);
criterion_main!(benches);
