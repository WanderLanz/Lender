use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, Zip,
};

impl<A: FallibleLender, B: FallibleLender> Zip<A, B> {
    #[inline]
    pub(crate) fn new_fallible(a: A, b: B) -> Self {
        crate::__check_fallible_lender_covariance::<A>();
        crate::__check_fallible_lender_covariance::<B>();
        Self { a, b }
    }
}

impl<'lend, A, B> FallibleLending<'lend> for Zip<A, B>
where
    A: FallibleLender,
    B: FallibleLender<Error = A::Error>,
{
    type Lend = (FallibleLend<'lend, A>, FallibleLend<'lend, B>);
}

impl<A, B> FallibleLender for Zip<A, B>
where
    A: FallibleLender,
    B: FallibleLender<Error = A::Error>,
{
    type Error = A::Error;
    // SAFETY: the lend is a tuple of the lends of A and B
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let Some(value_a) = self.a.next()? else {
            return Ok(None);
        };
        let Some(value_b) = self.b.next()? else {
            return Ok(None);
        };
        Ok(Some((value_a, value_b)))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();

        let lower = core::cmp::min(a_lower, b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => Some(core::cmp::min(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        };

        (lower, upper)
    }
}

impl<A, B> DoubleEndedFallibleLender for Zip<A, B>
where
    A: DoubleEndedFallibleLender + ExactSizeFallibleLender,
    B: DoubleEndedFallibleLender<Error = A::Error> + ExactSizeFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let a_sz = self.a.len();
        let b_sz = self.b.len();
        if a_sz > b_sz {
            self.a.advance_back_by(a_sz - b_sz)?.ok();
        } else if b_sz > a_sz {
            self.b.advance_back_by(b_sz - a_sz)?.ok();
        }
        match (self.a.next_back()?, self.b.next_back()?) {
            (Some(x), Some(y)) => Ok(Some((x, y))),
            (None, None) => Ok(None),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let a_sz = self.a.len();
        let b_sz = self.b.len();
        if a_sz > b_sz {
            self.a.advance_back_by(a_sz - b_sz)?.ok();
        } else if b_sz > a_sz {
            self.b.advance_back_by(b_sz - a_sz)?.ok();
        }
        match (self.a.nth_back(n)?, self.b.nth_back(n)?) {
            (Some(x), Some(y)) => Ok(Some((x, y))),
            (None, None) => Ok(None),
            _ => unreachable!(),
        }
    }
}

impl<A, B> ExactSizeFallibleLender for Zip<A, B>
where
    A: ExactSizeFallibleLender,
    B: ExactSizeFallibleLender<Error = A::Error>,
{
}

impl<A, B> FusedFallibleLender for Zip<A, B>
where
    A: FusedFallibleLender,
    B: FusedFallibleLender<Error = A::Error>,
{
}
