use crate::{
    Covar, DoubleEndedFallibleLender, FallibleLend, FallibleLender, FallibleLending, FilterMap,
    FusedFallibleLender, higher_order::FnMutHKAResOpt,
};

impl<L: FallibleLender, F> FilterMap<L, F> {
    #[inline]
    pub(crate) fn new_fallible(lender: L, f: Covar<F>) -> FilterMap<L, F> {
        crate::__check_fallible_lender_covariance::<L>();
        FilterMap { lender, f }
    }
}

impl<'lend, L, F> FallibleLending<'lend> for FilterMap<L, F>
where
    F: for<'all> FnMutHKAResOpt<'all, FallibleLend<'all, L>, L::Error>,
    L: FallibleLender,
{
    type Lend = <F as FnMutHKAResOpt<'lend, FallibleLend<'lend, L>, L::Error>>::B;
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
    fn count(mut self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        let mut count = 0;
        while let Some(x) = self.lender.next()? {
            if (self.f.as_inner_mut())(x)?.is_some() {
                count += 1;
            }
        }
        Ok(count)
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
