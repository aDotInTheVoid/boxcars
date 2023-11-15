use cstr::cstr;
use verona_rt_sys as ffi;

use verona_rt::{cown::CownPtr, log::log, scheduler};

fn main() {
    unsafe {
        // SAFETY: No other work done yet.
        ffi::enable_logging();
    }

    scheduler::with(|| {
        log(cstr!("TOP"));
        let v1 = CownPtr::new(10);
        log(cstr!("Just alloced"));
        let v2 = v1.clone();
        log(cstr!("Just cloned"));
        drop(v1);
        log(cstr!("droped v1"));
        drop(v2);
        log(cstr!("droped v2"));
    });
}
