use core::{fmt, marker};

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend,
    FallibleLender, FallibleLending, FusedFallibleLender,
};

/// Creates a fallible lender that yields nothing.
///
/// The [`FallibleLender`] version of [`core::iter::empty()`].
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut e = lender::fallible_empty::<String, fallible_lend!(&'lend u32)>();
/// let x: Result<Option<&u32>, String> = e.next();
/// assert_eq!(x, Ok(None));
/// ```
pub const fn empty<E, L: ?Sized + for<'all> FallibleLending<'all>>() -> Empty<E, L>
{
    Empty(marker::PhantomData)
}

/// A fallible lender that yields nothing.
///
/// This `struct` is created by the [`empty()`] function.
///
/// The [`FallibleLender`] version of [`core::iter::Empty`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
#[derive(Clone, Copy, Default)]
pub struct Empty<E, L: ?Sized>(marker::PhantomData<(E, L)>);

impl<E, L: ?Sized> fmt::Debug for Empty<E, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleEmpty").finish()
    }
}

impl<'lend, E, L> FallibleLending<'lend> for Empty<E, L>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<E, L> FallibleLender for Empty<E, L>
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

impl<E, L> DoubleEndedFallibleLender for Empty<E, L>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

impl<E, L> ExactSizeFallibleLender for Empty<E, L> where
    L: ?Sized + for<'all> FallibleLending<'all>
{
}

impl<E, L> FusedFallibleLender for Empty<E, L> where
    L: ?Sized + for<'all> FallibleLending<'all>
{
}
