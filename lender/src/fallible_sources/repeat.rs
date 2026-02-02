use core::{fmt, marker::PhantomData};

use crate::prelude::*;

/// Creates a new fallible lender that endlessly repeats a
/// single element.
///
/// This is the [`FallibleLender`] version of
/// [`iter::repeat()`](core::iter::repeat).
///
/// To create a lender that endlessly repeats an error, use
/// [`repeat_err()`].
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::fallible_repeat::<
///     '_, fallible_lend!(&'lend u8), String,
/// >(&0u8);
/// assert_eq!(lender.next().unwrap(), Some(&0));
/// ```
#[inline]
pub fn repeat<'a, L, E>(elt: FallibleLend<'a, L>) -> Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
    Repeat {
        elt,
        _marker: PhantomData,
    }
}

/// Creates a new fallible lender that endlessly repeats an
/// error.
///
/// This is the error counterpart to [`repeat()`]: it yields
/// the given error on every call to `next`.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::fallible_repeat_err::<
///     fallible_lend!(&'lend u8), _,
/// >("error".to_string());
/// assert_eq!(lender.next(), Err("error".to_string()));
/// assert_eq!(lender.next(), Err("error".to_string()));
/// ```
#[inline]
pub fn repeat_err<L, E>(error: E) -> RepeatErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
    E: Clone,
{
    RepeatErr {
        err: error,
        _marker: PhantomData,
    }
}

/// A fallible lender that repeats an element endlessly.
///
/// This `struct` is created by the [`repeat()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
{
    elt: FallibleLend<'a, L>,
    _marker: PhantomData<E>,
}

impl<'a, L, E> Clone for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    FallibleLend<'a, L>: Clone,
{
    fn clone(&self) -> Self {
        Repeat {
            elt: self.elt.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'a, L, E> fmt::Debug for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
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
    E: 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
    type Error = E;
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(Some(
            // SAFETY: 'a: 'lend
            unsafe {
                core::mem::transmute::<FallibleLend<'a, Self>, FallibleLend<'_, Self>>(
                    self.elt.clone(),
                )
            },
        ))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }

    #[inline(always)]
    fn advance_by(
        &mut self,
        _n: usize,
    ) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        Ok(Ok(()))
    }
}

impl<'a, L, E> DoubleEndedFallibleLender for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    E: 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.next()
    }

    #[inline(always)]
    fn advance_back_by(
        &mut self,
        _n: usize,
    ) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        Ok(Ok(()))
    }
}

impl<'a, L, E> FusedFallibleLender for Repeat<'a, L, E>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    E: 'a,
    for<'all> FallibleLend<'all, L>: Clone,
{
}

/// A fallible lender that endlessly repeats an error.
///
/// This `struct` is created by the [`repeat_err()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct RepeatErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    err: E,
    _marker: PhantomData<L>,
}

impl<L, E: Clone> Clone for RepeatErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    fn clone(&self) -> Self {
        RepeatErr {
            err: self.err.clone(),
            _marker: PhantomData,
        }
    }
}

impl<L, E: fmt::Debug> fmt::Debug for RepeatErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleRepeatErr")
            .field("err", &self.err)
            .finish()
    }
}

impl<'lend, L, E> FallibleLending<'lend> for RepeatErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L, E> FallibleLender for RepeatErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
    E: Clone,
{
    type Error = E;
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance_fallible!();

    /// Note: the error is cloned and returned on every call
    /// to `next`.
    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Err(self.err.clone())
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
            Err(self.err.clone())
        } else {
            Ok(Ok(()))
        }
    }
}

impl<L, E> DoubleEndedFallibleLender for RepeatErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
    E: Clone,
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
            Err(self.err.clone())
        } else {
            Ok(Ok(()))
        }
    }
}

impl<L, E> FusedFallibleLender for RepeatErr<L, E>
where
    L: ?Sized + CovariantFallibleLending,
    E: Clone,
{
}
