use core::fmt;

use crate::{DoubleEndedLender, FusedLender, Lend, Lender, Lending, higher_order::FnMutHKAOpt};

/// A lender that uses a closure to optionally produce elements.
///
/// This `struct` is created by the [`filter_map()`](crate::Lender::filter_map) method on [`Lender`].
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FilterMap<L, F> {
    pub(crate) lender: L,
    pub(crate) f: F,
}

impl<L, F> FilterMap<L, F> {
    #[inline(always)]
    pub(crate) fn new(lender: L, f: F) -> FilterMap<L, F> {
        FilterMap { lender, f }
    }

    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the mapping function.
    #[inline(always)]
    pub fn into_parts(self) -> (L, F) {
        (self.lender, self.f)
    }
}

impl<L: fmt::Debug, F> fmt::Debug for FilterMap<L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterMap")
            .field("lender", &self.lender)
            .finish()
    }
}

impl<'lend, B, L, F> Lending<'lend> for FilterMap<L, F>
where
    F: FnMut(Lend<'lend, L>) -> Option<B>,
    B: 'lend,
    L: Lender,
{
    type Lend = B;
}

impl<L, F> Lender for FilterMap<L, F>
where
    for<'all> F: FnMutHKAOpt<'all, Lend<'all, L>>,
    L: Lender,
{
    // SAFETY: the lend is the return type of F
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        while let Some(x) = self.lender.next() {
            if let Some(y) = (self.f)(x) {
                // SAFETY: polonius return
                return Some(unsafe { core::mem::transmute::<Lend<'_, Self>, Lend<'_, Self>>(y) });
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}

impl<L, F> DoubleEndedLender for FilterMap<L, F>
where
    for<'all> F: FnMutHKAOpt<'all, Lend<'all, L>>,
    L: DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        while let Some(x) = self.lender.next_back() {
            if let Some(y) = (self.f)(x) {
                // SAFETY: polonius return
                return Some(unsafe { core::mem::transmute::<Lend<'_, Self>, Lend<'_, Self>>(y) });
            }
        }
        None
    }
}

impl<L, F> FusedLender for FilterMap<L, F>
where
    for<'all> F: FnMutHKAOpt<'all, Lend<'all, L>>,
    L: FusedLender,
{
}
