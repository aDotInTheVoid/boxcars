use core::{mem, ptr};

use verona_rt_sys as ffi;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct WorkPtr(*mut ());

#[repr(transparent)]
#[derive(Clone, Copy)]
struct BehaviourCorePtr(*mut ());

#[link(name = "boxcar_bindings")]
extern "C" {
    /*
    size_t n_cowns,
    Cown** cowns,
    void (*f)(Work*),
    size_t payload_size,
    void* payload
     */

    fn boxcars_sched_lambda(
        n_cowns: usize,
        cowns: *const ffi::CownPtr,
        f: extern "C" fn(WorkPtr),
        payload_size: usize,
        payload: *const (),
    );

    fn boxcars_behaviourcore_from_work(w: WorkPtr) -> BehaviourCorePtr;
    fn boxcars_behaviourcore_get_body(w: BehaviourCorePtr) -> *mut ();
    fn boxcars_behaviourcore_release_all(b: BehaviourCorePtr);
    fn boxcars_work_dealloc(w: WorkPtr);
}

pub fn schedule_lambda<F>(func: F)
where
    // TODO: Is this the right bound?
    F: FnOnce() + Send + 'static,
{
    // TODO: Use inline-const here.
    // const {
    assert!(mem::align_of::<F>() <= mem::align_of::<*mut ()>());
    // }

    let func_nodrop = mem::ManuallyDrop::new(func);

    let s: &[ffi::CownPtr] = &[];

    let invoke = invoke_trampoline::<F>;

    unsafe {
        boxcars_sched_lambda(
            0,
            s.as_ptr(),
            invoke,
            mem::size_of::<F>(),
            &func_nodrop as *const _ as _,
        )
    }
}

extern "C" fn invoke_trampoline<F>(w: WorkPtr)
where
    F: FnOnce() + Send + 'static,
{
    unsafe {
        let be = boxcars_behaviourcore_from_work(w);

        let body = boxcars_behaviourcore_get_body(be);

        let func: F = ptr::read(body as *const F);
        func();

        boxcars_behaviourcore_release_all(be);
        boxcars_work_dealloc(w);
    }
}

#[cfg(test)]
mod tests {
    use stdx::SetOnDrop;

    use super::*;
    use crate::with_leak_detector;

    use std::sync::{Arc, Mutex};

    #[test]
    fn simple() {
        let shared_state = Arc::new(Mutex::new(200));

        let x = 10;

        with_leak_detector(|| {
            let shared_state = Arc::clone(&shared_state);

            schedule_lambda(move || {
                assert_eq!(x, 10);

                let mut state = shared_state.lock().unwrap();
                assert_eq!(*state, 200);
                *state = 300;
            });
        });

        let state = shared_state.lock().unwrap();
        assert_eq!(*state, 300);
    }

    #[test]
    fn drops() {
        let (dropset, check) = SetOnDrop::new();
        with_leak_detector(|| {
            schedule_lambda(|| {
                drop(dropset);
            });
            assert_eq!(*check.lock().unwrap(), false);
        });
        assert_eq!(*check.lock().unwrap(), true);
    }
}
