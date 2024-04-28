use verona_rt_sys::{
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

        scheduler_run(sched);

        if schedular_has_leaks() {
            panic!("Has leaks");
        }
    }
}
