use core::fmt;

use crate::{FusedLender, prelude::*};

/// Creates a new lender that endlessly repeats a single element.
///
/// The [`Lender`] version of
/// [`iter::repeat()`](core::iter::repeat).
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::repeat::<lend!(&'lend u8)>(&0u8);
/// assert_eq!(lender.next(), Some(&0));
/// ```
pub fn repeat<'a, L>(elt: Lend<'a, L>) -> Repeat<'a, L>
where
    L: ?Sized + CovariantLending + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    Repeat { elt }
}

/// A lender that repeats an element endlessly.
///
/// This `struct` is created by the [`repeat()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Repeat<'a, L>
where
    L: ?Sized + CovariantLending + 'a,
{
    elt: Lend<'a, L>,
}

impl<'a, L> Clone for Repeat<'a, L>
where
    L: ?Sized + CovariantLending + 'a,
    Lend<'a, L>: Clone,
{
    fn clone(&self) -> Self {
        Repeat {
            elt: self.elt.clone(),
        }
    }
}

impl<'a, L> fmt::Debug for Repeat<'a, L>
where
    L: ?Sized + CovariantLending + 'a,
    Lend<'a, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Repeat").field("elt", &self.elt).finish()
    }
}

impl<'lend, 'a, L> Lending<'lend> for Repeat<'a, L>
where
    L: ?Sized + CovariantLending + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L> Lender for Repeat<'a, L>
where
    L: ?Sized + CovariantLending + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend
        Some(unsafe { core::mem::transmute::<Lend<'a, Self>, Lend<'_, Self>>(self.elt.clone()) })
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

impl<'a, L> DoubleEndedLender for Repeat<'a, L>
where
    L: ?Sized + CovariantLending + 'a,
    for<'all> Lend<'all, L>: Clone,
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

impl<'a, L> FusedLender for Repeat<'a, L>
where
    L: ?Sized + CovariantLending + 'a,
    for<'all> Lend<'all, L>: Clone,
{
}
