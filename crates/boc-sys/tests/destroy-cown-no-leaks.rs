use boc_sys::{
    boxcars_allocate_cown, boxcars_release_object,
    descriptor::{noop_trace, Descriptor},
    enable_logging, schedular_has_leaks, schedular_set_detect_leaks, scheduler_get, scheduler_init,
    scheduler_run,
};

#[test]
fn main() {
    unsafe {
        enable_logging();
        let sched = scheduler_get();
        scheduler_init(sched, 1);
        schedular_set_detect_leaks(true);

        let cown = boxcars_allocate_cown(&Descriptor {
            size: 100,
            trace: noop_trace,
            finaliser: None,
            notified: None,
            destructor: None,
        });

        boxcars_release_object(cown);

        scheduler_run(sched);

        if schedular_has_leaks() {
            panic!("Leaked a cown, but shouldn't have");
        }
    }
}
