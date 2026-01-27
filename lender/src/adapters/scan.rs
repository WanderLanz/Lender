use core::fmt;

use crate::{
    higher_order::{FnMutHKAOpt, FnMutHKAResOpt},
    FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending,
};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Scan<L, St, F> {
    lender: L,
    f: F,
    state: St,
}
impl<L, St, F> Scan<L, St, F> {
    pub(crate) fn new(lender: L, state: St, f: F) -> Scan<L, St, F> {
        Scan { lender, state, f }
    }
    pub fn into_inner(self) -> L {
        self.lender
    }
    pub fn into_parts(self) -> (L, St, F) {
        (self.lender, self.state, self.f)
    }
}
impl<L: fmt::Debug, St: fmt::Debug, F> fmt::Debug for Scan<L, St, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scan").field("lender", &self.lender).field("state", &self.state).finish()
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
    crate::inherit_covariance!(L);
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

impl<'lend, B, L, St, F> FallibleLending<'lend> for Scan<L, St, F>
where
    F: FnMut((&'lend mut St, FallibleLend<'lend, L>)) -> Result<Option<B>, L::Error>,
    L: FallibleLender,
    B: 'lend,
{
    type Lend = B;
}
impl<L, St, F> FallibleLender for Scan<L, St, F>
where
    for<'all> F: FnMutHKAResOpt<'all, (&'all mut St, FallibleLend<'all, L>), L::Error>,
    L: FallibleLender,
{
    type Error = L::Error;
    crate::inherit_covariance_fallible!(L);

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.lender.next()? {
            Some(next) => (self.f)((&mut self.state, next)),
            None => Ok(None),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
