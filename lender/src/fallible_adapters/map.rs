use crate::{
    Covar, DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, Map, higher_order::FnMutHKARes,
};

impl<L: FallibleLender, F> Map<L, F> {
    #[inline(always)]
    pub(crate) fn new_fallible(lender: L, f: Covar<F>) -> Map<L, F> {
        crate::__check_fallible_lender_covariance::<L>();
        Map { lender, f }
    }
}

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
        self.lender.next()?.map(self.f.as_inner_mut()).transpose()
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
        R: crate::try_trait_v2::Try<Output = B>,
    {
        let f = self.f.as_inner_mut();
        self.lender.try_fold(init, move |acc, x| fold(acc, (f)(x)?))
    }

    #[inline]
    fn fold<B, Fold>(mut self, init: B, mut fold: Fold) -> Result<B, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let f = self.f.as_inner_mut();
        self.lender.fold(init, move |acc, x| fold(acc, (f)(x)?))
    }
}

impl<L: DoubleEndedFallibleLender, F> DoubleEndedFallibleLender for Map<L, F>
where
    F: for<'all> FnMutHKARes<'all, FallibleLend<'all, L>, L::Error>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender
            .next_back()?
            .map(self.f.as_inner_mut())
            .transpose()
    }

    #[inline]
    fn try_rfold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> Result<R, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: crate::try_trait_v2::Try<Output = B>,
    {
        let f = self.f.as_inner_mut();
        self.lender
            .try_rfold(init, move |acc, x| fold(acc, (f)(x)?))
    }

    #[inline]
    fn rfold<B, Fold>(mut self, init: B, mut fold: Fold) -> Result<B, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let f = self.f.as_inner_mut();
        self.lender.rfold(init, move |acc, x| fold(acc, (f)(x)?))
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
