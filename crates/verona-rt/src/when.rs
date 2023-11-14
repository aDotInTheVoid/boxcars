use std::{marker::PhantomData, mem, ops};

use verona_rt_sys as ffi;

use crate::cown::CownPtr;

extern "C" fn call_trampoline<'a>(aq: &mut ffi::AquiredCown, data: *const ()) {
    unsafe {
        let func = mem::transmute::<_, IntFn>(data);
        func(AquiredCown {
            ptr: *aq,
            marker: PhantomData,
        });
    }
}

pub struct AquiredCown<'a> {
    ptr: ffi::AquiredCown,
    marker: PhantomData<&'a mut i32>,
}

impl ops::Deref for AquiredCown<'_> {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ffi::cown_get_ref(&self.ptr) as _ }
    }
}
impl ops::DerefMut for AquiredCown<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *ffi::cown_get_ref(&self.ptr) as _ }
    }
}

pub type IntFn = for<'a> fn(AquiredCown<'a>);

pub fn when(cown: &CownPtr, f: IntFn) {
    unsafe { ffi::cown_int_when1(&cown.0, call_trampoline, f as _) };
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
