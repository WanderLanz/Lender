use core::{fmt, marker};

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending};

/// Creates a lender that yields nothing.
///
/// The [`Lender`] version of [`core::iter::empty()`].
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut e = lender::empty::<lend!(&'lend mut u32)>();
/// let x: Option<&'_ mut u32> = e.next();
/// assert_eq!(x, None);
/// ```
#[inline]
pub const fn empty<L: ?Sized + for<'all> Lending<'all>>() -> Empty<L> {
    Empty(marker::PhantomData)
}

/// A lender that yields nothing.
///
/// This `struct` is created by the [`empty()`] function.
///
/// The [`Lender`] version of [`core::iter::Empty`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
#[derive(Clone, Copy, Default)]
pub struct Empty<L: ?Sized>(marker::PhantomData<L>);

impl<L: ?Sized> fmt::Debug for Empty<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Empty").finish()
    }
}

impl<'lend, L> Lending<'lend> for Empty<L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    type Lend = Lend<'lend, L>;
}

impl<L> Lender for Empty<L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance!();
    #[inline(always)]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl<L> DoubleEndedLender for Empty<L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    #[inline(always)]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

impl<L> ExactSizeLender for Empty<L> where L: ?Sized + for<'all> Lending<'all> {}

impl<L> FusedLender for Empty<L> where L: ?Sized + for<'all> Lending<'all> {}
