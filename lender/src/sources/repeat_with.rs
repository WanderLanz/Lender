use core::{fmt, marker::PhantomData};

use crate::{FusedFallibleLender, FusedLender, prelude::*};

/// Creates a new lender that repeats elements endlessly by
/// applying the provided closure, the repeater, `F: FnMut() -> A`.
///
/// The [`repeat_with()`] function calls the repeater over and over again.
///
/// The [`Lender`] version of [`iter::repeat_with()`](core::iter::repeat_with).
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::repeat_with::<lend!(&'lend u8), _>(|| &0u8);
/// assert_eq!(lender.next(), Some(&0));
/// ```
pub fn repeat_with<'a, L, F>(f: F) -> RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    RepeatWith {
        f,
        _marker: PhantomData,
    }
}

/// A lender that repeats an element endlessly by applying a closure.
///
/// This `struct` is created by the [`repeat_with()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct RepeatWith<'a, L: ?Sized, F> {
    f: F,
    _marker: core::marker::PhantomData<&'a L>,
}

impl<L: ?Sized, F: Clone> Clone for RepeatWith<'_, L, F> {
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<L: ?Sized, F> fmt::Debug for RepeatWith<'_, L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RepeatWith").finish()
    }
}

impl<'lend, 'a, L, F> Lending<'lend> for RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L, F> Lender for RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend
        Some(unsafe { core::mem::transmute::<Lend<'a, L>, Lend<'_, L>>((self.f)()) })
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

impl<'a, L, F> DoubleEndedLender for RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
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

impl<'a, L, F> FusedLender for RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
{
}

/// Creates a new fallible lender that repeats elements endlessly by applying
/// the provided closure, the repeater, `F: FnMut() -> Result<A, E>`.
///
/// The [`fallible_repeat_with()`] function calls the repeater over and over again.
///
/// The [`FallibleLender`] version of [`iter::repeat_with()`](core::iter::repeat_with).
pub fn fallible_repeat_with<'a, L, E, F>(f: F) -> FallibleRepeatWith<'a, L, E, F>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    F: FnMut() -> Result<FallibleLend<'a, L>, E>,
{
    FallibleRepeatWith {
        f,
        _marker: PhantomData,
    }
}

/// A fallible lender that repeats an element endlessly by applying a closure.
///
/// This `struct` is created by the [`fallible_repeat_with()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FallibleRepeatWith<'a, L: ?Sized, E, F> {
    f: F,
    _marker: core::marker::PhantomData<(&'a L, E)>,
}

impl<L: ?Sized, E, F: Clone> Clone for FallibleRepeatWith<'_, L, E, F> {
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<L: ?Sized, E, F> fmt::Debug for FallibleRepeatWith<'_, L, E, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FallibleRepeatWith").finish()
    }
}

impl<'lend, 'a, L, E, F> FallibleLending<'lend> for FallibleRepeatWith<'a, L, E, F>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    F: FnMut() -> Result<FallibleLend<'a, L>, E>,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'a, L, E, F> FallibleLender for FallibleRepeatWith<'a, L, E, F>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    F: FnMut() -> Result<FallibleLend<'a, L>, E>,
{
    type Error = E;
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        (self.f)().map(|value| {
            Some(
                // SAFETY: 'a: 'lend
                unsafe { core::mem::transmute::<FallibleLend<'a, L>, FallibleLend<'_, L>>(value) },
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
        for _ in 0..n {
            (self.f)()?;
        }
        Ok(Ok(()))
    }
}

impl<'a, L, E, F> DoubleEndedFallibleLender for FallibleRepeatWith<'a, L, E, F>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    F: FnMut() -> Result<FallibleLend<'a, L>, E>,
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
        for _ in 0..n {
            (self.f)()?;
        }
        Ok(Ok(()))
    }
}

impl<'a, L, E, F> FusedFallibleLender for FallibleRepeatWith<'a, L, E, F>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    F: FnMut() -> Result<FallibleLend<'a, L>, E>,
{
}
