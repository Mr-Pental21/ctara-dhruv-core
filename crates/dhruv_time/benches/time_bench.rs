use std::path::PathBuf;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dhruv_time::{
    LeapSecondKernel, calendar_to_jd, gmst_rad, jd_to_tdb_seconds, local_sidereal_time_rad,
};

fn load_lsk() -> Option<LeapSecondKernel> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../kernels/data");
    let lsk = base.join("naif0012.tls");
    if !lsk.exists() {
        eprintln!("Skipping benchmarks: LSK not found");
        return None;
    }
    Some(LeapSecondKernel::load(&lsk).expect("should load naif0012.tls"))
}

fn utc_tdb_bench(c: &mut Criterion) {
    let lsk = match load_lsk() {
        Some(v) => v,
        None => return,
    };
    let jd_utc = calendar_to_jd(2024, 3, 20.5);
    let utc_s = jd_to_tdb_seconds(jd_utc);
    let tdb_s = lsk.utc_to_tdb(utc_s);

    let mut group = c.benchmark_group("time_scale");
    group.bench_function("utc_to_tdb", |b| {
        b.iter(|| lsk.utc_to_tdb(black_box(utc_s)))
    });
    group.bench_function("tdb_to_utc", |b| {
        b.iter(|| lsk.tdb_to_utc(black_box(tdb_s)))
    });
    group.finish();
}

fn sidereal_bench(c: &mut Criterion) {
    let jd_ut1 = 2_460_000.5;
    let lon_rad = 77.216721_f64.to_radians();

    let mut group = c.benchmark_group("sidereal");
    group.bench_function("gmst_rad", |b| b.iter(|| gmst_rad(black_box(jd_ut1))));
    group.bench_function("local_sidereal_time_rad", |b| {
        b.iter(|| local_sidereal_time_rad(black_box(gmst_rad(jd_ut1)), black_box(lon_rad)))
    });
    group.finish();
}

criterion_group!(benches, utc_tdb_bench, sidereal_bench);
criterion_main!(benches);
