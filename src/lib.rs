#![doc = include_str!("../README.md")]
#![no_std]

use core::{cmp::Ordering, ops::ControlFlow};

extern crate alloc;
use alloc::borrow::ToOwned;

mod adapters;
pub use adapters::*;
mod traits;
pub use traits::*;
pub mod try_trait_v2;
pub(crate) use try_trait_v2::*;
pub mod hkts;
pub(crate) use hkts::*;

mod sealed {
    pub trait Sealed {}
    pub struct Seal<T>(T);
    impl<T> Sealed for Seal<T> {}
}
pub(crate) use sealed::{Seal, Sealed};

#[cfg(test)]
mod test {
    use alloc::{vec, vec::Vec};

    use super::*;

    /// Minimal example of a lender
    struct MyLender<'a, T: 'a>(&'a mut T);
    impl<'lend, 'a, T: 'a> Lending<'lend> for MyLender<'a, T> {
        type Lend = &'lend mut T;
    }
    impl<'a, T: 'a> Lender for MyLender<'a, T> {
        fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { Some(&mut self.0) }
    }

    // BAD EXAMPLE
    // impl<'a> FromLender<&'a u8> for Vec<u8> {
    //     fn from_lender<L>(lender: L) -> Self
    //     where
    //         L: Lender + for<'lend> Lending<'lend, Lend = &'lend u8>,
    //     {
    //         let mut vec = Vec::new();
    //         lender.for_each(|x| vec.push(*x));
    //         vec
    //     }
    // }

    // GOOD EXAMPLE
    // impl<T> FromLender<T> for Vec<u8>
    // where
    //     T: HKT + ToOwned<Owned = u8>,
    // {
    //     fn from_lender<L>(lender: L) -> Self
    //     where
    //         L: Lender + for<'lend> Lending<'lend, Lend = T>,
    //     {
    //         let mut vec = Vec::new();
    //         lender.for_each(|x| vec.push(x.to_owned()));
    //         vec
    //     }
    // }

    // GOOD EXAMPLE
    impl<T, V> FromLender<T> for Vec<V>
    where
        T: HKT + ToOwned<Owned = V>,
    {
        fn from_lender<L>(lender: L) -> Self
        where
            L: Lender + for<'lend> Lending<'lend, Lend = T>,
        {
            let mut vec = Vec::new();
            lender.for_each(|x| vec.push(x.to_owned()));
            vec
        }
    }

    struct WindowsMut<'a, T> {
        inner: &'a mut [T],
        begin: usize,
        len: usize,
    }
    impl<'lend, 'a, T> Lending<'lend> for WindowsMut<'a, T> {
        type Lend = &'lend mut [T];
    }
    impl<'a, T> Lender for WindowsMut<'a, T> {
        fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
            let begin = self.begin;
            self.begin = self.begin.saturating_add(1);
            self.inner.get_mut(begin..begin + self.len)
        }
    }

    fn _windows_mut<'a, T>(slice: &'a mut [T], len: usize) -> WindowsMut<'a, T> {
        WindowsMut { inner: slice, begin: 0, len }
    }

    fn _test<'x>(x: &'x mut u32) {
        let mut bar: MyLender<'x, u32> = MyLender(x);
        let _ = bar.next();
        let _ = bar.next();
        let mut bar = bar.into_lender().mutate(|y| **y += 1).map(|x| *x + 1).into_iterator();
        let x = bar.find_map(|x| if x > 0 { Some(vec![1, 2, 3]) } else { None });
    }
}
