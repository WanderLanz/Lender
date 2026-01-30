use core::fmt;

use crate::{
    Lend, Lender, Lending,
    higher_order::FnMutHKAOpt,
};

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Scan<L, St, F> {
    pub(crate) lender: L,
    pub(crate) f: F,
    pub(crate) state: St,
}

impl<L, St, F> Scan<L, St, F> {
    pub(crate) fn new(lender: L, state: St, f: F) -> Scan<L, St, F> {
        Scan { lender, state, f }
    }

    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender, the state, and the scan function.
    pub fn into_parts(self) -> (L, St, F) {
        (self.lender, self.state, self.f)
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

impl<'lend, B, L, St, F> Lending<'lend> for Scan<L, St, F>
where
    F: FnMut((&'lend mut St, Lend<'lend, L>)) -> Option<B>,
    L: Lender,
    B: 'lend,
{
    type Lend = B;
}

impl<L, St, F> Lender for Scan<L, St, F>
where
    for<'all> F: FnMutHKAOpt<'all, (&'all mut St, Lend<'all, L>)>,
    L: Lender,
{
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        (self.f)((&mut self.state, self.lender.next()?))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
