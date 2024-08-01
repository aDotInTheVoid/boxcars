use std::time::{Duration, Instant};

use boc::log_snmalloc;
use boc::{with_leak_detector, Cown};

// RUSTFLAGS='--cfg slow_tests' cargo test --all
#[cfg(slow_tests)]
const TIME_TO_RUN: Duration = Duration::from_secs(10);
#[cfg(not(slow_tests))]
const TIME_TO_RUN: Duration = Duration::from_secs(1);

#[test]
fn main() {
    let start = Instant::now();

    while start.elapsed() < TIME_TO_RUN {
        std::thread::spawn(one_run).join().unwrap();
    }
}

fn one_run() {
    with_leak_detector(|| {
        log_snmalloc("!! begin main");

        stdx::thread::scope(|s| {
            for _ in 0..10 {
                create_sched_noise(&s);

                s.spawn(|| {
                    log_snmalloc("!! begin manipulation");
                    {
                        let mut v = Vec::new();

                        for i in 0..100 {
                            v.push(Cown::new(i));
                        }

                        let mut vs = Vec::new();

                        for _ in 0..100 {
                            vs.push(v.clone());
                        }
                        log_snmalloc("!! running dtors");
                    }
                    log_snmalloc("!! done manipulation");
                });
            }

            log_snmalloc("!! scope over, awaiting  join");
        });
        log_snmalloc("!! all joined, end main");
    })
}

fn create_sched_noise(s: &stdx::thread::Scope) {
    for spinc in 1..100 {
        let f = move || {
            for _ in 0..spinc * 100 {
                std::hint::spin_loop()
            }
        };

        s.spawn(f);
    }
}
