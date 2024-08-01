use boc_sys as ffi;

use boc::{log, with_scheduler, Cown};

fn main() {
    unsafe {
        // SAFETY: No other work done yet.
        ffi::enable_logging();
    }

    with_scheduler(|| {
        log(c"TOP");
        let v1 = Cown::new(10);
        log(c"Just allocated");
        let v2 = v1.clone();
        log(c"Just cloned");
        drop(v1);
        log(c"dropped v1");
        drop(v2);
        log(c"dropped v2");
    });
}
