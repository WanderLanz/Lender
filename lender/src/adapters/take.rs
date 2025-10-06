use core::num::NonZeroUsize;

use crate::{try_trait_v2::Try, DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending};
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Take<L> {
    lender: L,
    n: usize,
}
impl<L> Take<L> {
    pub(crate) fn new(lender: L, n: usize) -> Take<L> { Take { lender, n } }
    pub fn into_inner(self) -> L { self.lender }
    pub fn into_parts(self) -> (L, usize) { (self.lender, self.n) }
}
impl<'lend, L> Lending<'lend> for Take<L>
where
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}
impl<L> Lender for Take<L>
where
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.n != 0 {
            self.n -= 1;
            self.lender.next()
        } else {
            None
        }
    }
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        if self.n > n {
            self.n -= n + 1;
            self.lender.nth(n)
        } else {
            if self.n > 0 {
                self.lender.nth(self.n - 1);
                self.n = 0;
            }
            None
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.n == 0 {
            return (0, Some(0));
        }

        let (lower, upper) = self.lender.size_hint();

        let lower = lower.min(self.n);

        let upper = match upper {
            Some(x) if x < self.n => Some(x),
            _ => Some(self.n),
        };

        (lower, upper)
    }
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        let min = self.n.min(n);
        let rem = match self.lender.advance_by(min) {
            Ok(()) => 0,
            Err(rem) => rem.get(),
        };
        let advanced = min - rem;
        self.n -= advanced;
        NonZeroUsize::new(n - advanced).map_or(Ok(()), Err)
    }
}
impl<L> DoubleEndedLender for Take<L>
where
    L: DoubleEndedLender + ExactSizeLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        if self.n != 0 {
            let n = self.n;
            self.n -= 1;
            self.lender.nth_back(self.lender.len().saturating_sub(n))
        } else {
            None
        }
    }
    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        let len = self.lender.len();
        if self.n > n {
            let m = len.saturating_sub(self.n) + n;
            self.n -= n + 1;
            self.lender.nth_back(m)
        } else {
            if len > 0 {
                self.lender.nth_back(len - 1);
            }
            None
        }
    }
    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        if self.n == 0 {
            R::from_output(init)
        } else {
            let len = self.lender.len();
            if len > self.n && self.lender.nth_back(len - self.n - 1).is_none() {
                R::from_output(init)
            } else {
                self.lender.try_rfold(init, f)
            }
        }
    }
    #[inline]
    fn rfold<B, F>(mut self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        if self.n == 0 {
            init
        } else {
            let len = self.lender.len();
            if len > self.n && self.lender.nth_back(len - self.n - 1).is_none() {
                init
            } else {
                self.lender.rfold(init, f)
            }
        }
    }
    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        let trim_inner = self.lender.len().saturating_sub(self.n);
        let advance_by = trim_inner.saturating_add(n);
        let remainder = match self.lender.advance_back_by(advance_by) {
            Ok(()) => 0,
            Err(rem) => rem.get(),
        };
        let advanced_by_inner = advance_by - remainder;
        let advanced_by = advanced_by_inner - trim_inner;
        self.n -= advanced_by;
        NonZeroUsize::new(n - advanced_by).map_or(Ok(()), Err)
    }
}
impl<L> ExactSizeLender for Take<L> where L: ExactSizeLender {}
impl<L> FusedLender for Take<L> where L: FusedLender {}
