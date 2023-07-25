//! # Lender
//!
//! Commonly known as a `LendingIterator` (although formerly referred to as a `StreamingIterator`, which now referrs to async API instead),
//! a [`Lender`] is an [`Iterator`] over items that live at least as long as the call to [`Iterator::next()`]
//! or as long as the iterator itself. In other words, a lender is an iterator that lends an item one at a time.
//!
//! You might be wondering why you would want to use a `Lender` instead of an `Iterator`. The answer is never. If you can use an `Iterator`, you should.
//!
//! However, when you do find yourself in a situation where you want the ease of use of an `Iterator` but you can't use one, `Lender` is here to help.
//!
//! For example, consider the `WindowsMut` problem:
//!
//! If you try to implement a `WindowsMut` iterator over a slice, you will soon find that the borrow checker is smart enough to know that
//! you are attempting to mutably borrow some region of a slice multiple times, and will not allow it.
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
//! [`Lender`] allows you to use many of the methods and convenient APIs of [`Iterator`] without the same restrictions on lending.
//!
//! The caveat is that closures used on lenders and consumers of lenders have to meet stricter requirements.
//! For example, when consuming a lender with an early return, you must use polonius-emulating unsafe code in order to convince the borrow checker that the early return is safe.
//!
//! ## Example
//!
//! ```rust
//! use lender::prelude::*;
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
//! // we can manually iterate over our `WindowsMut`:
//! {
//!     let mut vec = vec![1u32, 2, 3, 4, 5];
//!     let mut windows = WindowsMut { inner: &mut vec, begin: 0, len: 3 };
//!
//!     assert_eq!(windows.next(), Some(&mut [1, 2, 3][..]));
//!     assert_eq!(windows.next(), Some(&mut [2, 3, 4][..]));
//!     assert_eq!(windows.next(), Some(&mut [3, 4, 5][..]));
//!     assert_eq!(windows.next(), None);
//! }
//! // we can use `for_each` or a while loop to iterate over our `WindowsMut`:
//! {
//!     let mut vec = vec![1u32, 2, 3, 4, 5];
//!     let mut vec2 = vec![1u32, 2, 3, 4, 5];
//!     let mut windows = WindowsMut { inner: &mut vec, begin: 0, len: 3 };
//!     let mut windows2 = WindowsMut { inner: &mut vec2, begin: 0, len: 3 };
//!
//!     windows.for_each(|x| x[2] = x[0]);
//!     // or
//!     while let Some(x) = windows2.next() {
//!         x[2] = x[0];
//!     }
//!     assert_eq!(vec, vec2);
//! }
//! // we can use familiar methods like `filter` and `map`:
//! {
//!     let mut vec = vec![1u32, 2, 3, 4, 5];
//!     let mut windows = WindowsMut { inner: &mut vec, begin: 0, len: 3 };
//!
//!     windows.filter(|x| (x[0] % 2) != 0).for_each(|x| x[0] = 0);
//!     assert_eq!(vec, vec![0, 2, 0, 4, 5]);
//! }
//! // we can even turn our `WindowsMut` into an `Iterator` that yields `u32`:
//! {
//!     let mut vec = vec![1u32, 2, 3, 4, 5];
//!     let mut windows = WindowsMut { inner: &mut vec, begin: 0, len: 3 };
//!
//!     let mut iter = windows.map(hrc_mut!(for<'all> |x: &'all mut [u32]| -> u32 { x[0] })).iter();
//!     assert_eq!(iter.next(), Some(1u32));
//! }
//! ```
//!
//! Within this example you might have noticed the use of the [`hrc_mut!`] macro, this a result of those strict requirements on closures mentioned earlier.
//! The Rust borrow checker is not always smart enough to know wether a closure can be used for inputs of 'any lifetime or only for one '1 specific lifetime,
//! and in nightly Rust, there is already a feature for higher-ranked-closures (closure_lifetime_binder) allowing the following:
//! ```ignore
//! #![feature(closure_lifetime_binder)]
//! let _ = for<'all> |x: &'all mut [u32]| -> &'all mut u32 { &mut x[0] };
//! ```
//! However, this feature is not yet stable, and so the [`hrc_once!`], [`hrc_mut!`], and [`hrc!`] macros are provided to emulate this feature for [`FnOnce`], [`FnMut`], and [`Fn`] respectively.
//! ```rust
//! # use lender::prelude::*;
//! let _ = hrc_mut!(for<'all> |x: &'all mut [u32]| -> &'all mut u32 { &mut x[0] });
//! //   ^? impl for<'all> FnMut(&'all mut [u32]) -> &'all mut u32
//! ```
//!
//! ## The Core Issue: Higher-Ranked Trait Bounds
//!
//! The following is essentially just a summary of [Sabrina Jewson's Blog](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats), which I highly recommend reading.
//!
//! Higher-Ranked Trait Bounds (HRTBs) are a part of stable Rust and, at first glance,
//! they might seem to be perfect for LendingIterators:
//!
//! - For a trait with a lifetime generic (`trait Lending<'lt> { type Lend: 'lt; }`)
//! - and HRTB (`...where T: for<'all> Lending<'all>`),
//! - `T` implements `Lending` for *ALL* lifetimes.
//!
//! Although they are certainly the core of what makes Lender work,
//! they prove to be anything but easy to work with.
//! This is mostly because they do not support qualifiers (e.g. `for<'all where 'all: 'lt>`).
//!
//! Fortunately, there are two core solutions to the problems HRTBs present.
//!
//! 1. You can implictly restrict a HRTB by including
//! a reference to the type you want to restrict the lifetime to,
//! because references are treated special.
//!     - For a trait with a lifetime generic (`trait Lending<'lt, T> { type Lend: 'lt; }`)
//!     - and HRTB (`...where T: for<'all /* where Self: 'all */> Lending<'all, &'all Self>`),
//!     - `T` implements `Lending` for all lifetimes *where Self is valid as well*.
//!
//! 2. OR, we can exploit a bug that allows `dyn` objects to implement
//! traits otherwise impossible to implement.
//!     - For a trait with lifetime generic (`trait DynLending<'lt> { type Lend; }`)
//!     - and HRTB (`...where T: ?Sized + for<'all> DynLending<'all>`),
//!     - `T` is not a valid type... under normal circumstances.
//!     - However, dyn object (`dyn for<'all> Buf<'all, Lend = ...>`) IS valid for `T`!
//!
//! The first solution is the one used the most in this crate, and as useful as the second solution is, we try and restrict its use to a minimum because it is considered a bug and may be fixed at any time.
//!
//! This comes at a cost of not being able to have a `dyn` Lender, which is unfortunate, but not a deal breaker.
//!
//! For now, solution 2 (impossible dyn) is only used by the [`lend!`] macro,
//! which is used in conjuction with source function like [`empty()`] to create Lenders
//! without needing to create a unit struct just for typing.

#![no_std]

extern crate alloc;

mod private {
    pub trait Sealed {}
    pub struct Seal<T>(T);
    impl<T> Sealed for Seal<T> {}
}
pub(crate) use private::{Seal, Sealed};

mod adapters;
pub use adapters::*;
mod traits;
pub use traits::*;
pub mod higher_order;
mod sources;
pub use sources::*;
pub mod try_trait_v2;

pub mod prelude {
    pub use crate::{
        hrc, hrc_mut, hrc_once, lend, DoubleEndedLender, ExactSizeLender, ExtendLender, FromLender, IntoLender, Lender,
        Lending,
    };
}
