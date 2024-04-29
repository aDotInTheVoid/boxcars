use std::{
    thread,
    time::{Duration, Instant},
};

use verona_rt::{with_leak_detector, CownPtr};

const TIME_TO_RUN: Duration = Duration::from_secs(10);

#[test]
fn main() {
    let start = Instant::now();

    while start.elapsed() < TIME_TO_RUN {
        std::thread::spawn(one_run).join().unwrap();
    }
}

fn one_run() {
    with_leak_detector(|| {
        thread::scope(|s| {
            for _ in 0..10 {
                create_sched_noise(s);

                s.spawn(|| {
                    let mut v = Vec::new();

                    for i in 0..100 {
                        v.push(CownPtr::new(i));
                    }

                    let mut vs = Vec::new();

                    for _ in 0..100 {
                        vs.push(v.clone());
                    }
                });
            }
        })
    })
}

fn create_sched_noise<'a: 'b, 'b>(s: &'a thread::Scope<'b, '_>) {
    for spinc in 1..100 {
        let f = move || {
            for _ in 0..spinc * 100 {
                std::hint::spin_loop()
            }
        };

        s.spawn(f);
    }
}
