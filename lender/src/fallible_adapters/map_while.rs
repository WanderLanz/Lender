use crate::{
    FallibleLend, FallibleLender, FallibleLending, MapWhile, higher_order::FnMutHKAResOpt,
};

impl<'lend, L, P> FallibleLending<'lend> for MapWhile<L, P>
where
    P: for<'all> FnMutHKAResOpt<'all, FallibleLend<'all, L>, L::Error>,
    L: FallibleLender,
{
    type Lend = <P as FnMutHKAResOpt<'lend, FallibleLend<'lend, L>, L::Error>>::B;
}

impl<L, P> FallibleLender for MapWhile<L, P>
where
    P: for<'all> FnMutHKAResOpt<'all, FallibleLend<'all, L>, L::Error>,
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is the return type of P
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.lender.next()? {
            Some(next) => (self.predicate.as_inner_mut())(next),
            None => Ok(None),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
