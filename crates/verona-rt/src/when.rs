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

unsafe fn make_aq<'a, T>(aq: &mut ffi::AquiredCown) -> AquiredCown<'a, T> {
    AquiredCown {
        ptr: *aq,
        marker: PhantomData,
    }
}

extern "C" fn trampoline1<T>(aq: &mut ffi::AquiredCown, data: *mut ()) {
    unsafe {
        let func = mem::transmute::<_, UseFunc1<T>>(data);
        func(make_aq(aq));
    }
}
extern "C" fn trampoline2<T, U>(
    a1: &mut ffi::AquiredCown,
    a2: &mut ffi::AquiredCown,
    data: *mut (),
) {
    unsafe {
        let func: UseFunc2<T, U> = mem::transmute(data);
        func(make_aq(a1), make_aq(a2));
    }
}

type UseFunc1<T> = for<'a> fn(AquiredCown<'a, T>);
type UseFunc2<T, U> = for<'a, 'b> fn(AquiredCown<'a, T>, AquiredCown<'b, U>);

pub fn when<T>(cown: &CownPtr<T>, f: UseFunc1<T>) {
    let trampoline = trampoline1::<T>;

    unsafe {
        ffi::boxcar_when1(&cown.ptr, trampoline, f as _);
    }
}

pub fn when2<T, U>(c1: &CownPtr<T>, c2: &CownPtr<U>, f: UseFunc2<T, U>) {
    // So we don't let the func aquire the same cown twice.
    assert_ne!(c1.ptr.addr(), c2.ptr.addr());

    let trampoline = trampoline2::<T, U>;
    unsafe {
        ffi::boxcar_when2(&c1.ptr, &c2.ptr, trampoline, f as _);
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
}
