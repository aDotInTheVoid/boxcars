use boxcars::{log, with_leak_detector, Cown};

// cargo run --example log-around-new --features systematic_testing,snmalloc_tracing
// RUSTFLAGS=-Zsanitizer=address cargo +nightly run --example log-around-new --features asan

fn main() {
    unsafe {
        // SAFETY: No other work done yet.
        verona_rt_sys::enable_logging();
    }

    with_leak_detector(|| {
        log(c"TOP");

        let c1 = Cown::new(101);

        log(c"made allocation, now dropping");

        drop(c1);

        log(c"dropped allocation");
    })
}
