use core::{num::NonZeroUsize, ops::ControlFlow};

use crate::{
    try_trait_v2::{FromResidual, Try, internal::NeverShortCircuit},
    *,
};

/// The [`Lender`] version of [`core::iter::DoubleEndedIterator`].
pub trait DoubleEndedLender: Lender {
    fn next_back(&mut self) -> Option<Lend<'_, Self>>;

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        for i in 0..n {
            if self.next_back().is_none() {
                // SAFETY: `i` is always less than `n`.
                return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
            }
        }
        Ok(())
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        if self.advance_back_by(n).is_err() {
            return None;
        }
        self.next_back()
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut accum = init;
        while let Some(x) = self.next_back() {
            accum = match f(accum, x).branch() {
                ControlFlow::Break(x) => return FromResidual::from_residual(x),
                ControlFlow::Continue(x) => x,
            };
        }
        Try::from_output(accum)
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut accum = init;
        while let Some(x) = self.next_back() {
            accum = f(accum, x);
        }
        accum
    }

    #[inline]
    fn rfind<P>(&mut self, mut predicate: P) -> Option<Lend<'_, Self>>
    where
        Self: Sized,
        P: FnMut(&Lend<'_, Self>) -> bool,
    {
        while let Some(x) = self.next_back() {
            if predicate(&x) {
                // SAFETY: polonius return
                return Some(unsafe { core::mem::transmute::<Lend<'_, Self>, Lend<'_, Self>>(x) });
            }
        }
        None
    }
}

impl<L: DoubleEndedLender> DoubleEndedLender for &mut L {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        (**self).next_back()
    }

    #[inline(always)]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        (**self).advance_back_by(n)
    }

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        (**self).nth_back(n)
    }
}

pub trait DoubleEndedFallibleLender: FallibleLender {
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>;

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        for i in 0..n {
            if self.next_back()?.is_none() {
                // SAFETY: `i` is always less than `n`.
                return Ok(Err(unsafe { NonZeroUsize::new_unchecked(n - i) }));
            }
        }
        Ok(Ok(()))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.advance_back_by(n)?.is_err() {
            return Ok(None);
        }
        self.next_back()
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, mut init: B, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        while let Some(v) = self.next_back()? {
            match f(init, v)?.branch() {
                ControlFlow::Break(residual) => return Ok(R::from_residual(residual)),
                ControlFlow::Continue(output) => init = output,
            }
        }
        Ok(R::from_output(init))
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.try_rfold(init, |acc, item| f(acc, item).map(NeverShortCircuit))
            .map(|res| res.0)
    }

    #[inline]
    fn rfind<P>(&mut self, mut predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        while let Some(x) = self.next_back()? {
            if predicate(&x)? {
                // SAFETY: polonius return
                return Ok(Some(unsafe {
                    core::mem::transmute::<FallibleLend<'_, Self>, FallibleLend<'_, Self>>(x)
                }));
            }
        }
        Ok(None)
    }
}

impl<L: DoubleEndedFallibleLender> DoubleEndedFallibleLender for &mut L {
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        (**self).next_back()
    }

    #[inline(always)]
    fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        (**self).advance_back_by(n)
    }

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        (**self).nth_back(n)
    }
}
