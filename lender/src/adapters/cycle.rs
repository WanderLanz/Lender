use core::{num::NonZeroUsize, ops::ControlFlow};

use crate::{
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender,
    Lending,
    try_trait_v2::{FromResidual, Try},
};

#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Cycle<L> {
    orig: L,
    lender: L,
}
impl<L> Cycle<L>
where
    L: Clone,
{
    pub(crate) fn new(lender: L) -> Cycle<L> {
        Cycle {
            orig: lender.clone(),
            lender,
        }
    }

    pub fn into_inner(self) -> (L, L) {
        (self.orig, self.lender)
    }
}

impl<'lend, L> Lending<'lend> for Cycle<L>
where
    L: Clone + Lender,
{
    type Lend = Lend<'lend, L>;
}

impl<L> Lender for Cycle<L>
where
    L: Clone + Lender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: polonius return
        let reborrow = unsafe { &mut *(self as *mut Self) };
        if let x @ Some(_) = reborrow.lender.next() {
            return x;
        }
        self.lender = self.orig.clone();
        self.lender.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.orig.size_hint() {
            h @ (0, Some(0)) => h,
            (0, _) => (0, None),
            _ => (usize::MAX, None),
        }
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut acc = init;
        acc = match self.lender.try_fold(acc, &mut f).branch() {
            ControlFlow::Break(x) => return FromResidual::from_residual(x),
            ControlFlow::Continue(x) => x,
        };
        self.lender = self.orig.clone();
        let mut empty = true;
        acc = match self
            .orig
            .try_fold(acc, |acc, x| {
                empty = false;
                f(acc, x)
            })
            .branch()
        {
            ControlFlow::Break(x) => return FromResidual::from_residual(x),
            ControlFlow::Continue(x) => x,
        };
        if empty {
            return Try::from_output(acc);
        }
        loop {
            self.lender = self.orig.clone();
            acc = match self.orig.try_fold(acc, &mut f).branch() {
                ControlFlow::Break(x) => return FromResidual::from_residual(x),
                ControlFlow::Continue(x) => x,
            };
        }
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        let mut n = match self.lender.advance_by(n) {
            Ok(()) => return Ok(()),
            Err(rem) => rem.get(),
        };

        while n > 0 {
            self.lender = self.orig.clone();
            n = match self.lender.advance_by(n) {
                Ok(()) => return Ok(()),
                e @ Err(rem) if rem.get() == n => return e,
                Err(rem) => rem.get(),
            };
        }

        NonZeroUsize::new(n).map_or(Ok(()), Err)
    }
}

impl<L> FusedLender for Cycle<L> where L: Clone + FusedLender {}

impl<'lend, L> FallibleLending<'lend> for Cycle<L>
where
    L: Clone + FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L> FallibleLender for Cycle<L>
where
    L: Clone + FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        // SAFETY: polonius return
        let reborrow = unsafe { &mut *(self as *mut Self) };
        if let x @ Some(_) = reborrow.lender.next()? {
            return Ok(x);
        }
        self.lender = self.orig.clone();
        self.lender.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.orig.size_hint() {
            h @ (0, Some(0)) => h,
            (0, _) => (0, None),
            _ => (usize::MAX, None),
        }
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let mut acc = init;
        acc = match self.lender.try_fold(acc, &mut f)?.branch() {
            ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
            ControlFlow::Continue(x) => x,
        };
        self.lender = self.orig.clone();
        let mut empty = true;
        acc = match self
            .orig
            .try_fold(acc, |acc, x| {
                empty = false;
                f(acc, x)
            })?
            .branch()
        {
            ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
            ControlFlow::Continue(x) => x,
        };
        if empty {
            return Ok(Try::from_output(acc));
        }
        loop {
            self.lender = self.orig.clone();
            acc = match self.orig.try_fold(acc, &mut f)?.branch() {
                ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
                ControlFlow::Continue(x) => x,
            };
        }
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        let mut n = match self.lender.advance_by(n)? {
            Ok(()) => return Ok(Ok(())),
            Err(rem) => rem.get(),
        };

        while n > 0 {
            self.lender = self.orig.clone();
            n = match self.lender.advance_by(n)? {
                Ok(()) => return Ok(Ok(())),
                Err(rem) if rem.get() == n => return Ok(Err(rem)),
                Err(rem) => rem.get(),
            };
        }

        Ok(Ok(()))
    }
}

impl<L> FusedFallibleLender for Cycle<L> where L: Clone + FusedFallibleLender {}
