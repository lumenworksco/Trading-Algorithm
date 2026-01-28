//! Benchmarks for indicator implementations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use trading_indicators::{simd, Sma, Ema, Rsi};
use trading_core::traits::Indicator;

fn generate_test_data(size: usize) -> Vec<f64> {
    (0..size)
        .map(|i| 100.0 + (i as f64 * 0.1).sin() * 10.0)
        .collect()
}

fn benchmark_sma(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMA");

    for size in [1000, 10000, 100000].iter() {
        let data = generate_test_data(*size);

        group.bench_with_input(BenchmarkId::new("standard", size), &data, |b, data| {
            let sma = Sma::new(20);
            b.iter(|| sma.calculate(black_box(data)))
        });

        group.bench_with_input(BenchmarkId::new("simd", size), &data, |b, data| {
            b.iter(|| simd::sma_simd(black_box(data), black_box(20)))
        });
    }

    group.finish();
}

fn benchmark_ema(c: &mut Criterion) {
    let mut group = c.benchmark_group("EMA");

    for size in [1000, 10000, 100000].iter() {
        let data = generate_test_data(*size);

        group.bench_with_input(BenchmarkId::new("standard", size), &data, |b, data| {
            let ema = Ema::new(20);
            b.iter(|| ema.calculate(black_box(data)))
        });

        group.bench_with_input(BenchmarkId::new("simd", size), &data, |b, data| {
            b.iter(|| simd::ema_simd(black_box(data), black_box(20)))
        });
    }

    group.finish();
}

fn benchmark_rsi(c: &mut Criterion) {
    let mut group = c.benchmark_group("RSI");

    for size in [1000, 10000, 100000].iter() {
        let data = generate_test_data(*size);

        group.bench_with_input(BenchmarkId::new("standard", size), &data, |b, data| {
            let rsi = Rsi::new(14);
            b.iter(|| rsi.calculate(black_box(data)))
        });

        group.bench_with_input(BenchmarkId::new("simd", size), &data, |b, data| {
            b.iter(|| simd::rsi_simd(black_box(data), black_box(14)))
        });
    }

    group.finish();
}

fn benchmark_std_dev(c: &mut Criterion) {
    let mut group = c.benchmark_group("StdDev");

    for size in [1000, 10000, 100000].iter() {
        let data = generate_test_data(*size);

        group.bench_with_input(BenchmarkId::new("simd", size), &data, |b, data| {
            b.iter(|| simd::std_dev_simd(black_box(data), black_box(20)))
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_sma, benchmark_ema, benchmark_rsi, benchmark_std_dev);
criterion_main!(benches);
