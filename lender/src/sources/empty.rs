use core::{fmt, marker};

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender,
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender,
    Lending,
};

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
pub const fn fallible_empty<E, L: ?Sized + for<'all> FallibleLending<'all>>() -> FallibleEmpty<E, L>
{
    FallibleEmpty(marker::PhantomData)
}

/// A fallible lender that yields nothing.
///
/// This `struct` is created by the [`fallible_empty()`] function.
///
/// The [`FallibleLender`] version of [`core::iter::Empty`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
#[derive(Clone, Copy, Default)]
pub struct FallibleEmpty<E, L: ?Sized>(marker::PhantomData<(E, L)>);

impl<E, L: ?Sized> fmt::Debug for FallibleEmpty<E, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleEmpty").finish()
    }
}

impl<'lend, E, L> FallibleLending<'lend> for FallibleEmpty<E, L>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<E, L> FallibleLender for FallibleEmpty<E, L>
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

impl<E, L> DoubleEndedFallibleLender for FallibleEmpty<E, L>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

impl<E, L> ExactSizeFallibleLender for FallibleEmpty<E, L> where
    L: ?Sized + for<'all> FallibleLending<'all>
{
}

impl<E, L> FusedFallibleLender for FallibleEmpty<E, L> where
    L: ?Sized + for<'all> FallibleLending<'all>
{
}
