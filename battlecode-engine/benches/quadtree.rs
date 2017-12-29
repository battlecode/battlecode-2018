/*#![feature(test)]
#[macro_use]
extern crate criterion;
extern crate battlecode_engine;
extern crate rand;
extern crate test;
extern crate fnv;

use criterion::Criterion;
use rand::Rng;
use battlecode_engine::spatialhashmap::SpatialHashMap;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::XorShiftRng::new_unseeded();

    let entries: Vec<((u8,u8), u32)> = (0..1000).map(|_| rng.gen()).collect();

    c.bench_function("quadtree insert 1000", |b| b.iter(|| {
        let mut q = SpatialHashMap::new();
        for entry in &entries {
            q.insert(entry.0, entry.1);
        }
    }));

    let mut q = SpatialHashMap::new();
    for entry in &entries {
        q.insert(entry.0, entry.1);
    }

    c.bench_function("quadtree clone 1", |b| b.iter(|| {
        q.clone();
    }));

    c.bench_function("quadtree get 1000", |b| b.iter(|| {
        for entry in &entries {
            q.get(entry.0);
        }
    }));

    c.bench_function("quadtree visibility(10) 1000", |b| b.iter(|| {
        let mut seen = fnv::FnvHashSet::default();
        for entry in &entries {
            q.range_query(entry.0, 10, |loc, _| {
                seen.insert(loc);
            });
        }
    }));

    let locations: Vec<(u8,u8)> = (0..1000).map(|_| rng.gen()).collect();
    c.bench_function("quadtree range_query(10) 1000", |b| b.iter(|| {
        for location in &locations {
            q.range_query(*location, 10, |loc, &value| {
                test::black_box((loc, value));
            });
        }
    }));

    let mut q = SpatialHashMap::new();
    for x in 0..20 {
        for y in 0..20 {
            q.insert((x,y), y);
        }
    }
    c.bench_function("quadtree get dense 1000", |b| b.iter(|| {
        for x in 0..20 {
            for y in 0..20 {
                q.get((x,y));
            }
        }
    }));



}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
*/