use core::{fmt, marker::PhantomData};

use crate::{
    Covar, CovariantFallibleLending, DoubleEndedFallibleLender, ExactSizeFallibleLender,
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, higher_order::FnOnceHKA,
};

/// Creates a fallible lender that lazily generates a value
/// exactly once by invoking the provided closure.
///
/// This is the [`FallibleLender`] version of
/// [`once_with()`](crate::once_with): the closure returns a
/// value directly (not a `Result`).
///
/// To create a lender that lazily generates a single error,
/// use [`once_with_err()`].
///
/// Note that functions passed to this function must be built
/// using the [`covar!`](crate::covar),
/// [`covar_mut!`](crate::covar_mut), or
/// [`covar_once!`](crate::covar_once) macros, which also checks for
/// covariance of the returned type.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::fallible_once_with::<_, String, _>(
///     0,
///     covar_once!(for<'all> |state: &'all mut i32| -> &'all mut i32 {
///         *state += 1;
///         state
///     })
/// );
/// assert_eq!(lender.next().unwrap(), Some(&mut 1));
/// assert_eq!(lender.next().unwrap(), None);
/// ```
#[inline]
pub fn once_with<St, E, F>(state: St, f: Covar<F>) -> OnceWith<St, E, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    OnceWith {
        state,
        f: Some(f),
        _marker: PhantomData,
    }
}

/// Creates a fallible lender that lazily generates a single
/// error by invoking the provided closure.
///
/// This is the error counterpart to [`once_with()`]: it calls
/// the closure once and yields its return value as an error.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::fallible_once_with_err::<
///     _, fallible_lend!(&'lend i32), _,
/// >(42, |state: &mut i32| format!("error code: {state}"));
/// assert_eq!(lender.next(), Err("error code: 42".to_string()));
/// assert_eq!(lender.next(), Ok(None));
/// ```
#[inline]
pub fn once_with_err<St, L, F>(state: St, f: F) -> OnceWithErr<St, L, F>
where
    L: ?Sized + CovariantFallibleLending,
{
    OnceWithErr {
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
    f: Option<Covar<F>>,
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
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    type Lend = <F as FnOnceHKA<'lend, &'lend mut St>>::B;
}

impl<St, E, F> FallibleLender for OnceWith<St, E, F>
where
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    type Error = E;
    // SAFETY: the lend is the return type of F, whose covariance
    // has been checked at Covar construction time.
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(self.f.take().map(|f| f.into_inner()(&mut self.state)))
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
    F: for<'all> FnOnceHKA<'all, &'all mut St>,
{
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.next()
    }
}

impl<St, E, F> ExactSizeFallibleLender for OnceWith<St, E, F> where
    F: for<'all> FnOnceHKA<'all, &'all mut St>
{
}

impl<St, E, F> FusedFallibleLender for OnceWith<St, E, F> where
    F: for<'all> FnOnceHKA<'all, &'all mut St>
{
}

/// A fallible lender that yields a single error by applying
/// the provided closure.
///
/// This `struct` is created by the [`once_with_err()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct OnceWithErr<St, L: ?Sized, F> {
    state: St,
    f: Option<F>,
    _marker: PhantomData<L>,
}

impl<St: Clone, L: ?Sized, F: Clone> Clone for OnceWithErr<St, L, F> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<St: fmt::Debug, L: ?Sized, F> fmt::Debug for OnceWithErr<St, L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleOnceWithErr")
            .field("state", &self.state)
            .finish()
    }
}

impl<'lend, St, L, E, F> FallibleLending<'lend> for OnceWithErr<St, L, F>
where
    L: ?Sized + CovariantFallibleLending,
    F: FnOnce(&mut St) -> E,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<St, L, E, F> FallibleLender for OnceWithErr<St, L, F>
where
    L: ?Sized + CovariantFallibleLending,
    F: FnOnce(&mut St) -> E,
{
    type Error = E;
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.f.take() {
            Some(f) => Err(f(&mut self.state)),
            None => Ok(None),
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl<St, L, E, F> DoubleEndedFallibleLender for OnceWithErr<St, L, F>
where
    L: ?Sized + CovariantFallibleLending,
    F: FnOnce(&mut St) -> E,
{
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.next()
    }
}

impl<St, L, E, F> FusedFallibleLender for OnceWithErr<St, L, F>
where
    L: ?Sized + CovariantFallibleLending,
    F: FnOnce(&mut St) -> E,
{
}

impl<St, L, E, F> ExactSizeFallibleLender for OnceWithErr<St, L, F>
where
    L: ?Sized + CovariantFallibleLending,
    F: FnOnce(&mut St) -> E,
{
}
