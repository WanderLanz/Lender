use core::fmt;

use crate::{DoubleEndedLender, FusedLender, Lend, Lender, Lending, try_trait_v2::Try};

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Filter<L, P> {
    pub(crate) lender: L,
    pub(crate) predicate: P,
}

impl<L, P> Filter<L, P> {
    #[inline(always)]
    pub(crate) fn new(lender: L, predicate: P) -> Filter<L, P> {
        Filter { lender, predicate }
    }

    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the predicate.
    #[inline(always)]
    pub fn into_parts(self) -> (L, P) {
        (self.lender, self.predicate)
    }
}

impl<I: fmt::Debug, P> fmt::Debug for Filter<I, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Filter")
            .field("lender", &self.lender)
            .finish()
    }
}

impl<'lend, L, P> Lending<'lend> for Filter<L, P>
where
    P: FnMut(&Lend<'lend, L>) -> bool,
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}

impl<L, P> Lender for Filter<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: Lender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.find(&mut self.predicate)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        #[inline]
        fn f<L: for<'all> Lending<'all>, F: FnMut(&Lend<'_, L>) -> bool>(
            mut f: F,
        ) -> impl FnMut(Lend<'_, L>) -> usize {
            move |x| (f)(&x) as usize
        }
        self.lender.map(f::<Self, _>(self.predicate)).iter().sum()
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let predicate = &mut self.predicate;
        self.lender.try_fold(init, move |acc, x| {
            if (predicate)(&x) { f(acc, x) } else { R::from_output(acc) }
        })
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.fold(init, move |acc, x| {
            if (self.predicate)(&x) { f(acc, x) } else { acc }
        })
    }
}

impl<L, P> DoubleEndedLender for Filter<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.rfind(&mut self.predicate)
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let predicate = &mut self.predicate;
        self.lender.try_rfold(init, move |acc, x| {
            if (predicate)(&x) { f(acc, x) } else { R::from_output(acc) }
        })
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.rfold(init, move |acc, x| {
            if (self.predicate)(&x) { f(acc, x) } else { acc }
        })
    }
}

impl<L, P> FusedLender for Filter<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: FusedLender,
{
}
