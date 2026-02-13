use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dhruv_frames::{
    SphericalCoords, cartesian_state_to_spherical_state, cartesian_to_spherical, ecliptic_to_icrf,
    fundamental_arguments, general_precession_longitude_deg, icrf_to_ecliptic, nutation_iau2000b,
    spherical_to_cartesian,
};

fn frame_rotation_bench(c: &mut Criterion) {
    let v = [1.2e8, -4.8e7, 9.3e6];

    let mut group = c.benchmark_group("frame_rotation");
    group.bench_function("icrf_to_ecliptic", |b| {
        b.iter(|| icrf_to_ecliptic(black_box(&v)))
    });
    group.bench_function("ecliptic_to_icrf", |b| {
        b.iter(|| ecliptic_to_icrf(black_box(&v)))
    });
    group.finish();
}

fn spherical_bench(c: &mut Criterion) {
    let pos = [2.4e8, 7.1e7, -1.2e7];
    let vel = [12.0, -23.0, 4.0];
    let sph = SphericalCoords {
        lon_deg: 210.0,
        lat_deg: -3.5,
        distance_km: 1.5e8,
    };

    let mut group = c.benchmark_group("spherical");
    group.bench_function("cartesian_to_spherical", |b| {
        b.iter(|| cartesian_to_spherical(black_box(&pos)))
    });
    group.bench_function("spherical_to_cartesian", |b| {
        b.iter(|| spherical_to_cartesian(black_box(&sph)))
    });
    group.bench_function("cartesian_state_to_spherical_state", |b| {
        b.iter(|| cartesian_state_to_spherical_state(black_box(&pos), black_box(&vel)))
    });
    group.finish();
}

fn precession_nutation_bench(c: &mut Criterion) {
    let t = 0.24; // Julian centuries since J2000

    let mut group = c.benchmark_group("precession_nutation");
    group.bench_function("general_precession_longitude_deg", |b| {
        b.iter(|| general_precession_longitude_deg(black_box(t)))
    });
    group.bench_function("fundamental_arguments", |b| {
        b.iter(|| fundamental_arguments(black_box(t)))
    });
    group.bench_function("nutation_iau2000b", |b| {
        b.iter(|| nutation_iau2000b(black_box(t)))
    });
    group.finish();
}

criterion_group!(
    benches,
    frame_rotation_bench,
    spherical_bench,
    precession_nutation_bench
);
criterion_main!(benches);
