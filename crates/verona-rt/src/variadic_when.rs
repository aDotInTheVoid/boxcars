use crate::when as mwhen;
use crate::CownPtr;

pub fn when<C>(cowns: C, func: C::Func)
where
    C: CownCollection,
{
    cowns.schedule(func)
}

// TODO: This should be sealed.
pub trait CownCollection {
    type Func;

    fn schedule(self, f: Self::Func);
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
        impl<$($gty),+> CownCollection for ($(&CownPtr<$gty>),+) {
            type Func = mwhen::$fntname < $($gty),+>;

            fn schedule(self, f: Self::Func) {
                let ($($cname),+) = self;
                mwhen::$fnname($($cname),+ , f);
            }
        }
    };
}

impl_collection_once!(when1 Func1 <c1 A>);
impl_collection_once!(when2 Func2 <c1 A, c2 B>);
impl_collection_once!(when3 Func3 <c1 A, c2 B, c3 C>);
impl_collection_once!(when4 Func4 <c1 A, c2 B, c3 C, c4 D>);
impl_collection_once!(when5 Func5 <c1 A, c2 B, c3 C, c4 D, c5 E>);
// TODO: More

#[cfg(test)]
mod tests {
    use crate::with_leak_detector;

    use super::*;

    #[test]
    fn playing_around() {
        with_leak_detector(|| {
            let c1 = CownPtr::new(1);
            let c2 = CownPtr::new(3);
            let c5 = CownPtr::new(8);

            when((&c1, &c2, &c5), |mut a, b, mut c| {
                assert_eq!(*a, 1);
                assert_eq!(*b, 3);
                assert_eq!(*c, 8);

                *a = 10;
                *c = 12;
            });

            when((&c1, &c5), |a, c| {
                assert_eq!(*a, 10);
                assert_eq!(*c, 12);
            })
        });
    }
}
