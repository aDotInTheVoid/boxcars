use std::hint::black_box;

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

fn busyloop_inside_when(n: usize) {
    with_scheduler(|| {
        let c = Cown::new(n);

        when(&c, |c| {
            unsafe { bbench_busy_loop(*c) };
        })
    })
}

extern "C" {
    fn bbench_create_n_cowns(n: usize);
    fn bbench_schedule_n_lambdas_onto_cown(n: usize);
    fn bbeench_busyloop_inside_when(n: usize);

    fn bbench_busy_loop(n: usize);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    time_n_cowns(c);
    time_n_behaviours(c);
    time_busy_loop(c);
}

fn time_busy_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("Busy Loop");

    for i in 0..10 {
        group.bench_with_input(BenchmarkId::new("rust", i), &i, |b, i| {
            b.iter(|| busyloop_inside_when(black_box(*i)))
        });
        group.bench_with_input(BenchmarkId::new("c++", i), &i, |b, i| {
            b.iter(|| unsafe { bbeench_busyloop_inside_when(*i) })
        });
    }
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
