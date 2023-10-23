use crate::{try_trait_v2::Try, DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending};
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Rev<L> {
    lender: L,
}
impl<L> Rev<L> {
    pub(crate) fn new(lender: L) -> Rev<L> {
        Rev { lender }
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
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        self.lender.next_back()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
        self.lender.advance_back_by(n)
    }
    #[inline]
    fn nth(&mut self, n: usize) -> Option<<Self as Lending<'_>>::Lend> {
        self.lender.nth_back(n)
    }
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> R,
        R: Try<Output = B>,
    {
        self.lender.try_rfold(init, f)
    }
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        self.lender.rfold(init, f)
    }
    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<<Self as Lending<'_>>::Lend>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        self.lender.rfind(predicate)
    }
}
impl<L> DoubleEndedLender for Rev<L>
where
    L: DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        self.lender.next()
    }
    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
        self.lender.advance_by(n)
    }
    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<<Self as Lending<'_>>::Lend> {
        self.lender.nth(n)
    }
    fn try_rfold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> R,
        R: Try<Output = B>,
    {
        self.lender.try_fold(init, f)
    }
    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        self.lender.fold(init, f)
    }
    fn rfind<P>(&mut self, predicate: P) -> Option<<Self as Lending<'_>>::Lend>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        self.lender.find(predicate)
    }
}
impl<L> ExactSizeLender for Rev<L>
where
    L: DoubleEndedLender + ExactSizeLender,
{
    #[inline]
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
