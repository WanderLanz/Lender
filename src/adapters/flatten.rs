use core::fmt;

use crate::{FusedLender, IntoLender, Lender, Lending, Map};
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Flatten<'this, L: Lender>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
{
    inner: FlattenCompat<'this, L>,
}
impl<'this, L: Lender> Flatten<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
{
    pub(crate) fn new(lender: L) -> Self {
        Self { inner: FlattenCompat::new(lender) }
    }
}
impl<'this, L: Lender + Clone> Clone for Flatten<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
    for<'all> <<L as Lending<'all>>::Lend as IntoLender>::Lender: Clone,
{
    fn clone(&self) -> Self {
        Flatten { inner: self.inner.clone() }
    }
}
impl<'this, L: Lender + fmt::Debug> fmt::Debug for Flatten<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
    for<'all> <<L as Lending<'all>>::Lend as IntoLender>::Lender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flatten").field("inner", &self.inner).finish()
    }
}
impl<'lend, 'this, L: Lender> Lending<'lend> for Flatten<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
{
    type Lend = <<<L as Lending<'this>>::Lend as IntoLender>::Lender as Lending<'lend>>::Lend;
}
impl<'this, L: Lender> Lender for Flatten<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl<'this, L: FusedLender> FusedLender for Flatten<'this, L> where for<'all> <L as Lending<'all>>::Lend: IntoLender {}

#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FlatMap<'this, L: Lender, F>
where
    Map<L, F>: Lender,
    for<'all> <Map<L, F> as Lending<'all>>::Lend: IntoLender,
{
    inner: FlattenCompat<'this, Map<L, F>>,
}
impl<'this, L: Lender, F> FlatMap<'this, L, F>
where
    Map<L, F>: Lender,
    for<'all> <Map<L, F> as Lending<'all>>::Lend: IntoLender,
{
    pub(crate) fn new(lender: L, f: F) -> Self {
        Self { inner: FlattenCompat::new(Map::new(lender, f)) }
    }
}
impl<'this, L: Lender + Clone, F: Clone> Clone for FlatMap<'this, L, F>
where
    Map<L, F>: Lender + Clone,
    for<'all> <Map<L, F> as Lending<'all>>::Lend: IntoLender,
    for<'all> <<Map<L, F> as Lending<'all>>::Lend as IntoLender>::Lender: Clone,
{
    fn clone(&self) -> Self {
        FlatMap { inner: self.inner.clone() }
    }
}
impl<'this, L: Lender + fmt::Debug, F> fmt::Debug for FlatMap<'this, L, F>
where
    Map<L, F>: Lender + Clone,
    for<'all> <Map<L, F> as Lending<'all>>::Lend: IntoLender,
    for<'all> <<Map<L, F> as Lending<'all>>::Lend as IntoLender>::Lender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlatMap").field("inner", &self.inner).finish()
    }
}
impl<'lend, 'this, L: Lender, F> Lending<'lend> for FlatMap<'this, L, F>
where
    Map<L, F>: Lender,
    for<'all> <Map<L, F> as Lending<'all>>::Lend: IntoLender,
{
    type Lend = <<<Map<L, F> as Lending<'this>>::Lend as IntoLender>::Lender as Lending<'lend>>::Lend;
}
impl<'this, L: Lender, F> Lender for FlatMap<'this, L, F>
where
    Map<L, F>: Lender,
    for<'all> <Map<L, F> as Lending<'all>>::Lend: IntoLender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl<'this, L: FusedLender, F> FusedLender for FlatMap<'this, L, F>
where
    Map<L, F>: Lender,
    for<'all> <Map<L, F> as Lending<'all>>::Lend: IntoLender,
{
}
pub struct FlattenCompat<'this, L: Lender>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
{
    lender: L,
    inner: Option<<<L as Lending<'this>>::Lend as IntoLender>::Lender>,
}
impl<'this, L: Lender> FlattenCompat<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
{
    pub(crate) fn new(lender: L) -> Self {
        Self { lender, inner: None }
    }
}
impl<'this, L: Lender + Clone> Clone for FlattenCompat<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
    for<'all> <<L as Lending<'all>>::Lend as IntoLender>::Lender: Clone,
{
    fn clone(&self) -> Self {
        Self { lender: self.lender.clone(), inner: self.inner.clone() }
    }
}
impl<'this, L: Lender + fmt::Debug> fmt::Debug for FlattenCompat<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
    for<'all> <<L as Lending<'all>>::Lend as IntoLender>::Lender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlattenCompat").field("lender", &self.lender).field("inner", &self.inner).finish()
    }
}
impl<'lend, 'this, L: Lender> Lending<'lend> for FlattenCompat<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
{
    type Lend = <<<L as Lending<'this>>::Lend as IntoLender>::Lender as Lending<'lend>>::Lend;
}
impl<'this, L: Lender> Lender for FlattenCompat<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: IntoLender,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        // SAFETY: polonius return
        let reborrow =
            unsafe { &mut *(&mut self.inner as *mut Option<<<L as Lending<'this>>::Lend as IntoLender>::Lender>) };
        if let Some(inner) = reborrow {
            if let Some(x) = inner.next() {
                return Some(x);
            }
        }
        // SAFETY: inner is manually guaranteed to be the only lend alive of the inner iterator
        self.inner = self.lender.next().map(|l| unsafe {
            core::mem::transmute::<
                <<L as Lending<'_>>::Lend as IntoLender>::Lender,
                <<L as Lending<'this>>::Lend as IntoLender>::Lender,
            >(l.into_lender())
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
impl<'this, L: FusedLender> FusedLender for FlattenCompat<'this, L> where for<'all> <L as Lending<'all>>::Lend: IntoLender {}
