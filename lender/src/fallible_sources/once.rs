use core::fmt;

use crate::{
    CovariantFallibleLending, DoubleEndedFallibleLender,
    ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender,
};

/// Creates a fallible lender that yields an element exactly
/// once.
///
/// The [`FallibleLender`] version of [`core::iter::once()`].
pub fn fallible_once<'a, E, L: ?Sized + CovariantFallibleLending>(
    value: Result<FallibleLend<'a, L>, E>,
) -> FallibleOnce<'a, E, L> {
    FallibleOnce { inner: Some(value) }
}

/// A fallible lender that yields an element exactly once.
///
/// This `struct` is created by the [`fallible_once()`] function.
///
/// The [`FallibleLender`] version of [`core::iter::Once`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FallibleOnce<'a, E, L>
where
    L: ?Sized + CovariantFallibleLending,
{
    inner: Option<Result<FallibleLend<'a, L>, E>>,
}

impl<'a, E, L> Clone for FallibleOnce<'a, E, L>
where
    L: ?Sized + CovariantFallibleLending,
    E: Clone,
    FallibleLend<'a, L>: Clone,
{
    fn clone(&self) -> Self {
        FallibleOnce {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, E, L> fmt::Debug for FallibleOnce<'a, E, L>
where
    L: ?Sized + CovariantFallibleLending,
    E: fmt::Debug,
    FallibleLend<'a, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleOnce")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'lend, E, L> FallibleLending<'lend> for FallibleOnce<'_, E, L>
where
    L: ?Sized + CovariantFallibleLending,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'a, E, L> FallibleLender for FallibleOnce<'a, E, L>
where
    E: 'a,
    L: ?Sized + CovariantFallibleLending,
{
    type Error = E;
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.inner.take() {
            None => Ok(None),
            Some(inner) => inner.map(|value| {
                Some(
                    // SAFETY: 'a: 'lend
                    unsafe {
                        core::mem::transmute::<FallibleLend<'a, Self>, FallibleLend<'_, Self>>(
                            value,
                        )
                    },
                )
            }),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Reports (1, Some(1)) even if inner holds Err:
        // size_hint counts calls to next() that return non-None,
        // and next() returns Err (not Ok(None)) in that case.
        if self.inner.is_some() {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}

impl<'a, E, L> DoubleEndedFallibleLender for FallibleOnce<'a, E, L>
where
    E: 'a,
    L: ?Sized + CovariantFallibleLending,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.next()
    }
}

impl<'a, E, L> ExactSizeFallibleLender for FallibleOnce<'a, E, L>
where
    E: 'a,
    L: ?Sized + CovariantFallibleLending,
{
}

impl<'a, E, L> FusedFallibleLender for FallibleOnce<'a, E, L>
where
    E: 'a,
    L: ?Sized + CovariantFallibleLending,
{
}
