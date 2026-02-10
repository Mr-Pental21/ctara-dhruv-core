use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;

use jpl_kernel::SpkKernel;

fn load_kernel() -> Option<SpkKernel> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kernels/data/de442s.bsp");
    if !path.exists() {
        eprintln!("Skipping benchmarks: kernel not found at {}", path.display());
        return None;
    }
    Some(SpkKernel::load(&path).expect("should load de442s.bsp"))
}

fn eval_benchmarks(c: &mut Criterion) {
    let kernel = match load_kernel() {
        Some(k) => k,
        None => return,
    };

    // J2000.0 epoch (TDB seconds past J2000 = 0.0)
    let epoch = 0.0;

    c.bench_function("evaluate_type2_earth_emb", |b| {
        b.iter(|| {
            // Earth (399) relative to EMB (3) — single segment evaluation
            kernel.evaluate(399, 3, epoch).unwrap()
        });
    });

    c.bench_function("resolve_to_ssb_earth", |b| {
        b.iter(|| {
            // Earth: 399->3->0 (2 hops)
            kernel.resolve_to_ssb(399, epoch).unwrap()
        });
    });

    c.bench_function("resolve_to_ssb_moon", |b| {
        b.iter(|| {
            // Moon: 301->3->0 (2 hops, same chain depth as Earth)
            kernel.resolve_to_ssb(301, epoch).unwrap()
        });
    });
}

criterion_group!(benches, eval_benchmarks);
criterion_main!(benches);
