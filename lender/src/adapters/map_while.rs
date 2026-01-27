use core::fmt;

use crate::{
    higher_order::{FnMutHKAOpt, FnMutHKAResOpt},
    FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending,
};
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct MapWhile<L, P> {
    lender: L,
    predicate: P,
}
impl<L, P> MapWhile<L, P> {
    pub(crate) fn new(lender: L, predicate: P) -> MapWhile<L, P> {
        MapWhile { lender, predicate }
    }
    pub fn into_inner(self) -> L {
        self.lender
    }
    pub fn into_parts(self) -> (L, P) {
        (self.lender, self.predicate)
    }
}
impl<L: fmt::Debug, P> fmt::Debug for MapWhile<L, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapWhile").field("lender", &self.lender).finish()
    }
}
impl<'lend, B, L, P> Lending<'lend> for MapWhile<L, P>
where
    P: FnMut(Lend<'lend, L>) -> Option<B>,
    L: Lender,
    B: 'lend,
{
    type Lend = B;
}
impl<L, P> Lender for MapWhile<L, P>
where
    P: for<'all> FnMutHKAOpt<'all, Lend<'all, L>>,
    L: Lender,
{
    // SAFETY: The Lend type is the closure's return type. Rust cannot verify covariance
    // of associated types from higher-order trait bounds at compile time. Users must
    // ensure P returns a covariant type (e.g., by using hrc!() macros). Returning an
    // non-covariant type (like &'lend Cell<&'lend T>) is undefined behavior.
    unsafe fn _check_covariance<'long: 'short, 'short>(
        lend: *const &'short <Self as Lending<'long>>::Lend,
        _: crate::Uncallable,
    ) -> *const &'short <Self as Lending<'short>>::Lend {
        unsafe { core::mem::transmute(lend) }
    }
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        (self.predicate)(self.lender.next()?)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}

impl<'lend, B, L, P> FallibleLending<'lend> for MapWhile<L, P>
where
    P: FnMut(FallibleLend<'lend, L>) -> Result<Option<B>, L::Error>,
    L: FallibleLender,
    B: 'lend,
{
    type Lend = B;
}
impl<L, P> FallibleLender for MapWhile<L, P>
where
    P: for<'all> FnMutHKAResOpt<'all, FallibleLend<'all, L>, L::Error>,
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: The Lend type is the closure's return type. Rust cannot verify covariance
    // of associated types from higher-order trait bounds at compile time. Users must
    // ensure P returns a covariant type (e.g., by using hrc!() macros). Returning an
    // non-covariant type (like &'lend Cell<&'lend T>) is undefined behavior.
    unsafe fn _check_covariance<'long: 'short, 'short>(
        lend: *const &'short <Self as FallibleLending<'long>>::Lend,
        _: crate::Uncallable,
    ) -> *const &'short <Self as FallibleLending<'short>>::Lend {
        unsafe { core::mem::transmute(lend) }
    }

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.lender.next()? {
            Some(next) => (self.predicate)(next),
            None => Ok(None),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}
