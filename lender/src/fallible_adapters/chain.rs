use core::{num::NonZeroUsize, ops::ControlFlow};

use crate::{
    try_trait_v2::{FromResidual, Try},
    Chain, DoubleEndedFallibleLender, FallibleLend, FallibleLender, FallibleLending,
    FusedFallibleLender,
};

impl<'lend, A, B> FallibleLending<'lend> for Chain<A, B>
where
    A: FallibleLender,
    B: FallibleLender<Error = A::Error>
        + for<'all> FallibleLending<'all, Lend = FallibleLend<'all, A>>,
{
    type Lend = FallibleLend<'lend, A>;
}

impl<A, B> FallibleLender for Chain<A, B>
where
    A: FallibleLender,
    B: FallibleLender<Error = A::Error>
        + for<'all> FallibleLending<'all, Lend = FallibleLend<'all, A>>,
{
    type Error = A::Error;
    // SAFETY: the lend is that of A (and B has the same lend type)
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.a.next()? {
            Some(next) => Ok(Some(next)),
            None => self.b.next(),
        }
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error> {
        Ok(self.a.count()? + self.b.count()?)
    }

    #[inline]
    fn try_fold<Acc, F, R>(&mut self, mut acc: Acc, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(Acc, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = Acc>,
    {
        acc = match self.a.try_fold(acc, &mut f)?.branch() {
            ControlFlow::Continue(x) => x,
            ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
        };
        acc = match self.b.try_fold(acc, f)?.branch() {
            ControlFlow::Continue(x) => x,
            ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
        };
        Ok(Try::from_output(acc))
    }

    #[inline]
    fn fold<Acc, F>(self, mut acc: Acc, mut f: F) -> Result<Acc, Self::Error>
    where
        F: FnMut(Acc, FallibleLend<'_, Self>) -> Result<Acc, Self::Error>,
    {
        acc = self.a.fold(acc, &mut f)?;
        acc = self.b.fold(acc, f)?;
        Ok(acc)
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        match self.a.advance_by(n)? {
            Ok(()) => Ok(Ok(())),
            Err(k) => self.b.advance_by(k.get()),
        }
    }

    #[inline]
    fn nth(&mut self, mut n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        n = match self.a.advance_by(n)? {
            Ok(()) => match self.a.next()? {
                None => 0,
                x => return Ok(x),
            },
            Err(k) => k.get(),
        };
        self.b.nth(n)
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        match self.a.find(&mut predicate)? {
            Some(value) => Ok(Some(value)),
            None => self.b.find(predicate),
        }
    }

    #[inline]
    fn last(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
    {
        let a_last = self.a.last()?;
        let b_last = self.b.last()?;
        Ok(b_last.or(a_last))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();
        let lower = a_lower.saturating_add(b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => x.checked_add(y),
            _ => None,
        };
        (lower, upper)
    }
}

impl<A, B> DoubleEndedFallibleLender for Chain<A, B>
where
    A: DoubleEndedFallibleLender,
    B: DoubleEndedFallibleLender<Error = A::Error>
        + for<'all> FallibleLending<'all, Lend = FallibleLend<'all, A>>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.b.next_back()? {
            Some(next) => Ok(Some(next)),
            None => self.a.next_back(),
        }
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        match self.b.advance_back_by(n)? {
            Ok(()) => Ok(Ok(())),
            Err(k) => self.a.advance_back_by(k.get()),
        }
    }

    #[inline]
    fn nth_back(&mut self, mut n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        n = match self.b.advance_back_by(n)? {
            Ok(()) => match self.b.next_back()? {
                None => 0,
                x => return Ok(x),
            },
            Err(k) => k.get(),
        };
        self.a.nth_back(n)
    }

    #[inline]
    fn rfind<P>(&mut self, mut predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        match self.b.rfind(&mut predicate)? {
            Some(value) => Ok(Some(value)),
            None => self.a.rfind(predicate),
        }
    }

    fn try_rfold<Acc, F, R>(&mut self, init: Acc, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(Acc, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = Acc>,
    {
        let mut acc = match self.b.try_rfold(init, &mut f)?.branch() {
            ControlFlow::Continue(x) => x,
            ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
        };
        acc = match self.a.try_rfold(acc, f)?.branch() {
            ControlFlow::Continue(x) => x,
            ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
        };
        Ok(Try::from_output(acc))
    }

    fn rfold<Acc, F>(self, init: Acc, mut f: F) -> Result<Acc, Self::Error>
    where
        Self: Sized,
        F: FnMut(Acc, FallibleLend<'_, Self>) -> Result<Acc, Self::Error>,
    {
        let mut acc = self.b.rfold(init, &mut f)?;
        acc = self.a.rfold(acc, f)?;
        Ok(acc)
    }
}

impl<A, B> FusedFallibleLender for Chain<A, B>
where
    A: FusedFallibleLender,
    B: FusedFallibleLender<Error = A::Error>
        + for<'all> FallibleLending<'all, Lend = FallibleLend<'all, A>>,
{
}
