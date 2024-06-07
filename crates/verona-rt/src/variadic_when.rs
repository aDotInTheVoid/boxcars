use crate::{AcquiredCown, Cown};

pub trait CownCollection {
    type Acquired<'a>;

    fn schedule_onto<F>(self, func: F)
    where
        F: for<'a> FnOnce(Self::Acquired<'a>) + Send + 'static;
}

pub fn when<C, F>(cowns: C, func: F)
where
    C: CownCollection,
    F: for<'a> FnOnce(C::Acquired<'a>) + Send + 'static,
{
    cowns.schedule_onto(func)
}

macro_rules! impl_collection_once {
    (
        $fnname:ident
        $fntname:ident
        <
        $($cname:ident $gty:ident),+
        >
    ) => {
        #[allow(unused_parens)]
        impl<$($gty: 'static),+> CownCollection for ($(&Cown<$gty>),+) {
            type Acquired<'a> = ($(AcquiredCown<'a, $gty>),+);

            fn schedule_onto<Func>(self, func: Func)
            where
                Func: for<'a> FnOnce(Self::Acquired<'a>) + Send + 'static,
            {
                use crate::when as mwhen;

                let ($($cname),+) = self;
                mwhen::$fnname($($cname),+ , |$($cname),+| func(($($cname),+)));
            }
        }
    };
}

impl_collection_once!(when1 Func1 <c1 A>);
impl_collection_once!(when2 Func2 <c1 A, c2 B>);
impl_collection_once!(when3 Func3 <c1 A, c2 B, c3 C>);
impl_collection_once!(when4 Func4 <c1 A, c2 B, c3 C, c4 D>);
impl_collection_once!(when5 Func5 <c1 A, c2 B, c3 C, c4 D, c5 E>);
impl_collection_once!(when6 Func6 <c1 A, c2 B, c3 C, c4 D, c5 E, c6 F>);
impl_collection_once!(when7 Func7 <c1 A, c2 B, c3 C, c4 D, c5 E, c6 F, c7 G>);
impl_collection_once!(when8 Func8 <c1 A, c2 B, c3 C, c4 D, c5 E, c6 F, c7 G, c8 H>);
impl_collection_once!(when9 Func9 <c1 A, c2 B, c3 C, c4 D, c5 E, c6 F, c7 G, c8 H, g9 I>);

// TODO: More?

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicI16, Ordering},
        Arc,
    };

    use crate::with_leak_detector;

    use super::*;

    #[test]
    fn playing_around() {
        with_leak_detector(|| {
            let c1 = Cown::new(1);
            let c2 = Cown::new(3);
            let c5 = Cown::new(8);

            when((&c1, &c2, &c5), |(mut a, b, mut c)| {
                assert_eq!(*a, 1);
                assert_eq!(*b, 3);
                assert_eq!(*c, 8);

                *a = 10;
                *c = 12;
            });

            when((&c1, &c5), |(a, c)| {
                assert_eq!(*a, 10);
                assert_eq!(*c, 12);
            })
        });
    }

    #[test]
    fn lambda() {
        with_leak_detector(|| {
            let x = Arc::new(AtomicI16::new(10));
            let c1 = Cown::new(10);
            let c2 = Cown::new(20);
            let c3 = Cown::new(30);

            let x_ = x.clone();
            when((&c1, &c2), move |(a1, a2)| {
                assert_eq!(*a1, 10);
                assert_eq!(*a2, 20);

                assert_eq!(x_.swap(100, Ordering::Relaxed), 10);
            });

            let x_ = x.clone();
            when((&c2, &c3), move |(a2, a3)| {
                assert_eq!(*a2, 20);
                assert_eq!(*a3, 30);

                assert_eq!(x_.swap(1000, Ordering::Relaxed), 100);
            });

            let x_ = x.clone();
            when((&c3, &c1), move |(a3, a1)| {
                assert_eq!(*a3, 30);
                assert_eq!(*a1, 10);

                assert_eq!(x_.swap(10000, Ordering::Relaxed), 1000);
            });

            when((&c1, &c2, &c3), move |(a1, a2, a3)| {
                assert_eq!(*a1, 10);
                assert_eq!(*a2, 20);
                assert_eq!(*a3, 30);
                assert_eq!(x.load(Ordering::Relaxed), 10000);
            });
        })
    }
}
