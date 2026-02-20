use core::num::NonZeroUsize;

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, Rev, try_trait_v2::Try,
};

impl<L: FallibleLender> Rev<L> {
    #[inline]
    pub(crate) fn new_fallible(lender: L) -> Rev<L> {
        crate::__check_fallible_lender_covariance::<L>();
        Rev { lender }
    }
}

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

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.next_back()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        self.lender.advance_back_by(n)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.nth_back(n)
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        self.lender.try_rfold(init, f)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.rfold(init, f)
    }

    #[inline]
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
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.next()
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        self.lender.advance_by(n)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.nth(n)
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        self.lender.try_fold(init, f)
    }

    #[inline]
    fn rfold<B, F>(self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.fold(init, f)
    }

    #[inline]
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
    #[inline]
    fn len(&self) -> usize {
        self.lender.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
    }
}

impl<L> FusedFallibleLender for Rev<L> where L: DoubleEndedFallibleLender + FusedFallibleLender {}
