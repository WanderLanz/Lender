use core::marker::PhantomData;

use crate::{
    higher_order::{FnOnceHKA, FnOnceHKARes},
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender, Lending,
};

/// Creates a lender that lazily generates a value exactly once by invoking
/// the provided closure.
///
/// Note that functions passed to this function must be built using the
/// [`hrc!`](crate::hrc), [`hrc_mut!`](crate::hrc_mut), or
/// [`hrc_once!`](crate::hrc_once) macro, which also checks for covariance of
/// the returned type. Circumventing the macro may result in undefined
/// behavior if the return type is not covariant.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::once_with(0u8, hrc_once!(for<'all> |state: &'all mut u8| -> &'all mut u8 {
///     *state += 1;
///     state
/// }));
/// assert_eq!(lender.next(), Some(&mut 1));
/// assert_eq!(lender.next(), None);
/// ```
pub fn once_with<St, F>(state: St, f: F) -> OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    OnceWith { state, f: Some(f) }
}

/// A lender that yields a single element by applying the provided closure.
///
/// This `struct` is created by the [`once_with()`] function.
///
pub struct OnceWith<St, F> {
    state: St,
    f: Option<F>,
}

impl<'lend, St, F> Lending<'lend> for OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    type Lend = <F as FnOnceHKA<'lend, &'lend mut St>>::B;
}

impl<St, F> Lender for OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.f.take().map(|f| f(&mut self.state))
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

impl<St, F> DoubleEndedLender for OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.next()
    }
}

impl<St, F> ExactSizeLender for OnceWith<St, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    #[inline]
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

impl<St, F> FusedLender for OnceWith<St, F> where F: for<'all> FnOnceHKA<'all, &'all mut St> {}

/// Creates a fallible lender that lazily generates a value exactly once by
/// invoking the provided closure.
///
/// Note that functions passed to this function must be built using the
/// [`hrc!`](crate::hrc), [`hrc_mut!`](crate::hrc_mut), or
/// [`hrc_once!`](crate::hrc_once) macro, which also checks for covariance of
/// the returned type. Circumventing the macro may result in undefined
/// behavior if the return type is not covariant.
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
pub fn fallible_once_with<St, E, F>(state: St, f: F) -> FallibleOnceWith<St, E, F>
where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>,
{
    FallibleOnceWith {
        state,
        f: Some(f),
        _marker: PhantomData,
    }
}

/// A fallible lender that yields a single element by applying the provided closure.
///
/// This `struct` is created by the [`fallible_once_with()`] function.
///
pub struct FallibleOnceWith<St, E, F> {
    state: St,
    f: Option<F>,
    _marker: PhantomData<E>,
}

impl<'lend, St, E, F> FallibleLending<'lend> for FallibleOnceWith<St, E, F>
where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>,
{
    type Lend = <F as FnOnceHKARes<'lend, &'lend mut St, E>>::B;
}

impl<St, E, F> FallibleLender for FallibleOnceWith<St, E, F>
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

impl<St, E, F> DoubleEndedFallibleLender for FallibleOnceWith<St, E, F>
where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.next()
    }
}

impl<St, E, F> FusedFallibleLender for FallibleOnceWith<St, E, F> where
    F: for<'all> FnOnceHKARes<'all, &'all mut St, E>
{
}
