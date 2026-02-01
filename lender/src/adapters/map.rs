use core::fmt;

use crate::{
    DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending, higher_order::FnMutHKA,
    try_trait_v2::Try,
};

/// A lender that maps the values of the underlying lender with a closure.
///
/// This `struct` is created by the [`map()`](crate::Lender::map) method on [`Lender`].
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Map<L, F> {
    pub(crate) lender: L,
    pub(crate) f: F,
}

impl<L, F> Map<L, F> {
    #[inline(always)]
    pub(crate) fn new(lender: L, f: F) -> Map<L, F> {
        Map { lender, f }
    }

    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the mapping function.
    #[inline(always)]
    pub fn into_parts(self) -> (L, F) {
        (self.lender, self.f)
    }
}

impl<L: fmt::Debug, F> fmt::Debug for Map<L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map").field("lender", &self.lender).finish()
    }
}

impl<'lend, L, F> Lending<'lend> for Map<L, F>
where
    F: for<'all> FnMutHKA<'all, Lend<'all, L>>,
    L: Lender,
{
    type Lend = <F as FnMutHKA<'lend, Lend<'lend, L>>>::B;
}

impl<L, F> Lender for Map<L, F>
where
    F: for<'all> FnMutHKA<'all, Lend<'all, L>>,
    L: Lender,
{
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next().map(&mut self.f)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn try_fold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender.try_fold(init, move |acc, x| fold(acc, (f)(x)))
    }

    #[inline]
    fn fold<B, Fold>(mut self, init: B, mut fold: Fold) -> B
    where
        Self: Sized,
        Fold: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.fold(init, move |acc, x| fold(acc, (self.f)(x)))
    }
}

impl<L: DoubleEndedLender, F> DoubleEndedLender for Map<L, F>
where
    F: for<'all> FnMutHKA<'all, Lend<'all, L>>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next_back().map(&mut self.f)
    }

    #[inline]
    fn try_rfold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender.try_rfold(init, move |acc, x| fold(acc, (f)(x)))
    }

    #[inline]
    fn rfold<B, Fold>(mut self, init: B, mut fold: Fold) -> B
    where
        Self: Sized,
        Fold: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender
            .rfold(init, move |acc, x| fold(acc, (self.f)(x)))
    }
}

impl<L: ExactSizeLender, F> ExactSizeLender for Map<L, F>
where
    F: for<'all> FnMutHKA<'all, Lend<'all, L>>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
    }
}

impl<L: FusedLender, F> FusedLender for Map<L, F> where F: for<'all> FnMutHKA<'all, Lend<'all, L>> {}
