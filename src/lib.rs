#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

#[macro_use]
extern crate higher_order_closure;

mod adapters;
pub use adapters::*;
mod traits;
pub use traits::*;
pub mod hkts;
mod impls;
pub mod try_trait_v2;

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
    #[derive(Debug)]
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

    // WORKING BUT BAD EXAMPLE
    // impl<'a> FromLender<&'a mut [u8]> for Vec<u8> {
    //     fn from_lender<L>(lender: L) -> Self
    //     where
    //         L: Lender + for<'lend> Lending<'lend, Lend = &'a mut [u8]>,
    //     {
    //         let mut vec = Vec::new();
    //         lender.for_each(|x| vec.extend_from_slice(x));
    //         vec
    //     }
    // }

    // GOOD EXAMPLE
    // impl<T, V> FromLender<T> for Vec<V>
    // where
    //     T: HKT + ToOwned<Owned = V>,
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
        let mut bar = bar.into_lender().mutate(|y| **y += 1).map(|x: &mut u32| *x + 1).iter();
        let _ = bar.find_map(|x| if x > 0 { Some(vec![1, 2, 3]) } else { None });
        let mut w = vec![1u32, 2, 3, 4, 5];
        let windows = _windows_mut(&mut w, 2);
        windows
            .filter(|x| x[0] > 0)
            // .map(|x| &mut x[0]) // This is not possible because of the lifetime
            .for_each(|x| x[0] += 1);
    }
}
