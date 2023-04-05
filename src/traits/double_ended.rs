use core::{num::NonZeroUsize, ops::ControlFlow};

use crate::{
    try_trait_v2::{FromResidual, Try},
    *,
};
pub trait DoubleEndedLender: Lender {
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend>;
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
    fn nth_back(&mut self, n: usize) -> Option<<Self as Lending<'_>>::Lend> {
        if self.advance_back_by(n).is_err() {
            return None;
        }
        self.next_back()
    }
    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> R,
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
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        let mut accum = init;
        while let Some(x) = self.next_back() {
            accum = f(accum, x);
        }
        accum
    }
    #[inline]
    fn rfind<'call, P>(&'call mut self, mut predicate: P) -> Option<<Self as Lending<'call>>::Lend>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        while let Some(x) = self.next_back() {
            if predicate(&x) {
                // SAFETY: polonius
                return Some(unsafe {
                    core::mem::transmute::<<Self as Lending<'_>>::Lend, <Self as Lending<'call>>::Lend>(x)
                });
            }
        }
        None
    }
}

impl<'a, L: DoubleEndedLender> DoubleEndedLender for &'a mut L {
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> { (**self).next_back() }
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> { (**self).advance_back_by(n) }
    fn nth_back(&mut self, n: usize) -> Option<<Self as Lending<'_>>::Lend> { (**self).nth_back(n) }
}
