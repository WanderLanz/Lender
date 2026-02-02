use core::{fmt, marker::PhantomData};

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, higher_order::FnOnceHKARes,
};

/// Creates a fallible lender that lazily generates a value
/// exactly once by invoking the provided closure.
///
/// Note that functions passed to this function must be built
/// using the [`hrc!`](crate::hrc),
/// [`hrc_mut!`](crate::hrc_mut), or
/// [`hrc_once!`](crate::hrc_once) macro, which also checks for
/// covariance of the returned type. Circumventing the macro may
/// result in undefined behavior if the return type is not
/// covariant.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// # use std::io::Error;
/// let mut lender = lender::fallible_once_with::<_, Error, _>(
///     0u8,
///     hrc_once!(for<'all> |state: &'all mut u8| -> Result<&'all mut u8, Error> {
///         *state += 1;
///         Ok(state)
///     })
/// );
/// assert_eq!(lender.next().unwrap(), Some(&mut 1));
/// assert_eq!(lender.next().unwrap(), None);
/// ```
#[inline]
pub fn once_with<St, E, F>(state: St, f: F) -> OnceWith<St, E, F>
where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>,
{
    OnceWith {
        state,
        f: Some(f),
        _marker: PhantomData,
    }
}

/// A fallible lender that yields a single element by applying
/// the provided closure.
///
/// This `struct` is created by the [`once_with()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct OnceWith<St, E, F> {
    state: St,
    f: Option<F>,
    _marker: PhantomData<E>,
}

impl<St: Clone, E, F: Clone> Clone for OnceWith<St, E, F> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<St: fmt::Debug, E, F> fmt::Debug for OnceWith<St, E, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleOnceWith")
            .field("state", &self.state)
            .finish()
    }
}

impl<'lend, St, E, F> FallibleLending<'lend> for OnceWith<St, E, F>
where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>,
{
    type Lend = <F as FnOnceHKARes<'lend, &'lend mut St, E>>::B;
}

impl<St, E, F> FallibleLender for OnceWith<St, E, F>
where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>,
{
    type Error = E;
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.f.take() {
            Some(f) => f(&mut self.state).map(Some),
            None => Ok(None),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.f.is_some() {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}

impl<St, E, F> DoubleEndedFallibleLender for OnceWith<St, E, F>
where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.next()
    }
}

impl<St, E, F> ExactSizeFallibleLender for OnceWith<St, E, F> where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>
{
}

impl<St, E, F> FusedFallibleLender for OnceWith<St, E, F> where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>
{
}
