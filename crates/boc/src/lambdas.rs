use core::{mem, ptr, slice};

use boc_sys as ffi;

pub(crate) fn schedule_lambda<F>(func: F, cowns: &[ffi::CownPtr])
where
    // TODO: Is this the right bound?
    F: FnOnce(&[ffi::Slot]) + Send + 'static,
{
    const {
        assert!(mem::align_of::<F>() <= mem::align_of::<*mut ()>());
    }

    let func_nodrop = mem::ManuallyDrop::new(func);

    let invoke = invoke_trampoline::<F>;

    unsafe {
        ffi::boxcars_sched_lambda(
            invoke,
            cowns.as_ptr(),
            cowns.len(),
            &func_nodrop as *const _ as _,
            mem::size_of::<F>(),
        )
    }
}

pub fn when0<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    schedule_lambda(|_| f(), &[]);
}

extern "C" fn invoke_trampoline<F>(work: ffi::WorkPtr)
where
    F: FnOnce(&[ffi::Slot]) + Send + 'static,
{
    unsafe {
        let mut slots = ptr::null();
        let mut body = ptr::null();
        let mut count = 0;

        ffi::boxcars_preinvoke(work, &mut slots, &mut body, &mut count);

        let slots = slice::from_raw_parts(slots, count);

        let func: F = ptr::read(body as *const F);
        func(slots);

        ffi::boxcars_postinvoke(work);
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

            when0(move || {
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
            when0(|| {
                drop(dropset);
            });
            assert_eq!(*check.lock().unwrap(), false);
        });
        assert_eq!(*check.lock().unwrap(), true);
    }
}
