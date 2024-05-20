use core::{fmt, marker::PhantomData, ops};
use std::ops::Deref;

use verona_rt_sys as ffi;

use crate::{cown::CownPtr, lambdas::Slot};

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

unsafe fn make_aq<'a, T>(aq: &Slot) -> AcquiredCown<'a, T> {
    AcquiredCown {
        ptr: aq.cown,
        marker: PhantomData,
    }
}

macro_rules! one_when {
    (
        $whenfunc:ident
        $usefunc:ident
        $tramp_name:ident
        <
        $($gty:ident $glife:lifetime $cname:ident $idx:literal),+
        >
    ) => {
        pub(crate) type $usefunc<$($gty),+> = for <$($glife),+> fn($(AcquiredCown<$glife, $gty>),+);

        pub fn $whenfunc
            <Func, $($gty : 'static ),+>
        ($($cname: &CownPtr<$gty>),+, func: Func)
            where
                Func: for <$($glife),+> FnOnce( $(AcquiredCown<$glife, $gty>),+)
                    + Send + 'static
         {
            let  cs = [$($cname.cown_ptr),+];
            assert!(is_unique(&cs), "Cowns not unique");
            $crate::schedule_lambda(
                move |s| {
                    unsafe { func($(make_aq(&s[$idx])),+) }
                },
                &cs
            );
        }
    };
}

// TODO: Add when0
// one_when!(when0 boxcars_sched_0 Func0 t0 <>);

one_when!(when1 Func1 t1 <A 'a cown0 0>);
one_when!(when2 Func2 t2 <A 'a cown0 0, B 'b cown1 1>);
one_when!(when3 Func3 t3 <A 'a cown0 0, B 'b cown1 1, C 'c cown2 2>);
one_when!(when4 Func4 t4 <A 'a cown0 0, B 'b cown1 1, C 'c cown2 2, D 'd cown3 3>);
one_when!(when5 Func5 t5 <A 'a cown0 0, B 'b cown1 1, C 'c cown2 2, D 'd cown3 3, E 'e cown4 4>);
one_when!(when6 Func6 t6 <A 'a cown0 0, B 'b cown1 1, C 'c cown2 2, D 'd cown3 3, E 'e cown4 4, F 'f cown5 5>);
one_when!(when7 Func7 t7 <A 'a cown0 0, B 'b cown1 1, C 'c cown2 2, D 'd cown3 3, E 'e cown4 4, F 'f cown5 5, G 'g cown6 6>);
one_when!(when8 Func8 t8 <A 'a cown0 0, B 'b cown1 1, C 'c cown2 2, D 'd cown3 3, E 'e cown4 4, F 'f cown5 5, G 'g cown6 6, H 'h cown7 7>);
one_when!(when9 Func9 t9 <A 'a cown0 0, B 'b cown1 1, C 'c cown2 2, D 'd cown3 3, E 'e cown4 4, F 'f cown5 5, G 'g cown6 6, H 'h cown7 7, I 'i cown8 8>);

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
    use stdx::SetOnDrop;

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
            when1(&v, |mut v| {
                assert_eq!(*v, 101);
                *v += 1;
                incr();
            });
            when1(&v, |v| {
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

            when1(&vec_cown, |mut v| {
                assert_eq!(*v, &[1, 2, 3]);
                v.push(4);
                incr();
            });

            when1(&vec_cown, |mut v| {
                assert_eq!(*v, &[1, 2, 3, 4]);
                assert_eq!(RUN_COUNTER.load(Ordering::SeqCst), 1);
                assert_eq!(v.pop(), Some(4));
                incr();
            });

            when1(&vec_cown, |v| {
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

            when1(&string, |mut s| {
                assert_eq!(&*s, "");
                s.push_str("foo");
            });
            when1(&vec, |mut v| {
                assert_eq!(&*v, &[]);
                v.push(101);
            });
            when2(&string, &vec, |mut s, mut v| {
                assert_eq!(&*s, "foo");
                assert_eq!(&*v, &[101]);
                s.push_str("bar");
                v.push(666);
            });
            when1(&string, |s| assert_eq!(&*s, "foobar"));
            when1(&vec, |v| assert_eq!(&*v, &[101, 666]));
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
            when1(&x, |x| {
                assert_eq!(*x, "101");
                assert_eq!(format!("{x}"), "101");
                assert_eq!(format!("{x:?}"), r#""101""#);
            })
        })
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
            when1(&c_bars, |bars| {
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
            when1(&c_bars, |bars| {
                bars.1.wait();
            });
            // t2
            when2(&c_main, &c_bars, |m, bars| {
                assert_eq!(*m.0.lock().unwrap(), false);
                bars.2.wait();
            });
            // t3
            when1(&c_bars, |bars| {
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

    #[test]
    fn many_airety() {
        with_leak_detector(|| {
            let c0 = CownPtr::new(0);
            let c1 = CownPtr::new(1);
            let c2 = CownPtr::new(2);
            let c3 = CownPtr::new(3);
            let c4 = CownPtr::new(4);
            let c5 = CownPtr::new(5);
            let c6 = CownPtr::new(6);
            let c7 = CownPtr::new(7);
            let c8 = CownPtr::new(8);

            when9(
                &c0,
                &c1,
                &c2,
                &c3,
                &c4,
                &c5,
                &c6,
                &c7,
                &c8,
                |mut a0, mut a1, mut a2, mut a3, mut a4, mut a5, mut a6, mut a7, mut a8| {
                    assert_eq!(*a0, 0);
                    *a0 *= 10;
                    assert_eq!(*a1, 1);
                    *a1 *= 10;
                    assert_eq!(*a2, 2);
                    *a2 *= 10;
                    assert_eq!(*a3, 3);
                    *a3 *= 10;
                    assert_eq!(*a4, 4);
                    *a4 *= 10;
                    assert_eq!(*a5, 5);
                    *a5 *= 10;
                    assert_eq!(*a6, 6);
                    *a6 *= 10;
                    assert_eq!(*a7, 7);
                    *a7 *= 10;
                    assert_eq!(*a8, 8);
                    *a8 *= 10;
                },
            );

            when6(
                &c0,
                &c1,
                &c2,
                &c3,
                &c4,
                &c5,
                |mut a0, mut a1, mut a2, mut a3, mut a4, mut a5| {
                    assert_eq!(*a0, 0);
                    *a0 *= 10;
                    assert_eq!(*a1, 10);
                    *a1 *= 10;
                    assert_eq!(*a2, 20);
                    *a2 *= 10;
                    assert_eq!(*a3, 30);
                    *a3 *= 10;
                    assert_eq!(*a4, 40);
                    *a4 *= 10;
                    assert_eq!(*a5, 50);
                    *a5 *= 10;
                },
            );

            when3(&c0, &c1, &c2, |mut a0, mut a1, mut a2| {
                assert_eq!(*a0, 0);
                *a0 *= 10;
                assert_eq!(*a1, 100);
                *a1 *= 10;
                assert_eq!(*a2, 200);
                *a2 *= 10;
            });

            when9(
                &c0,
                &c1,
                &c2,
                &c3,
                &c4,
                &c5,
                &c6,
                &c7,
                &c8,
                |mut a0, mut a1, mut a2, mut a3, mut a4, mut a5, mut a6, mut a7, mut a8| {
                    assert_eq!(*a0, 0);
                    *a0 *= 10;
                    assert_eq!(*a1, 1000);
                    *a1 *= 10;
                    assert_eq!(*a2, 2000);
                    *a2 *= 10;
                    assert_eq!(*a3, 300);
                    *a3 *= 10;
                    assert_eq!(*a4, 400);
                    *a4 *= 10;
                    assert_eq!(*a5, 500);
                    *a5 *= 10;
                    assert_eq!(*a6, 60);
                    *a6 *= 10;
                    assert_eq!(*a7, 70);
                    *a7 *= 10;
                    assert_eq!(*a8, 80);
                    *a8 *= 10;
                },
            );
        });
    }

    #[test]
    fn lambda_one() {
        with_leak_detector(|| {
            let shared = Arc::new(Mutex::new(10));
            let shared2 = Arc::clone(&shared);

            let cown = CownPtr::new(1);

            when1(&cown, move |mut s| {
                assert_eq!(*s, 1);
                let mut shared = shared.lock().unwrap();
                assert_eq!(*shared, 10);
                *shared = 20;
                *s = 2;
            });

            when1(&cown, move |s| {
                assert_eq!(*s, 2);
                let shared = shared2.lock().unwrap();
                assert_eq!(*shared, 20);
            });
        })
    }
}
