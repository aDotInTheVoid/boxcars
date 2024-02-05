use std::thread;

use verona_rt::{with_scheduler, CownPtr};

#[test]
fn main() {
    with_scheduler(|| {
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
