use core::{fmt, marker::PhantomData};

use crate::{FusedLender, prelude::*};

/// Creates a new lender that repeats elements endlessly by
/// applying the provided closure, the repeater,
/// `F: FnMut() -> A`.
///
/// The [`repeat_with()`] function calls the repeater over and
/// over again.
///
/// The [`Lender`] version of
/// [`iter::repeat_with()`](core::iter::repeat_with).
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::repeat_with::<
///     lend!(&'lend i32), _,
/// >(|| &0);
/// assert_eq!(lender.next(), Some(&0));
/// ```
#[inline]
pub fn repeat_with<'a, L, F>(f: F) -> RepeatWith<'a, L, F>
where
    L: ?Sized + CovariantLending + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    RepeatWith {
        f,
        _marker: PhantomData,
    }
}

/// A lender that repeats an element endlessly by applying a
/// closure.
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
        f.debug_struct("RepeatWith").finish_non_exhaustive()
    }
}

impl<'lend, 'a, L, F> Lending<'lend> for RepeatWith<'a, L, F>
where
    L: ?Sized + CovariantLending + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L, F> Lender for RepeatWith<'a, L, F>
where
    L: ?Sized + CovariantLending + 'a,
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

    /// Advances the lender by `n` elements.
    ///
    /// Unlike [`Repeat::advance_by`](crate::Repeat), which is
    /// a no-op, this method calls the closure `n` times
    /// (discarding the results) because the closure may have
    /// side effects.
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
        for _ in 0..n {
            (self.f)();
        }
        Ok(())
    }
}

impl<'a, L, F> DoubleEndedLender for RepeatWith<'a, L, F>
where
    L: ?Sized + CovariantLending + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    #[inline(always)]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.next()
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
        for _ in 0..n {
            (self.f)();
        }
        Ok(())
    }
}

impl<'a, L, F> FusedLender for RepeatWith<'a, L, F>
where
    L: ?Sized + CovariantLending + 'a,
    F: FnMut() -> Lend<'a, L>,
{
}
