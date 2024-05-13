use core::{fmt, marker::PhantomData, mem, ops, slice};
use std::ops::Deref;

use verona_rt_sys as ffi;

use crate::cown::CownPtr;

pub struct AcquiredCown<'a, T> {
    // TODO: As an optimization, point to the `T`, and roll the pointer back to
    // find the cown, (instead of pointing to cown, and going forward to T).
    ptr: ffi::CownPtr,
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

unsafe fn make_aq<'a, T>(aq: ffi::CownPtr) -> AcquiredCown<'a, T> {
    AcquiredCown {
        ptr: aq,
        marker: PhantomData,
    }
}

macro_rules! one_trampoline {
    (
        $whenfunc:ident
        $ffiname:ident
        $usefunc:ident
        $tramp_name:ident
        <
        $($gty:ident $glife:lifetime $cname:ident $idx:literal),+
        >
    ) => {
        type $usefunc<$($gty),+> = for <$($glife),+> fn($(AcquiredCown<$glife, $gty>),+);


        extern "C" fn $tramp_name<$($gty),+>(len: usize, cowns: *mut ffi::CownPtr, data: *mut ()) {
            unsafe {
                let s = slice::from_raw_parts(cowns, len);
                let func: $usefunc<$($gty),+> = mem::transmute(data);
                func($(make_aq(s[$idx])),+)
            }
        }

        pub fn $whenfunc<$($gty),+>($($cname: &CownPtr<$gty>),+, f: $usefunc<$($gty),+>) {
            let trampoline = $tramp_name::<$($gty),+>;
            let mut cs = [$($cname.cown_ptr),+];
            assert!(is_unique(&cs), "Cowns not unique");
            unsafe {
                ffi::$ffiname(cs.len(), cs.as_mut_ptr(), trampoline, f as _);
            }

        }
    };
}

one_trampoline!(when boxcars_sched_1 Func1 t1 <A 'a cown 0>);
one_trampoline!(when2 boxcars_sched_2 Func2 t2 <A 'a cown1 0, B 'b cown2 1>);

// one_trampoline!(when  UseFunc1 trampoline1 < A 'a                   > 0);
// one_trampoline!(when2 UseFunc2 trampoline2 < A 'a, B 'b             > 0, 1);
// one_trampoline!(when3 UseFunc3 trampoline3 < A 'a, B 'b, C 'c       > 0, 1, 2);
// one_trampoline!(when4 UseFunc4 trampoline4 < A 'a, B 'b, C 'c, D 'd > 0, 1, 2, 3);

// FIXME: LLVM eat's shit on this codegen. https://godbolt.org/z/s9sqGqGbP
fn is_unique<const N: usize>(cown: &[ffi::CownPtr; N]) -> bool {
    let mut addrs = cown.map(|c| c.addr());
    addrs.sort_unstable();
    !addrs.windows(2).any(|w| w[0] == w[1])
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicU8, Ordering},
            Arc, Barrier, Mutex,
        },
        thread,
    };

    use crate::{scheduler, with_leak_detector};

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
    fn when2_runs() {
        static RUN_COUNTER: AtomicU8 = AtomicU8::new(0);

        assert_eq!(RUN_COUNTER.load(Ordering::SeqCst), 0);

        scheduler::with(|| {
            let v1 = CownPtr::new(1);
            let v2 = CownPtr::new(2);
            when2(&v1, &v2, |a1, a2| {
                assert_eq!(*a1, 1);
                assert_eq!(*a2, 2);
                RUN_COUNTER.fetch_add(1, Ordering::SeqCst);
            })
        });

        assert_eq!(RUN_COUNTER.load(Ordering::SeqCst), 1);
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

    struct SetOnDrop(Arc<Mutex<bool>>);
    impl std::ops::Drop for SetOnDrop {
        fn drop(&mut self) {
            if std::thread::panicking() {
                return;
            }

            let mut is_droped = self.0.lock().unwrap();
            assert_eq!(*is_droped, false);
            *is_droped = true;
        }
    }
    impl SetOnDrop {
        fn new() -> (Self, Arc<Mutex<bool>>) {
            let state: Arc<Mutex<bool>> = Arc::default();
            (Self(state.clone()), state)
        }
    }

    #[test]
    fn when_retains_cowns() {
        let bars = Arc::new((Barrier::new(2), Barrier::new(2), Barrier::new(2)));
        let (droptrack, dropstate) = SetOnDrop::new();
        let bars_ = Arc::clone(&bars);

        let in_sched = move || {
            let c_main = CownPtr::new(droptrack);

            let c_bars = CownPtr::new(bars_);

            // t0
            when2(&c_main, &c_bars, |_, bars| {
                bars.0.wait();
            });
            // t1
            when2(&c_main, &c_bars, |_, bars| {
                bars.1.wait();
            });
            // t2
            when(&c_bars, |bars| {
                bars.2.wait();
            });
        };

        let jh = thread::spawn(|| with_leak_detector(in_sched));

        bars.0.wait();
        // t0 may be done, but may not be, but t1 still hold a reference.
        assert_eq!(*dropstate.lock().unwrap(), false);
        bars.1.wait();
        // At some point here, we start running dtor for droptrack.
        bars.2.wait();
        // but it's definatly done here, as we've run t2, which must be after t1, as they
        // both aquire c_bars.
        assert_eq!(*dropstate.lock().unwrap(), true);

        jh.join().unwrap();
    }

    #[test]
    fn when_retains_cown_two() {
        let (droptrack, dropstate) = SetOnDrop::new();
        let bars = Arc::new((
            Barrier::new(2),
            Barrier::new(2),
            Barrier::new(2),
            Barrier::new(2),
        ));
        let bars_ = Arc::clone(&bars);
        let is_droped = move || *dropstate.lock().unwrap();

        let in_sched = || {
            let c_main = CownPtr::new(droptrack);

            let c_bars = CownPtr::new(bars_);

            // t0
            when2(&c_main, &c_bars, |m, bars| {
                assert_eq!(*m.0.lock().unwrap(), false);
                bars.0.wait();
            });
            // t1
            when(&c_bars, |bars| {
                bars.1.wait();
            });
            // t2
            when2(&c_main, &c_bars, |m, bars| {
                assert_eq!(*m.0.lock().unwrap(), false);
                bars.2.wait();
            });
            // t3
            when(&c_bars, |bars| {
                bars.3.wait();
            });
        };
        let jh = thread::spawn(|| scheduler::with_leak_detector(in_sched));

        assert_eq!(is_droped(), false);
        bars.0.wait();
        assert_eq!(is_droped(), false);
        bars.1.wait();
        assert_eq!(is_droped(), false);
        bars.2.wait();
        // t2 is over here, but we've not started t3, so unknown if drop has run
        bars.3.wait();
        assert_eq!(is_droped(), true, "cown should still be alive here");

        jh.join().unwrap();
    }
}
