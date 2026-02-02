use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, Rev, try_trait_v2::Try,
};

impl<'lend, L> FallibleLending<'lend> for Rev<L>
where
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L> FallibleLender for Rev<L>
where
    L: DoubleEndedFallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline(always)]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.next_back()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline(always)]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        self.lender.advance_back_by(n)
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.nth_back(n)
    }

    #[inline(always)]
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        self.lender.try_rfold(init, f)
    }

    #[inline(always)]
    fn fold<B, F>(self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.rfold(init, f)
    }

    #[inline(always)]
    fn find<P>(&mut self, predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        self.lender.rfind(predicate)
    }
}

impl<L> DoubleEndedFallibleLender for Rev<L>
where
    L: DoubleEndedFallibleLender,
{
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.next()
    }

    #[inline(always)]
    fn advance_back_by(
        &mut self,
        n: usize,
    ) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        self.lender.advance_by(n)
    }

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.nth(n)
    }

    #[inline(always)]
    fn try_rfold<B, F, R>(&mut self, init: B, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        self.lender.try_fold(init, f)
    }

    #[inline(always)]
    fn rfold<B, F>(self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.fold(init, f)
    }

    #[inline(always)]
    fn rfind<P>(&mut self, predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        self.lender.find(predicate)
    }
}

impl<L> ExactSizeFallibleLender for Rev<L>
where
    L: ExactSizeFallibleLender + DoubleEndedFallibleLender,
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

impl<L> FusedFallibleLender for Rev<L> where L: DoubleEndedFallibleLender + FusedFallibleLender {}
