use crate::{
    FallibleLend, FallibleLender, FallibleLending, Scan,
    higher_order::FnMutHKAResOpt,
};

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
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance_fallible!();

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
