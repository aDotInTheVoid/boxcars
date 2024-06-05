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

fn schedule_n_lambdas_onto_cown(n: usize, iters: u64) {
    with_scheduler(|| {
        let threader = Cown::new(0);

        for _ in 0..iters {
            let c = Cown::new(0);
            for _ in 0..n {
                when(&c, |mut c| {
                    *c += 1;
                })
            }

            when((&c, &threader), |_| {});
        }
    });
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

extern "C" {
    fn bbench_create_n_cowns(n: usize);
    fn bbench_schedule_n_lambdas_onto_cown(n: usize, iters: u64);
    fn bbench_do_par_fib(n: u32, exp: u32, iters: u64);
    fn bbench_do_par_fib_carefull(n: u32, exp: u32, iters: u64);

    fn bbench_busyloop_inside_when(usecs: usize, iters: u64);
    fn boxcars_busy_loop(usecs: usize);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    time_n_cowns(c);
    time_n_behaviours(c);
    time_busy_loop(c);
    time_fib(c);
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

fn time_n_cowns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Create Cowns");

    for i in (10..16).map(|i| 2usize.pow(i)) {
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

    for i in (10..16).map(|i| 2usize.pow(i)) {
        group.bench_with_input(BenchmarkId::new("rust", i), &i, |b, i| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                schedule_n_lambdas_onto_cown(*i, iters);
                start.elapsed()
            });
        });
        group.bench_with_input(BenchmarkId::new("c++", i), &i, |b, i| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                unsafe { bbench_schedule_n_lambdas_onto_cown(*i, iters) };
                start.elapsed()
            });
        });
    }
}

fn sequential_fib(n: u32) -> u32 {
    if n <= 1 {
        n
    } else {
        sequential_fib(n - 1) + sequential_fib(n - 2)
    }
}

fn parallel_fib_into(n: u32, result: &Cown<u32>) {
    if n <= 4 {
        when(result, move |mut r| *r = sequential_fib(n));
    } else {
        let f1 = Cown::new(0);
        parallel_fib_into(n - 1, &f1);
        parallel_fib_into(n - 2, result);
        when((result, &f1), |(mut r, f)| *r += *f);
    }
}

fn parallel_fib_into_uncarefull(n: u32, result: Cown<u32>) {
    if n <= 4 {
        when(&result, move |mut r| *r = sequential_fib(n));
    } else {
        let f1 = Cown::new(0);
        parallel_fib_into_uncarefull(n - 1, f1.clone());
        parallel_fib_into_uncarefull(n - 2, result.clone());
        when((&result, &f1), |(mut r, f)| *r += *f);
    }
}

fn do_par_fib(n: u32, exp: u32, iters: u64) {
    with_scheduler(|| {
        let r = Cown::new(0);

        for _ in 0..iters {
            parallel_fib_into(n, &r);
            when(&r, move |r| assert_eq!(exp, *r));
        }
    })
}

fn do_par_fib_uncarefull(n: u32, exp: u32, iters: u64) {
    with_scheduler(|| {
        let r = Cown::new(0);

        for _ in 0..iters {
            parallel_fib_into_uncarefull(n, r.clone());
            when(&r, move |r| assert_eq!(exp, *r));
        }
    })
}

fn time_fib(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fibonacci");

    for i in 14..22 {
        group.bench_with_input(BenchmarkId::new("Rust", i), &i, |b, n| {
            b.iter_custom(|iters| {
                let exp = sequential_fib(*n);

                let start = Instant::now();
                do_par_fib(*n, exp, iters);
                start.elapsed()
            });
        });

        group.bench_with_input(BenchmarkId::new("Rust Uncarefull", i), &i, |b, n| {
            b.iter_custom(|iters| {
                let exp = sequential_fib(*n);

                let start = Instant::now();
                do_par_fib_uncarefull(*n, exp, iters);
                start.elapsed()
            });
        });

        group.bench_with_input(BenchmarkId::new("C++", i), &i, |b, n| {
            b.iter_custom(|iters| {
                let exp = sequential_fib(*n);
                let start = Instant::now();
                unsafe {
                    bbench_do_par_fib(*n, exp, iters);
                }
                start.elapsed()
            });
        });

        group.bench_with_input(BenchmarkId::new("C++ carefull", i), &i, |b, n| {
            b.iter_custom(|iters| {
                let exp = sequential_fib(*n);
                let start = Instant::now();
                unsafe {
                    bbench_do_par_fib_carefull(*n, exp, iters);
                }
                start.elapsed()
            });
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
