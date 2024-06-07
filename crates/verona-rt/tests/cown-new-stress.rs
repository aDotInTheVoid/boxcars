use verona_rt::{with_leak_detector, Cown};

#[test]
fn main() {
    with_leak_detector(|| {
        stdx::thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut v = Vec::new();

                    for i in 0..100 {
                        v.push(Cown::new(i));
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
