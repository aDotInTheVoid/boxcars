use std::thread;

use verona_rt::{cown::CownPtr, scheduler};

#[test]
fn main() {
    scheduler::with_leak_detector(|| {
        thread::scope(|s| {
            for _ in 0..10 {
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
