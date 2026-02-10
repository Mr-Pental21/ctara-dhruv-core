use criterion::{Criterion, criterion_group, criterion_main};
use std::path::PathBuf;
use std::sync::Arc;

use dhruv_core::{Body, Engine, EngineConfig, Frame, Observer, Query};

fn load_engine() -> Option<Engine> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../kernels/data");
    let spk = base.join("de442s.bsp");
    let lsk = base.join("naif0012.tls");
    if !spk.exists() || !lsk.exists() {
        eprintln!("Skipping benchmarks: kernel files not found");
        return None;
    }
    Some(
        Engine::new(EngineConfig {
            spk_paths: vec![spk],
            lsk_path: lsk,
            cache_capacity: 256,
            strict_validation: true,
        })
        .expect("should load engine"),
    )
}

fn single_query_benchmarks(c: &mut Criterion) {
    let engine = match load_engine() {
        Some(e) => e,
        None => return,
    };
    let epoch = 2_451_545.0; // J2000.0

    let mut group = c.benchmark_group("single_query");

    group.bench_function("earth_ssb", |b| {
        let query = Query {
            target: Body::Earth,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: epoch,
        };
        b.iter(|| engine.query(query).unwrap());
    });

    group.bench_function("moon_earth", |b| {
        let query = Query {
            target: Body::Moon,
            observer: Observer::Body(Body::Earth),
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: epoch,
        };
        b.iter(|| engine.query(query).unwrap());
    });

    group.bench_function("mars_earth_ecliptic", |b| {
        let query = Query {
            target: Body::Mars,
            observer: Observer::Body(Body::Earth),
            frame: Frame::EclipticJ2000,
            epoch_tdb_jd: epoch,
        };
        b.iter(|| engine.query(query).unwrap());
    });

    group.finish();
}

fn batch_benchmark(c: &mut Criterion) {
    let engine = match load_engine() {
        Some(e) => e,
        None => return,
    };
    let epoch = 2_451_545.0;

    let bodies = [
        Body::Sun,
        Body::Mercury,
        Body::Venus,
        Body::Earth,
        Body::Moon,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
        Body::Pluto,
    ];

    let queries: Vec<Query> = bodies
        .iter()
        .map(|&b| Query {
            target: b,
            observer: Observer::SolarSystemBarycenter,
            frame: Frame::IcrfJ2000,
            epoch_tdb_jd: epoch,
        })
        .collect();

    c.bench_function("batch_all_bodies_ssb", |b| {
        b.iter(|| engine.query_batch(&queries));
    });
}

fn concurrent_scaling_benchmarks(c: &mut Criterion) {
    let engine = match load_engine() {
        Some(e) => Arc::new(e),
        None => return,
    };

    let query = Query {
        target: Body::Moon,
        observer: Observer::Body(Body::Earth),
        frame: Frame::IcrfJ2000,
        epoch_tdb_jd: 2_451_545.0,
    };

    let mut group = c.benchmark_group("concurrent_scaling");

    for n_threads in [1, 2, 4, 8] {
        group.bench_function(format!("{n_threads}"), |b| {
            b.iter(|| {
                let handles: Vec<_> = (0..n_threads)
                    .map(|_| {
                        let engine = Arc::clone(&engine);
                        std::thread::spawn(move || {
                            for _ in 0..100 {
                                engine.query(query).unwrap();
                            }
                        })
                    })
                    .collect();
                for h in handles {
                    h.join().unwrap();
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    single_query_benchmarks,
    batch_benchmark,
    concurrent_scaling_benchmarks,
);
criterion_main!(benches);
