use std::path::PathBuf;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dhruv_core::Query;
use dhruv_time::UtcTime;
use dhruv_rs::{
    Body, DhruvContext, EngineConfig, Frame, LunarPhaseKind, LunarPhaseRequest,
    LunarPhaseRequestQuery, LunarPhaseResult, Observer, TimeInput, UtcDate, lunar_phase,
};

fn make_context() -> Option<DhruvContext> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../kernels/data");
    let spk = base.join("de442s.bsp");
    let lsk = base.join("naif0012.tls");
    if !spk.exists() || !lsk.exists() {
        eprintln!("Skipping benchmarks: kernel files not found");
        return None;
    }

    let config = EngineConfig::with_single_spk(spk, lsk, 256, true);
    Some(DhruvContext::new(config).expect("context init"))
}

fn rs_query_bench(c: &mut Criterion) {
    let Some(ctx) = make_context() else {
        return;
    };
    let date = UtcDate::new(2024, 3, 20, 12, 0, 0.0);
    let utc: UtcTime = date.into();
    let query = Query {
        target: Body::Mars,
        observer: Observer::Body(Body::Earth),
        frame: Frame::EclipticJ2000,
        epoch_tdb_jd: utc.to_jd_tdb(ctx.engine().lsk()),
    };

    let mut group = c.benchmark_group("dhruv_rs_query");
    group.bench_function("longitude_mars_earth", |b| {
        b.iter(|| {
            ctx.engine()
                .query(black_box(query))
                .expect("query should succeed")
        })
    });
    group.finish();
}

fn rs_search_bench(c: &mut Criterion) {
    let Some(ctx) = make_context() else {
        return;
    };
    let date = UtcDate::new(2024, 3, 20, 12, 0, 0.0);

    let mut group = c.benchmark_group("dhruv_rs_search");
    group.sample_size(20);
    group.bench_function("lunar_phase_next_purnima", |b| {
        b.iter(|| {
            let req = LunarPhaseRequest {
                kind: LunarPhaseKind::Purnima,
                query: LunarPhaseRequestQuery::Next {
                    at: TimeInput::Utc(black_box(date)),
                },
            };
            match lunar_phase(&ctx, &req).expect("search should succeed") {
                LunarPhaseResult::Single(Some(_)) => {}
                other => panic!("unexpected result shape: {other:?}"),
            }
        })
    });
    group.finish();
}

criterion_group!(benches, rs_query_bench, rs_search_bench);
criterion_main!(benches);
