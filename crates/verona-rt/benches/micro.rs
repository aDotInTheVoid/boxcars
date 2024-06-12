use std::{hint::black_box, time::Instant};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use philosophers::do_phil;
use verona_rt::{when, with_n_threads, with_scheduler, Cown};

#[path = "micro/banking.rs"]
mod banking;
#[path = "micro/barber.rs"]
mod barber;
#[path = "micro/philosophers.rs"]
mod philosophers;

fn create_n_cowns(n: usize) -> Vec<Cown<usize>> {
    let mut v = Vec::with_capacity(n);
    for i in 0..n {
        v.push(Cown::<usize>::new(i));
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

fn do_banking(accounts: u64, transactions: u64) {
    with_scheduler(|| {
        let initial = f64::MAX / (accounts * transactions) as f64;

        let teller = banking::Teller::new(initial, accounts, transactions);
        let teller = Cown::new(teller);

        banking::Teller::spawn_transactions(&teller);
    })
}

extern "C" {
    fn bbench_create_n_cowns(n: usize);
    fn bbench_schedule_n_lambdas_onto_cown(n: usize, iters: u64);
    fn bbench_do_par_fib(n: u32, exp: u32, iters: u64);
    fn bbench_do_par_fib_carefull(n: u32, exp: u32, iters: u64);

    fn bbench_busyloop_inside_when(usecs: usize, iters: u64);
    fn boxcars_busy_loop(usecs: usize);

    fn bbench_do_banking(accounts: u64, transactions: u64);
    fn bbench_do_barber(haircuts: u64, room: u64, production: u32, cut: u32);
    fn bbench_do_philosopher(philosophers: u64, rounds: u64, n_threads: usize);
    fn bbench_create_scheduler();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    time_n_cowns(c);
    time_n_behaviours(c);
    time_busy_loop(c);
    time_fib(c);

    time_banking(c);
    time_barber(c);
    time_philosophers(c);

    time_with_sched(c);
}

fn time_philosophers(c: &mut Criterion) {
    let mut group = c.benchmark_group("Philosophers");

    for nthread in 1..10 {
        group.bench_with_input(BenchmarkId::new("Rust", nthread), &nthread, |b, nthread| {
            b.iter(|| {
                do_phil(20, 10000, false, *nthread);
            });
        });

        // group.bench_with_input(
        //     BenchmarkId::new("Rust Optimal", nthread),
        //     &nthread,
        //     |b, nthread| {
        //         b.iter(|| {
        //             do_phil(20, 10000, false, *nthread);
        //         });
        //     },
        // );

        group.bench_with_input(BenchmarkId::new("C++", nthread), &nthread, |b, nthread| {
            b.iter(|| unsafe {
                bbench_do_philosopher(20, 10000, *nthread);
            });
        });
    }
}

fn time_with_sched(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scheduler");

    group.bench_function("Rust", |b| b.iter(|| with_n_threads(1, || { /* no-op */ })));
    group.bench_function("C++", |b| {
        b.iter(|| unsafe {
            bbench_create_scheduler();
        })
    });
}

fn time_barber(c: &mut Criterion) {
    let mut group = c.benchmark_group("savina/Barber");

    group.bench_function("Rust", |b| {
        b.iter(|| {
            barber::bench_barber(5000, 1000, 1000, 1000);
        });
    });
    group.bench_function("C++", |b| {
        b.iter(|| unsafe {
            bbench_do_barber(5000, 1000, 1000, 1000);
        })
    });
}

fn time_banking(c: &mut Criterion) {
    let mut group = c.benchmark_group("savina/Banking");

    group.bench_function("Rust", |b| {
        b.iter(|| {
            do_banking(1000, 50000);
        });
    });

    group.bench_function("C++", |b| {
        b.iter(|| unsafe {
            bbench_do_banking(1000, 50000);
        });
    });
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
