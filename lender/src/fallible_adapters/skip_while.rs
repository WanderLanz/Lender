use core::ops::ControlFlow;

use crate::{
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, SkipWhile,
    try_trait_v2::Try,
};

impl<L: FallibleLender, P> SkipWhile<L, P> {
    #[inline(always)]
    pub(crate) fn new_fallible(lender: L, predicate: P) -> SkipWhile<L, P> {
        crate::__check_fallible_lender_covariance::<L>();
        SkipWhile {
            lender,
            flag: false,
            predicate,
        }
    }
}

impl<'lend, L, P> FallibleLending<'lend> for SkipWhile<L, P>
where
    P: FnMut(&FallibleLend<'lend, L>) -> Result<bool, L::Error>,
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L, P> FallibleLender for SkipWhile<L, P>
where
    P: FnMut(&FallibleLend<'_, L>) -> Result<bool, L::Error>,
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let flag = &mut self.flag;
        let predicate = &mut self.predicate;
        self.lender.find(move |x| {
            if *flag || !(predicate)(x)? {
                *flag = true;
                Ok(true)
            } else {
                Ok(false)
            }
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, mut init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        if !self.flag {
            match self.next()? {
                Some(x) => {
                    init = match f(init, x)?.branch() {
                        ControlFlow::Break(x) => return Ok(R::from_residual(x)),
                        ControlFlow::Continue(x) => x,
                    }
                }
                None => return Ok(R::from_output(init)),
            }
        }
        self.lender.try_fold(init, f)
    }

    #[inline]
    fn fold<B, F>(mut self, mut init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        if !self.flag {
            match self.next()? {
                Some(x) => init = f(init, x)?,
                None => return Ok(init),
            }
        }
        self.lender.fold(init, f)
    }
}

impl<L, P> FusedFallibleLender for SkipWhile<L, P>
where
    P: FnMut(&FallibleLend<'_, L>) -> Result<bool, L::Error>,
    L: FusedFallibleLender,
{
}
