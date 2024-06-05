use std::{hint::black_box, time::Instant};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use verona_rt::{when, with_scheduler, Cown};

fn create_n_cowns(n: usize) -> Vec<Cown<usize>> {
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(Cown::new(n));
    }
    v
}

fn schedule_n_lambdas_onto_cown(n: usize) {
    with_scheduler(|| {
        let c = Cown::new(0);
        for _ in 0..n {
            when(&c, |mut c| {
                *c += 1;
            })
        }
    });
}

extern "C" {
    fn bbench_create_n_cowns(n: usize);
    fn bbench_schedule_n_lambdas_onto_cown(n: usize);

    fn bbench_busyloop_inside_when(usecs: usize, iters: u64);
    fn boxcars_busy_loop(usecs: usize);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    time_n_cowns(c);
    time_n_behaviours(c);
    time_busy_loop(c);
}

fn time_busy_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("Busy Loop");

    for nsecs in 0..5 {
        group.bench_with_input(BenchmarkId::new("Rust", nsecs), &nsecs, |b, nsecs| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                busyloop_inside_when(*nsecs, iters);
                start.elapsed()
            })
        });

        group.bench_with_input(BenchmarkId::new("C++", nsecs), &nsecs, |b, nsecs| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                unsafe {
                    bbench_busyloop_inside_when(*nsecs, iters);
                }
                start.elapsed()
            })
        });
    }
}

fn busyloop_inside_when(usecs: usize, iters: u64) {
    with_scheduler(|| {
        let c = Cown::new(usecs);

        for _ in 0..iters {
            when(&c, |c| unsafe {
                boxcars_busy_loop(*c);
            })
        }
    });
}

fn time_n_cowns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Create Cowns");

    for i in (1..5).map(|i| i * 1000) {
        group.bench_with_input(BenchmarkId::new("rust", i), &i, |b, i| {
            b.iter(|| black_box(create_n_cowns(black_box(*i))))
        });
        group.bench_with_input(BenchmarkId::new("c++", i), &i, |b, i| {
            b.iter(|| unsafe { bbench_create_n_cowns(*i) })
        });
    }
}

fn time_n_behaviours(c: &mut Criterion) {
    let mut group = c.benchmark_group("Schedule Behaviours");

    for i in (1..5).map(|i| i * 1000) {
        group.bench_with_input(BenchmarkId::new("rust", i), &i, |b, i| {
            b.iter(|| schedule_n_lambdas_onto_cown(black_box(*i)))
        });
        group.bench_with_input(BenchmarkId::new("c++", i), &i, |b, i| {
            b.iter(|| unsafe { bbench_schedule_n_lambdas_onto_cown(*i) })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
