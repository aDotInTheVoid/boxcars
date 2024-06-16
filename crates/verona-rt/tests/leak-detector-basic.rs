use boxcars::with_leak_detector;
use boxcars::Cown;

use std::mem;

// Once one test leaks memory, all other tests will see the leak,
// and also fail.

#[test]
#[should_panic = "leaks detected"]
#[cfg_attr(
    feature = "__any_sanitizer",
    ignore = "leak detector doesn't work under ASAN"
)]
fn leak_detector_works() {
    with_leak_detector(|| {
        let v = Cown::new(666);
        mem::forget(v);
    });
}
