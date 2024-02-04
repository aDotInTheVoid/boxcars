use core::{fmt, marker::PhantomData, mem, ops};
use std::ops::Deref;

use verona_rt_sys as ffi;

use crate::cown::CownPtr;

pub struct AcquiredCown<'a, T> {
    // TODO: As an optimization, point to the `T`, and roll the pointer back to
    // find the cown, (instead of pointing to cown, and going forward to T).
    ptr: ffi::AcquiredCown,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T> AcquiredCown<'a, T> {
    fn data_ptr(&self) -> *mut T {
        super::cown::cown_to_data(self.ptr.addr())
    }
}

impl<'a, T> ops::Deref for AcquiredCown<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data_ptr() }
    }
}

impl<'a, T> ops::DerefMut for AcquiredCown<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data_ptr() }
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for AcquiredCown<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}
impl<'a, T: fmt::Display> fmt::Display for AcquiredCown<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.deref(), f)
    }
}

unsafe fn make_aq<'a, T>(aq: &mut ffi::AcquiredCown) -> AcquiredCown<'a, T> {
    AcquiredCown {
        ptr: *aq,
        marker: PhantomData,
    }
}

extern "C" fn trampoline1<T>(aq: &mut ffi::AcquiredCown, data: *mut ()) {
    unsafe {
        let func = mem::transmute::<_, UseFunc1<T>>(data);
        func(make_aq(aq));
    }
}
extern "C" fn trampoline2<T, U>(
    a1: &mut ffi::AcquiredCown,
    a2: &mut ffi::AcquiredCown,
    data: *mut (),
) {
    unsafe {
        let func: UseFunc2<T, U> = mem::transmute(data);
        func(make_aq(a1), make_aq(a2));
    }
}

type UseFunc1<T> = for<'a> fn(AcquiredCown<'a, T>);
type UseFunc2<T, U> = for<'a, 'b> fn(AcquiredCown<'a, T>, AcquiredCown<'b, U>);

pub fn when<T>(cown: &CownPtr<T>, f: UseFunc1<T>) {
    let trampoline = trampoline1::<T>;

    unsafe {
        ffi::boxcar_when1(&cown.cown_ptr, trampoline, f as _);
    }
}

pub fn when2<T, U>(c1: &CownPtr<T>, c2: &CownPtr<U>, f: UseFunc2<T, U>) {
    // So we don't let the func acquire the same cown twice.
    // See also: https://github.com/microsoft/verona-rt/pull/30
    assert_ne!(
        c1.cown_ptr.addr(),
        c2.cown_ptr.addr(),
        "used the same cown twice"
    );

    let trampoline = trampoline2::<T, U>;
    unsafe {
        ffi::boxcar_when2(&c1.cown_ptr, &c2.cown_ptr, trampoline, f as _);
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

    #[test]
    fn on_vec() {
        static RUN_COUNTER: AtomicU8 = AtomicU8::new(0);
        fn incr() {
            RUN_COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        assert_eq!(RUN_COUNTER.load(Ordering::SeqCst), 0);

        scheduler::with(|| {
            let vec_cown = CownPtr::new(vec![1, 2, 3]);

            when(&vec_cown, |mut v| {
                assert_eq!(*v, &[1, 2, 3]);
                v.push(4);
                incr();
            });

            when(&vec_cown, |mut v| {
                assert_eq!(*v, &[1, 2, 3, 4]);
                assert_eq!(RUN_COUNTER.load(Ordering::SeqCst), 1);
                assert_eq!(v.pop(), Some(4));
                incr();
            });

            when(&vec_cown, |v| {
                assert_eq!(*v, &[1, 2, 3]);
                assert_eq!(RUN_COUNTER.load(Ordering::SeqCst), 2);
                incr();
            });
        });

        assert_eq!(RUN_COUNTER.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn when_two() {
        scheduler::with(|| {
            let string = CownPtr::new(String::new());
            let vec = CownPtr::new(Vec::new());

            when(&string, |mut s| {
                assert_eq!(&*s, "");
                s.push_str("foo");
            });
            when(&vec, |mut v| {
                assert_eq!(&*v, &[]);
                v.push(101);
            });
            when2(&string, &vec, |mut s, mut v| {
                assert_eq!(&*s, "foo");
                assert_eq!(&*v, &[101]);
                s.push_str("bar");
                v.push(666);
            });
            when(&string, |s| assert_eq!(&*s, "foobar"));
            when(&vec, |v| assert_eq!(&*v, &[101, 666]));
        })
    }

    #[test]
    #[should_panic = ""]
    #[ignore = "Panics with schedular lock don't work, see #16"]
    fn double_acquire() {
        scheduler::with(|| {
            let c1 = CownPtr::new(10);
            let c2 = c1.clone();
            when2(&c1, &c2, |_, _| loop {});
        })
    }

    #[test]
    fn fmt_acquired() {
        scheduler::with(|| {
            let x = CownPtr::new("101");
            when(&x, |x| {
                assert_eq!(*x, "101");
                assert_eq!(format!("{x}"), "101");
                assert_eq!(format!("{x:?}"), r#""101""#);
            })
        })
    }
}
