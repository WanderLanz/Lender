//! # Lender
//!
//! A standard `Iterator` cannot iterate over items that can only be borrowed for the lifetime of the call to `next()`.
//! This is because the `Iterator` trait does not define `next()` to have a lifetime parameter that can be used to specify the
//! lifetime of the returned item to be bound to the lifetime of the call to `next()`.
//!
//! For example, if you try to implement a `WindowsMut` iterator over a slice, you will soon find that the borrow checker is smart enough to know that
//! you are attempting to mutably borrow some region of a slice for a lifetime larger than you can return, and will not allow it.
//!
//! ```rust,compile_fail
//! struct WindowsMut<'a, T> {
//!     inner: &'a mut [T],
//!     begin: usize,
//!     len: usize,
//! }
//! impl<'a, T> Iterator for WindowsMut<'a, T> {
//!     type Item = &'a mut [T];
//!
//!     // imagine the { &mut self } here has lifetime { '1 }
//!     fn next(&mut self) -> Option<Self::Item> {
//!         let begin = self.begin;
//!         self.begin = self.begin.saturating_add(1);
//!         // cannot return { &'1 mut [T] } because { '1 } does not live long enough to fulfill the lifetime { 'a }
//!         self.inner.get_mut(begin..begin + self.len)
//!     }
//! }
//! ```
//!
//! `lender` allows you to use many of the methods and convenient APIs of `Iterator` on types that can only be borrowed for the lifetime of the call to `next()`.
//!
//! ## Example
//!
//! ```rust
//! use lender::{Lending, Lender};
//!
//! struct WindowsMut<'a, T> {
//!     inner: &'a mut [T],
//!     begin: usize,
//!     len: usize,
//! }
//!
//! // first, we need to implement the `Lending` and `Lender` traits for our `WindowsMut` type:
//!
//! impl<'lend, 'a, T> Lending<'lend> for WindowsMut<'a, T> {
//!     type Lend = &'lend mut [T];
//! }
//! impl<'a, T> Lender for WindowsMut<'a, T> {
//!     fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
//!         let begin = self.begin;
//!         self.begin = self.begin.saturating_add(1);
//!         self.inner.get_mut(begin..begin + self.len)
//!     }
//! }
//!
//! // Now we can use many of the methods and convenient APIs of `Iterator` on our `WindowsMut` type:
//!
//! fn main() {
//!     // we can manually iterate over our `WindowsMut`:
//!     {
//!         let mut vec = vec![1u32, 2, 3, 4, 5];
//!         let mut windows = WindowsMut { inner: &mut vec, begin: 0, len: 3 };
//!
//!         assert_eq!(windows.next(), Some(&mut [1, 2, 3][..]));
//!         assert_eq!(windows.next(), Some(&mut [2, 3, 4][..]));
//!         assert_eq!(windows.next(), Some(&mut [3, 4, 5][..]));
//!         assert_eq!(windows.next(), None);
//!     }
//!     // we can use `for_each` or a while loop to iterate over our `WindowsMut`:
//!     {
//!         let mut vec = vec![1u32, 2, 3, 4, 5];
//!         let mut vec2 = vec![1u32, 2, 3, 4, 5];
//!         let mut windows = WindowsMut { inner: &mut vec, begin: 0, len: 3 };
//!         let mut windows2 = WindowsMut { inner: &mut vec2, begin: 0, len: 3 };
//!
//!         windows.for_each(|x| x[2] = x[0]);
//!         // or
//!         while let Some(x) = windows2.next() {
//!             x[2] = x[0];
//!         }
//!         assert_eq!(vec, vec2);
//!     }
//!     // we can use familiar methods like `filter` and `map`:
//!     {
//!         let mut vec = vec![1u32, 2, 3, 4, 5];
//!         let mut windows = WindowsMut { inner: &mut vec, begin: 0, len: 3 };
//!
//!         windows.filter(|x| (x[0] % 2) != 0).for_each(|x| x[0] = 0);
//!         assert_eq!(vec, vec![0, 2, 0, 4, 5]);
//!     }
//!     // we can even turn our `WindowsMut` into an `Iterator` that yields `u32`:
//!     {
//!         let mut vec = vec![1u32, 2, 3, 4, 5];
//!         let mut windows = WindowsMut { inner: &mut vec, begin: 0, len: 3 };
//!
//!         let mut iter = windows.map(|x| x[0]).iter();
//!         assert_eq!(iter.next(), Some(1u32));
//!     }
//! }
//! ```
#![no_std]

extern crate alloc;

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
