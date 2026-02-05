use crate::{
    DoubleEndedFallibleLender, FallibleLend, FallibleLender, FallibleLending, Filter,
    FusedFallibleLender, try_trait_v2::Try,
};

impl<'lend, L, P> FallibleLending<'lend> for Filter<L, P>
where
    P: FnMut(&FallibleLend<'lend, L>) -> Result<bool, L::Error>,
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L, P> FallibleLender for Filter<L, P>
where
    P: FnMut(&FallibleLend<'_, L>) -> Result<bool, L::Error>,
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.find(&mut self.predicate)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        #[inline]
        fn f<
            E,
            L: for<'all> FallibleLending<'all>,
            F: FnMut(&FallibleLend<'_, L>) -> Result<bool, E>,
        >(
            mut f: F,
        ) -> impl FnMut(FallibleLend<'_, L>) -> Result<usize, E> {
            move |x| (f)(&x).map(|res| res as usize)
        }
        // SAFETY: the closure returns Result<usize, E> (owned types),
        // which is trivially covariant.
        let lender = self
            .lender
            .map(unsafe { crate::Covar::__new(f::<_, Self, _>(self.predicate)) });
        crate::fallible_adapters::non_fallible_adapter::process(lender, |iter| {
            core::iter::Iterator::sum(iter)
        })
        .map_err(|(_, err)| err)
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let predicate = &mut self.predicate;
        self.lender.try_fold(init, move |acc, x| {
            if (predicate)(&x)? {
                f(acc, x)
            } else {
                Ok(R::from_output(acc))
            }
        })
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.fold(init, move |acc, x| {
            if (self.predicate)(&x)? {
                f(acc, x)
            } else {
                Ok(acc)
            }
        })
    }
}

impl<L, P> DoubleEndedFallibleLender for Filter<L, P>
where
    P: FnMut(&FallibleLend<'_, L>) -> Result<bool, L::Error>,
    L: DoubleEndedFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.rfind(&mut self.predicate)
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let predicate = &mut self.predicate;
        self.lender.try_rfold(init, move |acc, x| {
            if (predicate)(&x)? {
                f(acc, x)
            } else {
                Ok(R::from_output(acc))
            }
        })
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.rfold(init, move |acc, x| {
            if (self.predicate)(&x)? {
                f(acc, x)
            } else {
                Ok(acc)
            }
        })
    }
}

impl<L, P> FusedFallibleLender for Filter<L, P>
where
    P: FnMut(&FallibleLend<'_, L>) -> Result<bool, L::Error>,
    L: FusedFallibleLender,
{
}
