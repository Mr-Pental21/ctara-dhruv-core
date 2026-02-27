use criterion::{Criterion, criterion_group, criterion_main};

use dhruv_tara::propagation::propagate_position;

fn bench_propagation(c: &mut Criterion) {
    c.bench_function("propagate_position", |b| {
        b.iter(|| propagate_position(201.298, -11.161, 13.06, -42.50, -31.73, 1.0, 24.0));
    });
}

criterion_group!(benches, bench_propagation);
criterion_main!(benches);
