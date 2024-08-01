use boc_sys::{
    boxcars_allocate_cown,
    descriptor::{noop_trace, Descriptor},
    enable_logging, schedular_has_leaks, schedular_set_detect_leaks, scheduler_get, scheduler_init,
    scheduler_run,
};

#[test]
#[cfg_attr(
    feature = "__any_sanitizer",
    ignore = "leak detector doesn't work under ASAN"
)]
fn main() {
    unsafe {
        enable_logging();
        let sched = scheduler_get();
        scheduler_init(sched, 1);
        schedular_set_detect_leaks(true);

        boxcars_allocate_cown(&Descriptor {
            size: 100,
            trace: noop_trace,
            finaliser: None,
            notified: None,
            destructor: None,
        });

        scheduler_run(sched);

        if !schedular_has_leaks() {
            panic!("expected to leak a cown, but didn't");
        }
    }
}
