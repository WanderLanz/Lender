use core::num::NonZeroUsize;

use crate::{
    DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending, try_trait_v2::Try,
};

/// A lender that yields the current count and the element during iteration.
///
/// This `struct` is created by the [`enumerate()`](crate::Lender::enumerate)
/// method on [`Lender`].
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Enumerate<L> {
    pub(crate) lender: L,
    pub(crate) count: usize,
}

impl<L> Enumerate<L> {
    /// Returns the inner lender.
    #[inline]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the current count.
    #[inline]
    pub fn into_parts(self) -> (L, usize) {
        (self.lender, self.count)
    }
}

impl<L: Lender> Enumerate<L> {
    #[inline]
    pub(crate) fn new(lender: L) -> Enumerate<L> {
        crate::__check_lender_covariance::<L>();
        Enumerate { lender, count: 0 }
    }
}

impl<'lend, L> Lending<'lend> for Enumerate<L>
where
    L: Lender,
{
    type Lend = (usize, Lend<'lend, L>);
}

impl<L> Lender for Enumerate<L>
where
    L: Lender,
{
    // SAFETY: the lend is a pair of usize and the lend of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next().map(|x| {
            let count = self.count;
            self.count += 1;
            (count, x)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        let a = self.lender.nth(n)?;
        // May overflow on very large indices; matches std::iter::Enumerate.
        let i = self.count + n;
        self.count = i + 1;
        Some((i, a))
    }

    #[inline]
    fn count(self) -> usize {
        self.lender.count()
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
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
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut count = self.count;
        self.lender.fold(init, |acc, x| {
            let elt = (count, x);
            count += 1;
            f(acc, elt)
        })
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        let remaining = self.lender.advance_by(n);
        let advanced = match remaining {
            Ok(()) => n,
            Err(rem) => n - rem.get(),
        };
        self.count += advanced;
        remaining
    }
}

impl<L> DoubleEndedLender for Enumerate<L>
where
    L: ExactSizeLender + DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        let len = self.lender.len();
        let x = self.lender.next_back()?;
        Some((self.count + len - 1, x))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        let len = self.lender.len();
        let x = self.lender.nth_back(n)?;
        Some((self.count + len - n - 1, x))
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut count = self.count + self.lender.len();
        self.lender.try_rfold(init, move |acc, x| {
            count -= 1;
            f(acc, (count, x))
        })
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut count = self.count + self.lender.len();
        self.lender.rfold(init, move |acc, x| {
            count -= 1;
            f(acc, (count, x))
        })
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        self.lender.advance_back_by(n)
    }
}

impl<L> ExactSizeLender for Enumerate<L>
where
    L: ExactSizeLender,
{
    #[inline]
    fn len(&self) -> usize {
        self.lender.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
    }
}

impl<L> FusedLender for Enumerate<L> where L: FusedLender {}

impl<L: Default + Lender> Default for Enumerate<L> {
    #[inline]
    fn default() -> Self {
        Enumerate::new(Default::default())
    }
}
