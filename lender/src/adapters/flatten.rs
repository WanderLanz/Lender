use core::fmt;
use alloc::boxed::Box;

use crate::{FusedLender, IntoLender, Lend, Lender, Lending, Map};
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Flatten<'this, L: Lender>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    inner: FlattenCompat<'this, L>,
}
impl<L: Lender> Flatten<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    pub(crate) fn new(lender: L) -> Self {
        Self { inner: FlattenCompat::new(lender) }
    }
    pub fn into_inner(self) -> L {
        *self.inner.lender
    }
}
impl<L: Lender + Clone> Clone for Flatten<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
    for<'all> <Lend<'all, L> as IntoLender>::Lender: Clone,
{
    fn clone(&self) -> Self {
        Flatten { inner: self.inner.clone() }
    }
}
impl<L: Lender + fmt::Debug> fmt::Debug for Flatten<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
    for<'all> <Lend<'all, L> as IntoLender>::Lender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flatten").field("inner", &self.inner).finish()
    }
}
impl<'lend, 'this, L: Lender> Lending<'lend> for Flatten<'this, L>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    type Lend = Lend<'lend, <Lend<'this, L> as IntoLender>::Lender>;
}
impl<L: Lender> Lender for Flatten<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl<L: FusedLender> FusedLender for Flatten<'_, L> where for<'all> Lend<'all, L>: IntoLender {}

#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FlatMap<'this, L: Lender, F>
where
    Map<L, F>: Lender,
    for<'all> Lend<'all, Map<L, F>>: IntoLender,
{
    inner: FlattenCompat<'this, Map<L, F>>,
}
impl<L: Lender, F> FlatMap<'_, L, F>
where
    Map<L, F>: Lender,
    for<'all> Lend<'all, Map<L, F>>: IntoLender,
{
    pub(crate) fn new(lender: L, f: F) -> Self {
        Self { inner: FlattenCompat::new(Map::new(lender, f)) }
    }
}
impl<L: Lender + Clone, F: Clone> Clone for FlatMap<'_, L, F>
where
    Map<L, F>: Lender + Clone,
    for<'all> Lend<'all, Map<L, F>>: IntoLender,
    for<'all> <Lend<'all, Map<L, F>> as IntoLender>::Lender: Clone,
{
    fn clone(&self) -> Self {
        FlatMap { inner: self.inner.clone() }
    }
}
impl<L: Lender + fmt::Debug, F> fmt::Debug for FlatMap<'_, L, F>
where
    Map<L, F>: Lender + Clone,
    for<'all> Lend<'all, Map<L, F>>: IntoLender,
    for<'all> <Lend<'all, Map<L, F>> as IntoLender>::Lender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlatMap").field("inner", &self.inner).finish()
    }
}
impl<'lend, 'this, L: Lender, F> Lending<'lend> for FlatMap<'this, L, F>
where
    Map<L, F>: Lender,
    for<'all> Lend<'all, Map<L, F>>: IntoLender,
{
    type Lend = Lend<'lend, <Lend<'this, Map<L, F>> as IntoLender>::Lender>;
}
impl<L: Lender, F> Lender for FlatMap<'_, L, F>
where
    Map<L, F>: Lender,
    for<'all> Lend<'all, Map<L, F>>: IntoLender,
{
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl<L: FusedLender, F> FusedLender for FlatMap<'_, L, F>
where
    Map<L, F>: Lender,
    for<'all> Lend<'all, Map<L, F>>: IntoLender,
{
}
pub struct FlattenCompat<'this, L: Lender>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    lender: Box<L>,
    inner: Option<<Lend<'this, L> as IntoLender>::Lender>,
}
impl<L: Lender> FlattenCompat<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    pub(crate) fn new(lender: L) -> Self {
        Self { lender: Box::new(lender), inner: None }
    }
}
impl<L: Lender + Clone> Clone for FlattenCompat<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
    for<'all> <Lend<'all, L> as IntoLender>::Lender: Clone,
{
    fn clone(&self) -> Self {
        Self { lender: self.lender.clone(), inner: self.inner.clone() }
    }
}
impl<L: Lender + fmt::Debug> fmt::Debug for FlattenCompat<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
    for<'all> <Lend<'all, L> as IntoLender>::Lender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlattenCompat").field("lender", &self.lender).field("inner", &self.inner).finish()
    }
}
impl<'lend, 'this, L: Lender> Lending<'lend> for FlattenCompat<'this, L>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    type Lend = Lend<'lend, <Lend<'this, L> as IntoLender>::Lender>;
}
impl<'this, L: Lender> Lender for FlattenCompat<'this, L>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: polonius return
        let reborrow = unsafe { &mut *(&mut self.inner as *mut Option<<Lend<'this, L> as IntoLender>::Lender>) };
        if let Some(inner) = reborrow {
            if let Some(x) = inner.next() {
                return Some(x);
            }
        }
        // SAFETY: inner is manually guaranteed to be the only lend alive of the inner iterator
        self.inner = self.lender.next().map(|l| unsafe {
            core::mem::transmute::<<Lend<'_, L> as IntoLender>::Lender, <Lend<'this, L> as IntoLender>::Lender>(
                l.into_lender(),
            )
        });
        self.inner.as_mut()?.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            match &self.inner {
                Some(inner) => inner.size_hint().0,
                None => self.lender.size_hint().0,
            },
            None,
        )
    }
}
impl<L: FusedLender> FusedLender for FlattenCompat<'_, L> where for<'all> Lend<'all, L>: IntoLender {}
