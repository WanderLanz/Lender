use core::fmt;

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender,
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender,
    Lending,
    higher_order::{FnMutHKA, FnMutHKARes},
    try_trait_v2::Try,
};

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Map<L, F> {
    pub(crate) lender: L,
    f: F,
}

impl<L, F> Map<L, F> {
    pub(crate) fn new(lender: L, f: F) -> Map<L, F> {
        Map { lender, f }
    }

    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the mapping function.
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
        self.lender.rfold(init, move |acc, x| fold(acc, (self.f)(x)))
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

impl<'lend, L, F> FallibleLending<'lend> for Map<L, F>
where
    F: for<'all> FnMutHKARes<'all, FallibleLend<'all, L>, L::Error>,
    L: FallibleLender,
{
    type Lend = <F as FnMutHKARes<'lend, FallibleLend<'lend, L>, L::Error>>::B;
}

impl<L, F> FallibleLender for Map<L, F>
where
    F: for<'all> FnMutHKARes<'all, FallibleLend<'all, L>, L::Error>,
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.next()?.map(&mut self.f).transpose()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn try_fold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> Result<R, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender.try_fold(init, move |acc, x| fold(acc, (f)(x)?))
    }

    #[inline]
    fn fold<B, Fold>(mut self, init: B, mut fold: Fold) -> Result<B, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender
            .fold(init, move |acc, x| fold(acc, (self.f)(x)?))
    }
}

impl<L: DoubleEndedFallibleLender, F> DoubleEndedFallibleLender for Map<L, F>
where
    F: for<'all> FnMutHKARes<'all, FallibleLend<'all, L>, L::Error>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.next_back()?.map(&mut self.f).transpose()
    }

    #[inline]
    fn try_rfold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> Result<R, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender
            .try_rfold(init, move |acc, x| fold(acc, (f)(x)?))
    }

    #[inline]
    fn rfold<B, Fold>(mut self, init: B, mut fold: Fold) -> Result<B, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender
            .rfold(init, move |acc, x| fold(acc, (self.f)(x)?))
    }
}

impl<L: ExactSizeFallibleLender, F> ExactSizeFallibleLender for Map<L, F>
where
    F: for<'all> FnMutHKARes<'all, FallibleLend<'all, L>, L::Error>,
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

impl<L: FusedFallibleLender, F> FusedFallibleLender for Map<L, F> where
    F: for<'all> FnMutHKARes<'all, FallibleLend<'all, L>, L::Error>
{
}
