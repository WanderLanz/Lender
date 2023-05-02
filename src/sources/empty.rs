use core::{fmt, marker};

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending};

/// Creates a lender that yields nothing.
///
/// similar to [`core::iter::empty()`]
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// struct U32RefMut;
/// impl<'lend> Lending<'lend> for U32RefMut {
///    type Lend = &'lend mut u32;
/// }
/// let mut e = lender::empty::<U32RefMut>();
/// assert_eq!(e.next(), None);
/// ```
pub const fn empty<L: for<'all> Lending<'all>>() -> Empty<L> { Empty(marker::PhantomData) }

/// A lender that yields nothing.
///
/// This `struct` is created by the [`empty()`] function. See its documentation for more.
///
/// similar to [`core::iter::Empty`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Empty<L>(marker::PhantomData<L>);

impl<L> fmt::Debug for Empty<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("Empty").finish() }
}

impl<'lend, L> Lending<'lend> for Empty<L>
where
    L: for<'all> Lending<'all>,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L> Lender for Empty<L>
where
    L: for<'all> Lending<'all>,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { None }
    fn size_hint(&self) -> (usize, Option<usize>) { (0, Some(0)) }
}

impl<L> DoubleEndedLender for Empty<L>
where
    L: for<'all> Lending<'all>,
{
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> { None }
}

impl<L> ExactSizeLender for Empty<L>
where
    L: for<'all> Lending<'all>,
{
    fn len(&self) -> usize { 0 }
}

impl<L> FusedLender for Empty<L> where L: for<'all> Lending<'all> {}

impl<L> Clone for Empty<L> {
    fn clone(&self) -> Empty<L> { Empty(marker::PhantomData) }
}

impl<L> Default for Empty<L> {
    fn default() -> Empty<L> { Empty(marker::PhantomData) }
}
