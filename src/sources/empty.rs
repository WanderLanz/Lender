use core::{fmt, marker};

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};

/// Creates an lender that yields nothing.
///
/// lender equivalent to [`core::iter::empty()`].
///
/// # Examples
/// ```rust
/// use lender::{Lender, empty};
/// let e = empty::<&mut i32>();
/// e.for_each(|x| drop(x));
/// ```
pub const fn empty<T>() -> Empty<T> { Empty(marker::PhantomData) }

/// An lender that yields nothing.
///
/// This `struct` is created by the [`empty()`] function. See its documentation for more.
///
/// lender equivalent to [`core::iter::Empty`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Empty<T>(marker::PhantomData<fn() -> T>);

impl<T> fmt::Debug for Empty<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("Empty").finish() }
}

impl<'lend, T> Lending<'lend> for Empty<T> {
    type Lend = T;
}
impl<T> Lender for Empty<T> {
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { None }
    fn size_hint(&self) -> (usize, Option<usize>) { (0, Some(0)) }
}

impl<T> DoubleEndedLender for Empty<T> {
    fn next_back(&mut self) -> Option<T> { None }
}

impl<T> ExactSizeLender for Empty<T> {
    fn len(&self) -> usize { 0 }
}

impl<T> FusedLender for Empty<T> {}

impl<T> Clone for Empty<T> {
    fn clone(&self) -> Empty<T> { Empty(marker::PhantomData) }
}

impl<T> Default for Empty<T> {
    fn default() -> Empty<T> { Empty(marker::PhantomData) }
}
