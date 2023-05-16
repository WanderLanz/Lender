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
pub fn repeat<'a, L>(elt: <L as Lending<'a>>::Lend) -> Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> <L as Lending<'all>>::Lend: Clone,
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
    elt: <L as Lending<'a>>::Lend,
}

impl<'lend, 'a, L> Lending<'lend> for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> <L as Lending<'all>>::Lend: Clone,
{
    type Lend = <L as Lending<'lend>>::Lend;
}

impl<'a, L> Lender for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> <L as Lending<'all>>::Lend: Clone,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        // SAFETY: 'a: 'lend
        Some(unsafe { core::mem::transmute(self.elt.clone()) })
    }
    #[inline]
    fn advance_by(&mut self, _n: usize) -> Result<(), core::num::NonZeroUsize> { Ok(()) }
}

impl<'a, L> DoubleEndedLender for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> <L as Lending<'all>>::Lend: Clone,
{
    #[inline]
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.next() }
    #[inline]
    fn advance_back_by(&mut self, _n: usize) -> Result<(), core::num::NonZeroUsize> { Ok(()) }
}

impl<'a, L> FusedLender for Repeat<'a, L>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    for<'all> <L as Lending<'all>>::Lend: Clone,
{
}
