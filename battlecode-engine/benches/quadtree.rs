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

    let entries: Vec<((u8,u8), u32)> = (0..1000)
        .map(|_| ((rng.gen::<u8>() % 64, rng.gen::<u8>() % 64), rng.gen()))
        .collect();

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

    c.bench_function("quadtree range_query[visibility](40) 1000", |b| b.iter(|| {
        let mut seen = fnv::FnvHashSet::default();
        for entry in &entries {
            q.range_query(entry.0, 40, |_, &id| {
                //test::black_box(loc);
                seen.insert(id);
            });
        }
    }));

    let locations: Vec<(u8,u8)> = (0..1000).map(|_| (rng.gen::<u8>() % 64, rng.gen::<u8>() % 64)).collect();
    c.bench_function("quadtree range_query(40) 1000", |b| b.iter(|| {
        for location in &locations {
            q.range_query(*location, 40, |loc, &value| {
                test::black_box((loc, value));
            });
        }
    }));

    let mut q = SpatialHashMap::new();
    for x in 0..20 {
        for y in 0..20 {
            q.insert((x,y), x * 20 + y);
        }
    }
    c.bench_function("quadtree get dense 400", |b| b.iter(|| {
        for x in 0..20 {
            for y in 0..20 {
                q.get((x,y));
            }
        }
    }));
    c.bench_function("quadtree range_query[dense visibility](40) 400", |b| b.iter(|| {
        let mut seen = fnv::FnvHashSet::default();
        for x in 0..20 {
            for y in 0..20 {
                q.range_query((x,y), 40, |_, &value| {
                    seen.insert(value);
                });
            }
        }
    }));





}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

*/