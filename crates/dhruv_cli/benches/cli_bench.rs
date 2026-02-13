use std::path::PathBuf;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dhruv_core::{Engine, EngineConfig};
use dhruv_search::{SankrantiConfig, next_purnima, panchang_for_date};
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::{AyanamshaSystem, GeoLocation, RiseSetConfig};

struct CliBenchContext {
    engine: Engine,
    eop: EopKernel,
    utc: UtcTime,
    location: GeoLocation,
    riseset_config: RiseSetConfig,
    sankranti_config: SankrantiConfig,
}

fn load_context() -> Option<CliBenchContext> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../kernels/data");
    let spk = base.join("de442s.bsp");
    let lsk = base.join("naif0012.tls");
    let eop = base.join("finals2000A.all");
    if !spk.exists() || !lsk.exists() || !eop.exists() {
        eprintln!("Skipping benchmarks: kernel/EOP files not found");
        return None;
    }

    let engine = Engine::new(EngineConfig {
        spk_paths: vec![spk],
        lsk_path: lsk,
        cache_capacity: 256,
        strict_validation: true,
    })
    .expect("should load engine");

    let eop = EopKernel::load(&eop).expect("should load EOP");
    Some(CliBenchContext {
        engine,
        eop,
        utc: UtcTime::new(2024, 3, 20, 12, 0, 0.0),
        location: GeoLocation::new(12.9716, 77.5946, 920.0),
        riseset_config: RiseSetConfig::default(),
        sankranti_config: SankrantiConfig::new(AyanamshaSystem::Lahiri, false),
    })
}

fn cli_like_search_bench(c: &mut Criterion) {
    let ctx = match load_context() {
        Some(v) => v,
        None => return,
    };

    let mut group = c.benchmark_group("cli_like_search");
    group.sample_size(20);
    group.bench_function("next_purnima_command_path", |b| {
        b.iter(|| {
            next_purnima(black_box(&ctx.engine), black_box(&ctx.utc))
                .expect("search should succeed")
                .expect("event should exist")
        })
    });
    group.finish();
}

fn cli_like_panchang_bench(c: &mut Criterion) {
    let ctx = match load_context() {
        Some(v) => v,
        None => return,
    };

    let mut group = c.benchmark_group("cli_like_panchang");
    group.sample_size(20);
    group.bench_function("panchang_command_path", |b| {
        b.iter(|| {
            panchang_for_date(
                black_box(&ctx.engine),
                black_box(&ctx.eop),
                black_box(&ctx.utc),
                black_box(&ctx.location),
                black_box(&ctx.riseset_config),
                black_box(&ctx.sankranti_config),
                true,
            )
            .expect("panchang should succeed")
        })
    });
    group.finish();
}

criterion_group!(benches, cli_like_search_bench, cli_like_panchang_bench);
criterion_main!(benches);
