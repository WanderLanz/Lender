use core::{num::NonZeroUsize, ops::ControlFlow};

use crate::{
    try_trait_v2::{FromResidual, Try},
    *,
};

/// Documentation is incomplete. Refer to [`core::iter::DoubleEndedIterator`] for more information
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
    fn next_back(&mut self) -> Option<Lend<'_, Self>> { (**self).next_back() }
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> { (**self).advance_back_by(n) }
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> { (**self).nth_back(n) }
}
