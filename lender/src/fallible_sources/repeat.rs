use core::fmt;

use crate::prelude::*;

/// Creates a new fallible lender that endlessly repeats a
/// single element.
///
/// The [`FallibleLender`] version of
/// [`iter::repeat()`](core::iter::repeat).
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::fallible_repeat::<
///     '_, fallible_lend!(&'lend u8), String,
/// >(Ok(&0u8));
/// assert_eq!(lender.next().unwrap(), Some(&0));
/// ```
pub fn repeat<'a, L, E>(elt: Result<FallibleLend<'a, L>, E>) -> Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    E: Clone,
    for<'all> FallibleLend<'all, L>: Clone,
{
    Repeat { elt }
}

/// A fallible lender that repeats an element endlessly.
///
/// This `struct` is created by the [`repeat()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
{
    elt: Result<FallibleLend<'a, L>, E>,
}

impl<'a, L, E> Clone for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    E: Clone,
    FallibleLend<'a, L>: Clone,
{
    fn clone(&self) -> Self {
        Repeat {
            elt: self.elt.clone(),
        }
    }
}

impl<'a, L, E> fmt::Debug for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    E: fmt::Debug,
    FallibleLend<'a, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleRepeat")
            .field("elt", &self.elt)
            .finish()
    }
}

impl<'lend, 'a, L, E> FallibleLending<'lend> for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'a, L, E> FallibleLender for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    E: Clone + 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
    type Error = E;
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance_fallible!();

    /// Note: if the stored value is `Err`, the error is cloned
    /// and returned on every call to `next`. This matches the
    /// semantics of [`Repeat`](crate::Repeat) (which yields the
    /// value forever) applied to the fallible case.
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
    fn advance_by(&mut self, n: usize) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        if n > 0 {
            self.elt.clone()?;
        }
        Ok(Ok(()))
    }
}

impl<'a, L, E> DoubleEndedFallibleLender for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    E: Clone + 'a,
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

impl<'a, L, E> FusedFallibleLender for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    E: Clone + 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
}
