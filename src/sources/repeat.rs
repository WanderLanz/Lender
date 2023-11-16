use crate::{prelude::*, FusedLender};

/// Creates a new lender that endlessly repeats a single element.
///
/// See [`iter::repeat()`](core::iter::repeat) for more information.
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::repeat::<lend!(&'lend u8)>(&0u8);
/// assert_eq!(lender.next(), Some(&0));
/// ```
pub fn repeat<'a, L>(elt: Lend<'a, L>) -> Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    Repeat { elt }
}

/// A lender that repeats an element endlessly.
///
/// This `struct` is created by the [`repeat()`] function. See its documentation for more.
pub struct Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
{
    elt: Lend<'a, L>,
}

impl<'lend, 'a, L> Lending<'lend> for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L> Lender for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> Lend<'all, L>: Clone,
{
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend
        Some(unsafe { core::mem::transmute::<<Self as Lending<'a>>::Lend, Lend<'_, Self>>(self.elt.clone()) })
    }
    #[inline]
    fn advance_by(&mut self, _n: usize) -> Result<(), core::num::NonZeroUsize> {
        Ok(())
    }
}

impl<'a, L> DoubleEndedLender for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
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
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> Lend<'all, L>: Clone,
{
}
