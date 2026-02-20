use core::fmt;

use crate::{Covar, Lend, Lender, Lending, higher_order::FnMutHKAOpt};

/// A lender that yields elements based on a predicate and maps them.
///
/// This `struct` is created by the [`map_while()`](crate::Lender::map_while)
/// method on [`Lender`].
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct MapWhile<L, P> {
    pub(crate) lender: L,
    pub(crate) predicate: Covar<P>,
}

impl<L, P> MapWhile<L, P> {
    /// Returns the inner lender.
    #[inline]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the predicate.
    #[inline]
    pub fn into_parts(self) -> (L, Covar<P>) {
        (self.lender, self.predicate)
    }
}

impl<L: Lender, P> MapWhile<L, P> {
    #[inline]
    pub(crate) fn new(lender: L, predicate: Covar<P>) -> MapWhile<L, P> {
        crate::__check_lender_covariance::<L>();
        MapWhile { lender, predicate }
    }
}

impl<L: fmt::Debug, P> fmt::Debug for MapWhile<L, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapWhile")
            .field("lender", &self.lender)
            .finish_non_exhaustive()
    }
}

impl<'lend, L, P> Lending<'lend> for MapWhile<L, P>
where
    P: for<'all> FnMutHKAOpt<'all, Lend<'all, L>>,
    L: Lender,
{
    type Lend = <P as FnMutHKAOpt<'lend, Lend<'lend, L>>>::B;
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
        (self.predicate.as_inner_mut())(self.lender.next()?)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
