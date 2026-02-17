use core::num::NonZeroUsize;

use crate::{
    DoubleEndedFallibleLender, Enumerate, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, try_trait_v2::Try,
};

impl<L: FallibleLender> Enumerate<L> {
    #[inline(always)]
    pub(crate) fn new_fallible(lender: L) -> Enumerate<L> {
        let _ = L::__check_covariance(crate::CovariantProof::new());
        Enumerate { lender, count: 0 }
    }
}

impl<'lend, L> FallibleLending<'lend> for Enumerate<L>
where
    L: FallibleLender,
{
    type Lend = (usize, FallibleLend<'lend, L>);
}

impl<L> FallibleLender for Enumerate<L>
where
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is a pair of usize and the lend of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(self.lender.next()?.map(|x| {
            let count = self.count;
            self.count += 1;
            (count, x)
        }))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let a = match self.lender.nth(n)? {
            Some(a) => a,
            None => return Ok(None),
        };
        let i = self.count + n;
        self.count = i + 1;
        Ok(Some((i, a)))
    }

    #[inline(always)]
    fn count(self) -> Result<usize, Self::Error> {
        self.lender.count()
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let count = &mut self.count;
        self.lender.try_fold(init, |acc, x| {
            let elt = (*count, x);
            *count += 1;
            f(acc, elt)
        })
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut count = self.count;
        self.lender.fold(init, |acc, x| {
            let elt = (count, x);
            count += 1;
            f(acc, elt)
        })
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        let remaining = self.lender.advance_by(n)?;
        let advanced = match remaining {
            Ok(()) => n,
            Err(rem) => n - rem.get(),
        };
        self.count += advanced;
        Ok(remaining)
    }
}

impl<L> DoubleEndedFallibleLender for Enumerate<L>
where
    L: ExactSizeFallibleLender + DoubleEndedFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let len = self.lender.len();
        let x = match self.lender.next_back()? {
            Some(x) => x,
            None => return Ok(None),
        };
        Ok(Some((self.count + len - 1, x)))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let len = self.lender.len();
        let x = match self.lender.nth_back(n)? {
            Some(x) => x,
            None => return Ok(None),
        };
        Ok(Some((self.count + len - n - 1, x)))
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let mut count = self.count + self.lender.len();
        self.lender.try_rfold(init, move |acc, x| {
            count -= 1;
            f(acc, (count, x))
        })
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut count = self.count + self.lender.len();
        self.lender.rfold(init, move |acc, x| {
            count -= 1;
            f(acc, (count, x))
        })
    }

    #[inline(always)]
    fn advance_back_by(
        &mut self,
        n: usize,
    ) -> Result<Result<(), core::num::NonZeroUsize>, Self::Error> {
        self.lender.advance_back_by(n)
    }
}

impl<L> ExactSizeFallibleLender for Enumerate<L>
where
    L: ExactSizeFallibleLender,
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

impl<L> FusedFallibleLender for Enumerate<L> where L: FusedFallibleLender {}
