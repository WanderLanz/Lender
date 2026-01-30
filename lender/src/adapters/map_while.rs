use core::fmt;

use crate::{
    Lend, Lender, Lending,
    higher_order::FnMutHKAOpt,
};

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct MapWhile<L, P> {
    pub(crate) lender: L,
    pub(crate) predicate: P,
}

impl<L, P> MapWhile<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> MapWhile<L, P> {
        MapWhile { lender, predicate }
    }

    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the predicate.
    pub fn into_parts(self) -> (L, P) {
        (self.lender, self.predicate)
    }
}

impl<L: fmt::Debug, P> fmt::Debug for MapWhile<L, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapWhile")
            .field("lender", &self.lender)
            .finish()
    }
}

impl<'lend, B, L, P> Lending<'lend> for MapWhile<L, P>
where
    P: FnMut(Lend<'lend, L>) -> Option<B>,
    L: Lender,
    B: 'lend,
{
    type Lend = B;
}

impl<L, P> Lender for MapWhile<L, P>
where
    P: for<'all> FnMutHKAOpt<'all, Lend<'all, L>>,
    L: Lender,
{
    // SAFETY: the lend is the return type of P
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        (self.predicate)(self.lender.next()?)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
