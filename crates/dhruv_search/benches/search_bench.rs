use std::path::PathBuf;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dhruv_core::{Engine, EngineConfig};
use dhruv_search::{SankrantiConfig, next_purnima, next_sankranti};
use dhruv_time::UtcTime;
use dhruv_vedic_base::AyanamshaSystem;

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

fn lunar_phase_bench(c: &mut Criterion) {
    let engine = match load_engine() {
        Some(v) => v,
        None => return,
    };
    let utc = UtcTime::new(2024, 3, 20, 12, 0, 0.0);

    let mut group = c.benchmark_group("search_lunar_phase");
    group.sample_size(20);
    group.bench_function("next_purnima", |b| {
        b.iter(|| {
            next_purnima(black_box(&engine), black_box(&utc))
                .expect("search should succeed")
                .expect("event should exist")
        })
    });
    group.finish();
}

fn sankranti_bench(c: &mut Criterion) {
    let engine = match load_engine() {
        Some(v) => v,
        None => return,
    };
    let utc = UtcTime::new(2024, 3, 20, 12, 0, 0.0);
    let config = SankrantiConfig::new(AyanamshaSystem::Lahiri, false);

    let mut group = c.benchmark_group("search_sankranti");
    group.sample_size(20);
    group.bench_function("next_sankranti", |b| {
        b.iter(|| {
            next_sankranti(black_box(&engine), black_box(&utc), black_box(&config))
                .expect("search should succeed")
                .expect("event should exist")
        })
    });
    group.finish();
}

criterion_group!(benches, lunar_phase_bench, sankranti_bench);
criterion_main!(benches);
