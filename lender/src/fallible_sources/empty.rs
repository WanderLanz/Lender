use core::{fmt, marker};

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender,
};

/// Creates a fallible lender that yields nothing.
///
/// The [`FallibleLender`] version of [`core::iter::empty()`].
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut e = lender::fallible_empty::<fallible_lend!(&'lend u32), String>();
/// let x: Result<Option<&u32>, String> = e.next();
/// assert_eq!(x, Ok(None));
/// ```
#[inline]
pub const fn empty<L: ?Sized + for<'all> FallibleLending<'all>, E>() -> Empty<L, E> {
    Empty(marker::PhantomData)
}

/// A fallible lender that yields nothing.
///
/// This `struct` is created by the [`fallible_empty()`] function.
///
/// The [`FallibleLender`] version of [`core::iter::Empty`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
#[derive(Clone, Copy, Default)]
pub struct Empty<L: ?Sized, E>(marker::PhantomData<(E, L)>);

impl<L: ?Sized, E> fmt::Debug for Empty<L, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleEmpty").finish()
    }
}

impl<'lend, L, E> FallibleLending<'lend> for Empty<L, E>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L, E> FallibleLender for Empty<L, E>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    type Error = E;
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance_fallible!();

    #[inline(always)]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl<L, E> DoubleEndedFallibleLender for Empty<L, E>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

impl<L, E> ExactSizeFallibleLender for Empty<L, E> where L: ?Sized + for<'all> FallibleLending<'all> {}

impl<L, E> FusedFallibleLender for Empty<L, E> where L: ?Sized + for<'all> FallibleLending<'all> {}
