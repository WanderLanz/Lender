use crate::{
    DoubleEndedFallibleLender, FallibleLend, FallibleLender, FallibleLending, FilterMap,
    FusedFallibleLender, higher_order::FnMutHKAResOpt,
};

impl<'lend, B, L, F> FallibleLending<'lend> for FilterMap<L, F>
where
    F: FnMut(FallibleLend<'lend, L>) -> Result<Option<B>, L::Error>,
    B: 'lend,
    L: FallibleLender,
{
    type Lend = B;
}

impl<L, F> FallibleLender for FilterMap<L, F>
where
    for<'all> F: FnMutHKAResOpt<'all, FallibleLend<'all, L>, L::Error>,
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is the return type of F, whose covariance
    // has been checked at Covar construction time.
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        while let Some(x) = self.lender.next()? {
            if let Some(y) = (self.f.as_inner_mut())(x)? {
                return Ok(Some(
                    // SAFETY: polonius return
                    unsafe {
                        core::mem::transmute::<FallibleLend<'_, Self>, FallibleLend<'_, Self>>(y)
                    },
                ));
            }
        }
        Ok(None)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}

impl<L, F> DoubleEndedFallibleLender for FilterMap<L, F>
where
    for<'all> F: FnMutHKAResOpt<'all, FallibleLend<'all, L>, L::Error>,
    L: DoubleEndedFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        while let Some(x) = self.lender.next_back()? {
            if let Some(y) = (self.f.as_inner_mut())(x)? {
                return Ok(Some(
                    // SAFETY: polonius return
                    unsafe {
                        core::mem::transmute::<FallibleLend<'_, Self>, FallibleLend<'_, Self>>(y)
                    },
                ));
            }
        }
        Ok(None)
    }
}

impl<L, F> FusedFallibleLender for FilterMap<L, F>
where
    for<'all> F: FnMutHKAResOpt<'all, FallibleLend<'all, L>, L::Error>,
    L: FusedFallibleLender,
{
}
