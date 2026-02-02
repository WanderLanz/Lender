use core::fmt;

use crate::{
    CovariantFallibleLending, DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend,
    FallibleLender, FallibleLending, FusedFallibleLender,
};

/// Creates a fallible lender that yields an element exactly
/// once.
///
/// The [`FallibleLender`] version of [`core::iter::once()`].
///
/// # Examples
///
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::fallible_once::<fallible_lend!(&'lend u32), String>(
///     Ok(&42),
/// );
/// assert_eq!(lender.next(), Ok(Some(&42)));
/// assert_eq!(lender.next(), Ok(None));
/// ```
#[inline]
pub fn once<'a, L: ?Sized + CovariantFallibleLending, E>(
    value: Result<FallibleLend<'a, L>, E>,
) -> Once<'a, L, E> {
    Once { inner: Some(value) }
}

/// A fallible lender that yields an element exactly once.
///
/// This `struct` is created by the [`once()`] function.
///
/// The [`FallibleLender`] version of [`core::iter::Once`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Once<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    inner: Option<Result<FallibleLend<'a, L>, E>>,
}

impl<'a, L, E> Clone for Once<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending,
    E: Clone,
    FallibleLend<'a, L>: Clone,
{
    fn clone(&self) -> Self {
        Once {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, L, E> fmt::Debug for Once<'a, L, E>
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

impl<'lend, L, E> FallibleLending<'lend> for Once<'_, L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'a, L, E> FallibleLender for Once<'a, L, E>
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

impl<'a, L, E> DoubleEndedFallibleLender for Once<'a, L, E>
where
    E: 'a,
    L: ?Sized + CovariantFallibleLending,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.next()
    }
}

impl<'a, L, E> ExactSizeFallibleLender for Once<'a, L, E>
where
    E: 'a,
    L: ?Sized + CovariantFallibleLending,
{
}

impl<'a, L, E> FusedFallibleLender for Once<'a, L, E>
where
    E: 'a,
    L: ?Sized + CovariantFallibleLending,
{
}
