use core::fmt;

use crate::{Covar, Lend, Lender, Lending, higher_order::FnMutHKAOpt};

/// A lender to maintain state while lending elements from the underlying
/// lender.
///
/// This `struct` is created by the [`scan()`](crate::Lender::scan) method on
/// [`Lender`].
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Scan<L, St, F> {
    pub(crate) lender: L,
    pub(crate) state: St,
    pub(crate) f: Covar<F>,
}

impl<L, St, F> Scan<L, St, F> {
    /// Returns the inner lender.
    #[inline]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender, the state, and the scan function.
    #[inline]
    pub fn into_parts(self) -> (L, St, Covar<F>) {
        (self.lender, self.state, self.f)
    }
}

impl<L: Lender, St, F> Scan<L, St, F> {
    #[inline]
    pub(crate) fn new(lender: L, state: St, f: Covar<F>) -> Scan<L, St, F> {
        crate::__check_lender_covariance::<L>();
        Scan { lender, state, f }
    }
}

impl<L: fmt::Debug, St: fmt::Debug, F> fmt::Debug for Scan<L, St, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scan")
            .field("lender", &self.lender)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl<'lend, L, St, F> Lending<'lend> for Scan<L, St, F>
where
    F: for<'all> FnMutHKAOpt<'all, (&'all mut St, Lend<'all, L>)>,
    L: Lender,
{
    type Lend = <F as FnMutHKAOpt<'lend, (&'lend mut St, Lend<'lend, L>)>>::B;
}

impl<L, St, F> Lender for Scan<L, St, F>
where
    for<'all> F: FnMutHKAOpt<'all, (&'all mut St, Lend<'all, L>)>,
    L: Lender,
{
    // SAFETY: the lend is the return type of F, whose covariance
    // has been checked at Covar construction time.
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        (self.f.as_inner_mut())((&mut self.state, self.lender.next()?))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
