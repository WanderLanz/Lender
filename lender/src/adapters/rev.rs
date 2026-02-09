use crate::{
    DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending, try_trait_v2::Try,
};

/// A double-ended lender with the direction inverted.
///
/// This `struct` is created by the [`rev()`](crate::Lender::rev) method on
/// [`Lender`]. See its documentation for more.
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Rev<L> {
    pub(crate) lender: L,
}

impl<L> Rev<L> {
    #[inline(always)]
    pub(crate) fn new(lender: L) -> Rev<L> {
        Rev { lender }
    }

    /// Returns the inner lender.
    #[inline(always)]
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
    #[inline(always)]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next_back()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline(always)]
    fn advance_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
        self.lender.advance_back_by(n)
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        self.lender.nth_back(n)
    }

    #[inline(always)]
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        self.lender.try_rfold(init, f)
    }

    #[inline(always)]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.rfold(init, f)
    }

    #[inline(always)]
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
    #[inline(always)]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next()
    }

    #[inline(always)]
    fn advance_back_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
        self.lender.advance_by(n)
    }

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        self.lender.nth(n)
    }

    #[inline(always)]
    fn try_rfold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        self.lender.try_fold(init, f)
    }

    #[inline(always)]
    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.fold(init, f)
    }

    #[inline(always)]
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

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
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
