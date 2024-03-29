use verona_rt::with_leak_detector;
use verona_rt::CownPtr;

use std::mem;

// Once one test leaks memory, all other tests will see the leak,
// and also fail.

#[test]
#[should_panic = "leaks detected"]
fn leak_detector_works() {
    with_leak_detector(|| {
        let v = CownPtr::new(666);
        mem::forget(v);
    });
}
