use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender,
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender,
    Lending, try_trait_v2::Try,
};
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Rev<L> {
    lender: L,
}
impl<L> Rev<L> {
    pub(crate) fn new(lender: L) -> Rev<L> {
        Rev { lender }
    }

    pub fn into_inner(self) -> L {
        self.lender
    }
}
impl<'lend, L> Lending<'lend> for Rev<L>
where
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}
impl<L> Lender for Rev<L>
where
    L: DoubleEndedLender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next_back()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
        self.lender.advance_back_by(n)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        self.lender.nth_back(n)
    }

    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        self.lender.try_rfold(init, f)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.rfold(init, f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Lend<'_, Self>>
    where
        Self: Sized,
        P: FnMut(&Lend<'_, Self>) -> bool,
    {
        self.lender.rfind(predicate)
    }
}
impl<L> DoubleEndedLender for Rev<L>
where
    L: DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next()
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
        self.lender.advance_by(n)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        self.lender.nth(n)
    }

    fn try_rfold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        self.lender.try_fold(init, f)
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.fold(init, f)
    }

    fn rfind<P>(&mut self, predicate: P) -> Option<Lend<'_, Self>>
    where
        Self: Sized,
        P: FnMut(&Lend<'_, Self>) -> bool,
    {
        self.lender.find(predicate)
    }
}
impl<L> ExactSizeLender for Rev<L>
where
    L: DoubleEndedLender + ExactSizeLender,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }
}
impl<L> FusedLender for Rev<L> where L: DoubleEndedLender + FusedLender {}
impl<L> Default for Rev<L>
where
    L: Default,
{
    fn default() -> Self {
        Rev::new(L::default())
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

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        self.lender.advance_back_by(n)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.nth_back(n)
    }

    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        self.lender.try_rfold(init, f)
    }

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
    fn advance_back_by(
        &mut self,
        n: usize,
    ) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        self.lender.advance_by(n)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.nth(n)
    }

    fn try_rfold<B, F, R>(&mut self, init: B, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        self.lender.try_fold(init, f)
    }

    fn rfold<B, F>(self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.fold(init, f)
    }

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
