use core::ops::ControlFlow;

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, Fuse, FusedFallibleLender,
    try_trait_v2::{FromResidual, Try},
};

impl<'lend, L> FallibleLending<'lend> for Fuse<L>
where
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L> FallibleLender for Fuse<L>
where
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.flag {
            Ok(None)
        } else {
            match self.lender.next()? {
                Some(next) => Ok(Some(next)),
                None => {
                    self.flag = true;
                    Ok(None)
                }
            }
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.flag {
            Ok(None)
        } else {
            match self.lender.nth(n)? {
                Some(value) => Ok(Some(value)),
                None => {
                    self.flag = true;
                    Ok(None)
                }
            }
        }
    }

    #[inline]
    fn last(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
    {
        if self.flag {
            Ok(None)
        } else {
            match self.lender.last()? {
                x @ Some(_) => Ok(x),
                None => {
                    self.flag = true;
                    Ok(None)
                }
            }
        }
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error> {
        if !self.flag {
            self.lender.count()
        } else {
            Ok(0)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if !self.flag {
            self.lender.size_hint()
        } else {
            (0, Some(0))
        }
    }

    #[inline]
    fn try_fold<Acc, F, R>(&mut self, mut acc: Acc, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(Acc, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = Acc>,
    {
        if !self.flag {
            acc = match self.lender.try_fold(acc, &mut f)?.branch() {
                ControlFlow::Continue(x) => x,
                ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
            };
            self.flag = true;
        }
        Ok(Try::from_output(acc))
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        if !self.flag {
            acc = self.lender.fold(acc, &mut f)?;
            self.flag = true;
        }
        Ok(acc)
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        if !self.flag {
            if let x @ Some(_) = self.lender.find(&mut predicate)? {
                return Ok(x);
            }
            self.flag = true;
        }
        Ok(None)
    }
}

impl<L> DoubleEndedFallibleLender for Fuse<L>
where
    L: DoubleEndedFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if !self.flag {
            if let x @ Some(_) = self.lender.next_back()? {
                return Ok(x);
            }
            self.flag = true;
        }
        Ok(None)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if !self.flag {
            if let x @ Some(_) = self.lender.nth_back(n)? {
                return Ok(x);
            }
            self.flag = true;
        }
        Ok(None)
    }

    #[inline]
    fn try_rfold<Acc, F, R>(&mut self, mut acc: Acc, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(Acc, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = Acc>,
    {
        if !self.flag {
            acc = match self.lender.try_rfold(acc, &mut f)?.branch() {
                ControlFlow::Continue(x) => x,
                ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
            };
            self.flag = true;
        }
        Ok(Try::from_output(acc))
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        if !self.flag {
            acc = self.lender.rfold(acc, &mut f)?;
            self.flag = true;
        }
        Ok(acc)
    }

    #[inline]
    fn rfind<P>(&mut self, mut predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        if !self.flag {
            if let x @ Some(_) = self.lender.rfind(&mut predicate)? {
                return Ok(x);
            }
            self.flag = true;
        }
        Ok(None)
    }
}

impl<L> ExactSizeFallibleLender for Fuse<L>
where
    L: ExactSizeFallibleLender,
{
    #[inline(always)]
    fn len(&self) -> usize {
        if self.flag { 0 } else { self.lender.len() }
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.flag || self.lender.is_empty()
    }
}

impl<L> FusedFallibleLender for Fuse<L> where L: FallibleLender {}
