use core::fmt;

use crate::{FusedFallibleLender, FusedLender, prelude::*};

/// Creates a new lender that endlessly repeats a single element.
///
/// The [`Lender`] version of [`iter::repeat()`](core::iter::repeat).
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::repeat::<lend!(&'lend u8)>(&0u8);
/// assert_eq!(lender.next(), Some(&0));
/// ```
pub fn repeat<'a, L>(elt: Lend<'a, L>) -> Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    Repeat { elt }
}

/// A lender that repeats an element endlessly.
///
/// This `struct` is created by the [`repeat()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
{
    elt: Lend<'a, L>,
}

impl<'a, L> Clone for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    Lend<'a, L>: Clone,
{
    fn clone(&self) -> Self {
        Repeat {
            elt: self.elt.clone(),
        }
    }
}

impl<'a, L> fmt::Debug for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    Lend<'a, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Repeat").field("elt", &self.elt).finish()
    }
}

impl<'lend, 'a, L> Lending<'lend> for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L> Lender for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend
        Some(unsafe { core::mem::transmute::<Lend<'a, Self>, Lend<'_, Self>>(self.elt.clone()) })
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }

    #[inline]
    fn advance_by(&mut self, _n: usize) -> Result<(), core::num::NonZeroUsize> {
        Ok(())
    }
}

impl<'a, L> DoubleEndedLender for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.next()
    }

    #[inline]
    fn advance_back_by(&mut self, _n: usize) -> Result<(), core::num::NonZeroUsize> {
        Ok(())
    }
}

impl<'a, L> FusedLender for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> Lend<'all, L>: Clone,
{
}

/// Creates a new fallible lender that endlessly repeats a single element.
///
/// The [`FallibleLender`] version of [`iter::repeat()`](core::iter::repeat).
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::fallible_repeat::<'_, String, fallible_lend!(&'lend u8)>(Ok(&0u8));
/// assert_eq!(lender.next().unwrap(), Some(&0));
/// ```
pub fn fallible_repeat<'a, E, L>(elt: Result<FallibleLend<'a, L>, E>) -> FallibleRepeat<'a, E, L>
where
    E: Clone,
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
    FallibleRepeat { elt }
}

/// A fallible lender that repeats an element endlessly.
///
/// This `struct` is created by the [`fallible_repeat()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FallibleRepeat<'a, E, L>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
{
    elt: Result<FallibleLend<'a, L>, E>,
}

impl<'a, E, L> Clone for FallibleRepeat<'a, E, L>
where
    E: Clone,
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    FallibleLend<'a, L>: Clone,
{
    fn clone(&self) -> Self {
        FallibleRepeat {
            elt: self.elt.clone(),
        }
    }
}

impl<'a, E, L> fmt::Debug for FallibleRepeat<'a, E, L>
where
    E: fmt::Debug,
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    FallibleLend<'a, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleRepeat")
            .field("elt", &self.elt)
            .finish()
    }
}

impl<'lend, 'a, E, L> FallibleLending<'lend> for FallibleRepeat<'a, E, L>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'a, E, L> FallibleLender for FallibleRepeat<'a, E, L>
where
    E: Clone + 'a,
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
    type Error = E;
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance_fallible!();

    /// Note: if the stored value is `Err`, the error is cloned and
    /// returned on every call to `next`. This matches the semantics
    /// of [`Repeat`](crate::Repeat) (which yields the value forever)
    /// applied to the fallible case.
    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.elt.clone().map(|value| {
            Some(
                // SAFETY: 'a: 'lend
                unsafe {
                    core::mem::transmute::<FallibleLend<'a, Self>, FallibleLend<'_, Self>>(value)
                },
            )
        })
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }

    #[inline]
    fn advance_by(
        &mut self,
        n: usize,
    ) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        if n > 0 {
            self.elt.clone()?;
        }
        Ok(Ok(()))
    }
}

impl<'a, E, L> DoubleEndedFallibleLender for FallibleRepeat<'a, E, L>
where
    E: Clone + 'a,
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.next()
    }

    #[inline]
    fn advance_back_by(
        &mut self,
        n: usize,
    ) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        if n > 0 {
            self.elt.clone()?;
        }
        Ok(Ok(()))
    }
}

impl<'a, E, L> FusedFallibleLender for FallibleRepeat<'a, E, L>
where
    E: Clone + 'a,
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
}
