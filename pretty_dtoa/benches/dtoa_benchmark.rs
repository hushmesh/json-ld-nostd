use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pretty_dtoa::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("3.14159 -> string (builtin Display)", |b| {
        b.iter(|| format!("{}", black_box(3.14159f64)))
    });
    c.bench_function(
        "3.14159 -> string (display_float dtoa default config)",
        |b| b.iter(|| dtoa(black_box(3.14159f64), FmtFloatConfig::default())),
    );

    c.bench_function("3.14e10 -> string (builtin Display)", |b| {
        b.iter(|| format!("{}", black_box(3.14e10f64)))
    });
    c.bench_function(
        "3.14e10 -> string (display_float dtoa default config)",
        |b| b.iter(|| dtoa(black_box(3.14e10), FmtFloatConfig::default())),
    );

    c.bench_function("13124014 -> string (builtin Display)", |b| {
        b.iter(|| format!("{}", black_box(13124014f64)))
    });
    c.bench_function(
        "13124014 -> string (display_float dtoa default config)",
        |b| b.iter(|| dtoa(black_box(13124014f64), FmtFloatConfig::default())),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
