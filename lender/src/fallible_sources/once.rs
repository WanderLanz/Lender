use core::{fmt, marker::PhantomData};

use crate::{
    CovariantFallibleLending, DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend,
    FallibleLender, FallibleLending, FusedFallibleLender,
};

/// Creates a fallible lender that yields a value exactly once.
///
/// This is the [`FallibleLender`] equivalent of
/// [`core::iter::once()`]: it yields one item and then
/// returns `Ok(None)`.
///
/// To create a lender that yields a single error, use
/// [`once_err()`].
///
/// # Examples
///
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::fallible_once::<
///     fallible_lend!(&'lend i32), String,
/// >(&42);
/// assert_eq!(lender.next(), Ok(Some(&42)));
/// assert_eq!(lender.next(), Ok(None));
/// ```
#[inline]
pub fn once<'a, L: ?Sized + CovariantFallibleLending, E>(
    value: FallibleLend<'a, L>,
) -> Once<'a, L, E> {
    Once {
        inner: Some(value),
        _marker: PhantomData,
    }
}

/// Creates a fallible lender that yields a single error.
///
/// This is the error counterpart to [`once()`]: it yields one
/// error and then returns `Ok(None)`.
///
/// # Examples
///
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::fallible_once_err::<
///     fallible_lend!(&'lend i32), _,
/// >("error".to_string());
/// assert_eq!(lender.next(), Err("error".to_string()));
/// assert_eq!(lender.next(), Ok(None));
/// ```
#[inline]
pub fn once_err<L: ?Sized + CovariantFallibleLending, E>(error: E) -> OnceErr<L, E> {
    OnceErr {
        inner: Some(error),
        _marker: PhantomData,
    }
}

/// A fallible lender that yields a value exactly once.
///
/// This `struct` is created by the
/// [`fallible_once()`](crate::fallible_once) function.
///
/// The [`FallibleLender`] version of [`core::iter::Once`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Once<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    inner: Option<FallibleLend<'a, L>>,
    _marker: PhantomData<fn() -> E>,
}

impl<'a, L, E> Clone for Once<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending,
    FallibleLend<'a, L>: Clone,
{
    fn clone(&self) -> Self {
        Once {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'a, L, E> fmt::Debug for Once<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending,
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
        Ok(self.inner.take().map(|value| {
            // SAFETY: 'a: 'lend
            unsafe { core::mem::transmute::<FallibleLend<'a, Self>, FallibleLend<'_, Self>>(value) }
        }))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
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
    #[inline(always)]
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

/// A fallible lender that yields a single error.
///
/// This `struct` is created by the [`once_err()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct OnceErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    inner: Option<E>,
    _marker: PhantomData<fn() -> L>,
}

impl<L, E: Clone> Clone for OnceErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    fn clone(&self) -> Self {
        OnceErr {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<L, E: fmt::Debug> fmt::Debug for OnceErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleOnceErr")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'lend, L, E> FallibleLending<'lend> for OnceErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L, E> FallibleLender for OnceErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    type Error = E;
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.inner.take() {
            Some(err) => Err(err),
            None => Ok(None),
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl<L, E> DoubleEndedFallibleLender for OnceErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.next()
    }
}

impl<L, E> FusedFallibleLender for OnceErr<L, E> where L: ?Sized + CovariantFallibleLending {}

impl<L, E> ExactSizeFallibleLender for OnceErr<L, E> where L: ?Sized + CovariantFallibleLending {}
