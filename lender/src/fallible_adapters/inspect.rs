use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, Inspect,
};

impl<L: FallibleLender, F> Inspect<L, F> {
    #[inline(always)]
    pub(crate) fn new_fallible(lender: L, f: F) -> Inspect<L, F> {
        let _ = L::__check_covariance(crate::CovariantProof::new());
        Inspect { lender, f }
    }
}

impl<'lend, L, F> FallibleLending<'lend> for Inspect<L, F>
where
    L: FallibleLender,
    F: FnMut(&FallibleLend<'lend, L>) -> Result<(), L::Error>,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L, F> FallibleLender for Inspect<L, F>
where
    L: FallibleLender,
    F: FnMut(&FallibleLend<'_, L>) -> Result<(), L::Error>,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let next = self.lender.next()?;
        if let Some(ref x) = next {
            (self.f)(x)?;
        }
        Ok(next)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn try_fold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> Result<R, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: crate::try_trait_v2::Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender.try_fold(init, move |acc, x| {
            (f)(&x)?;
            fold(acc, x)
        })
    }

    #[inline]
    fn fold<B, Fold>(mut self, init: B, mut fold: Fold) -> Result<B, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.fold(init, move |acc, x| {
            (self.f)(&x)?;
            fold(acc, x)
        })
    }
}

impl<L, F> DoubleEndedFallibleLender for Inspect<L, F>
where
    L: DoubleEndedFallibleLender,
    F: FnMut(&FallibleLend<'_, L>) -> Result<(), L::Error>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let next = self.lender.next_back()?;
        if let Some(ref x) = next {
            (self.f)(x)?;
        }
        Ok(next)
    }

    #[inline]
    fn try_rfold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> Result<R, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: crate::try_trait_v2::Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender.try_rfold(init, move |acc, x| {
            (f)(&x)?;
            fold(acc, x)
        })
    }

    #[inline]
    fn rfold<B, Fold>(mut self, init: B, mut fold: Fold) -> Result<B, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.rfold(init, move |acc, x| {
            (self.f)(&x)?;
            fold(acc, x)
        })
    }
}

impl<L: ExactSizeFallibleLender, F> ExactSizeFallibleLender for Inspect<L, F>
where
    F: FnMut(&FallibleLend<'_, L>) -> Result<(), L::Error>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
    }
}

impl<L: FusedFallibleLender, F> FusedFallibleLender for Inspect<L, F> where
    F: FnMut(&FallibleLend<'_, L>) -> Result<(), L::Error>
{
}
