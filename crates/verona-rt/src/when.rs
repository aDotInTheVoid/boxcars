use std::{marker::PhantomData, mem, ops};

use verona_rt_sys as ffi;

use crate::cown::{CownData, CownPtr};

pub struct AquiredCown<'a, T> {
    // TODO: As an optimization, point to the `T`, and roll the pointer back to
    // find the cown, (instead of pointing to cown, and going forward to T).
    ptr: ffi::AquiredCown,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T> AquiredCown<'a, T> {
    fn cown_ptr(&self) -> *mut CownData<T> {
        self.ptr.addr() as _
    }
}

impl<'a, T> ops::Deref for AquiredCown<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.cown_ptr()).data }
    }
}

impl<'a, T> ops::DerefMut for AquiredCown<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.cown_ptr()).data }
    }
}

extern "C" fn call_trampoline<'a, T>(aq: &mut ffi::AquiredCown, data: *mut ()) {
    unsafe {
        let func = mem::transmute::<_, UseFunc<T>>(data);
        func(AquiredCown {
            ptr: *aq,
            marker: PhantomData,
        });
    }
}

type UseFunc<T> = for<'a> fn(AquiredCown<'a, T>);

pub fn when<T>(cown: &CownPtr<T>, f: UseFunc<T>) {
    let trampoline = call_trampoline::<T>;

    unsafe {
        ffi::boxcar_when1(&cown.ptr, trampoline, f as _);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU8, Ordering};

    use crate::scheduler;

    use super::*;

    #[test]
    fn basic() {
        static RUN_COUNTER: AtomicU8 = AtomicU8::new(0);
        fn incr() {
            RUN_COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        assert_eq!(RUN_COUNTER.load(Ordering::SeqCst), 0);

        scheduler::with(|| {
            let v = CownPtr::new(101);
            when(&v, |mut v| {
                assert_eq!(*v, 101);
                *v += 1;
                incr();
            });
            when(&v, |v| {
                assert_eq!(*v, 102);
                incr();
            });
        });

        assert_eq!(RUN_COUNTER.load(Ordering::SeqCst), 2);
    }
}
