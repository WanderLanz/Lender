use aliasable::boxed::AliasableBox;
use core::fmt;
use maybe_dangling::MaybeDangling;

use crate::{FusedLender, IntoLender, Lend, Lender, Lending, Map, try_trait_v2::Try};

/// A lender that flattens one level of nesting in a lender of lenders.
///
/// This `struct` is created by the [`flatten()`](crate::Lender::flatten) method on [`Lender`].
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
    #[inline(always)]
    pub(crate) fn new(lender: L) -> Self {
        Self {
            inner: FlattenCompat::new(lender),
        }
    }

    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        *AliasableBox::into_unique(self.inner.lender)
    }
}

// Clone is not implemented for Flatten because the inner sub-lender may
// reference the AliasableBox allocation; a clone would create a new allocation
// but the cloned inner sub-lender would still reference the original.

impl<L: Lender + fmt::Debug> fmt::Debug for Flatten<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
    for<'all> <Lend<'all, L> as IntoLender>::Lender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flatten")
            .field("inner", &self.inner)
            .finish()
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
    // SAFETY: the lend is that of the inner lender
    crate::unsafe_assume_covariance!();
    #[inline(always)]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.inner.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        self.inner.try_fold(init, f)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.inner.fold(init, f)
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner.count()
    }
}

impl<L: FusedLender> FusedLender for Flatten<'_, L> where for<'all> Lend<'all, L>: IntoLender {}

/// A lender that maps each element to a lender, and yields the elements of the produced lenders.
///
/// This `struct` is created by the [`flat_map()`](crate::Lender::flat_map) method on [`Lender`].
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
    #[inline(always)]
    pub(crate) fn new(lender: L, f: F) -> Self {
        Self {
            inner: FlattenCompat::new(Map::new(lender, f)),
        }
    }

    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        (*AliasableBox::into_unique(self.inner.lender)).into_inner()
    }

    /// Returns the inner lender and the mapping function.
    #[inline(always)]
    pub fn into_parts(self) -> (L, F) {
        (*AliasableBox::into_unique(self.inner.lender)).into_parts()
    }
}

// Clone is not implemented for FlatMap because the inner sub-lender may
// reference the AliasableBox allocation; a clone would create a new allocation
// but the cloned inner sub-lender would still reference the original.

impl<L: Lender + fmt::Debug, F> fmt::Debug for FlatMap<'_, L, F>
where
    Map<L, F>: Lender,
    for<'all> Lend<'all, Map<L, F>>: IntoLender,
    for<'all> <Lend<'all, Map<L, F>> as IntoLender>::Lender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlatMap")
            .field("inner", &self.inner)
            .finish()
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
    // SAFETY: the lend is that of the inner lender
    crate::unsafe_assume_covariance!();
    #[inline(always)]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.inner.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn try_fold<B, G, R>(&mut self, init: B, f: G) -> R
    where
        Self: Sized,
        G: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        self.inner.try_fold(init, f)
    }

    #[inline]
    fn fold<B, G>(self, init: B, f: G) -> B
    where
        Self: Sized,
        G: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.inner.fold(init, f)
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner.count()
    }
}

impl<L: FusedLender, F> FusedLender for FlatMap<'_, L, F>
where
    Map<L, F>: Lender,
    for<'all> Lend<'all, Map<L, F>>: IntoLender,
{
}

pub(crate) struct FlattenCompat<'this, L: Lender>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    // MaybeDangling wraps the inner lender to indicate it may reference data
    // from the outer lender. AliasableBox eliminates noalias retagging that would
    // invalidate the inner reference when the struct is moved.
    // Field order ensures outer lender drops last.
    //
    // See https://github.com/WanderLanz/Lender/issues/34
    inner: MaybeDangling<Option<<Lend<'this, L> as IntoLender>::Lender>>,
    lender: AliasableBox<L>,
}

impl<L: Lender> FlattenCompat<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
{
    #[inline(always)]
    pub(crate) fn new(lender: L) -> Self {
        Self {
            inner: MaybeDangling::new(None),
            lender: AliasableBox::from_unique(alloc::boxed::Box::new(lender)),
        }
    }
}

// Clone is not implemented for FlattenCompat because the inner sub-lender may
// reference the AliasableBox allocation; a clone would create a new allocation
// but the cloned inner sub-lender would still reference the original.

impl<L: Lender + fmt::Debug> fmt::Debug for FlattenCompat<'_, L>
where
    for<'all> Lend<'all, L>: IntoLender,
    for<'all> <Lend<'all, L> as IntoLender>::Lender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlattenCompat")
            .field("lender", &self.lender)
            .field("inner", &self.inner)
            .finish()
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
    // SAFETY: the lend is that of the inner lender
    crate::unsafe_assume_covariance!();
    #[inline]
    #[allow(clippy::question_mark)]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        loop {
            // SAFETY: Polonius return
            #[allow(clippy::deref_addrof)]
            let reborrow = unsafe { &mut *(&raw mut *self.inner) };
            if let Some(inner) = reborrow {
                if let Some(x) = inner.next() {
                    return Some(x);
                }
            }

            // SAFETY: inner is manually guaranteed to be the only lend alive of the inner iterator
            *self.inner = self.lender.next().map(|l| unsafe {
                core::mem::transmute::<
                    <Lend<'_, L> as IntoLender>::Lender,
                    <Lend<'this, L> as IntoLender>::Lender,
                >(l.into_lender())
            });

            if self.inner.is_none() {
                return None;
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner_len = match &*self.inner {
            Some(inner) => inner.size_hint().0,
            None => 0,
        };
        (inner_len, None)
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        use core::ops::ControlFlow;
        let mut acc = init;
        if let Some(ref mut inner) = *self.inner {
            match inner.try_fold(acc, &mut f).branch() {
                ControlFlow::Continue(b) => acc = b,
                ControlFlow::Break(r) => return R::from_residual(r),
            }
        }
        *self.inner = None;
        loop {
            let Some(l) = self.lender.next() else { break };
            // SAFETY: inner is manually guaranteed to be the only lend alive of the inner lender
            *self.inner = Some(unsafe {
                core::mem::transmute::<
                    <Lend<'_, L> as IntoLender>::Lender,
                    <Lend<'this, L> as IntoLender>::Lender,
                >(l.into_lender())
            });
            if let Some(ref mut inner) = *self.inner {
                match inner.try_fold(acc, &mut f).branch() {
                    ControlFlow::Continue(b) => acc = b,
                    ControlFlow::Break(r) => return R::from_residual(r),
                }
            }
            *self.inner = None;
        }
        R::from_output(acc)
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut acc = init;
        if let Some(inner) = (&mut *self.inner).take() {
            acc = inner.fold(acc, &mut f);
        }
        while let Some(l) = self.lender.next() {
            // SAFETY: inner is manually guaranteed to be the only lend alive of the inner lender
            let sub = unsafe {
                core::mem::transmute::<
                    <Lend<'_, L> as IntoLender>::Lender,
                    <Lend<'this, L> as IntoLender>::Lender,
                >(l.into_lender())
            };
            acc = sub.fold(acc, &mut f);
        }
        acc
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.fold(0, |count, _| count + 1)
    }
}

impl<L: FusedLender> FusedLender for FlattenCompat<'_, L> where for<'all> Lend<'all, L>: IntoLender {}

#[cfg(test)]
mod test {
    use super::*;

    struct Parent([i32; 4]);

    impl<'lend> Lending<'lend> for Parent {
        type Lend = Child<'lend>;
    }

    impl Lender for Parent {
        crate::check_covariance!();
        fn next(&mut self) -> Option<Lend<'_, Self>> {
            Some(Child { array_ref: &self.0 })
        }
    }

    struct Child<'a> {
        array_ref: &'a [i32; 4],
    }

    impl<'a, 'lend> Lending<'lend> for Child<'a> {
        type Lend = &'lend [i32; 4];
    }

    impl<'a> Lender for Child<'a> {
        crate::check_covariance!();
        fn next(&mut self) -> Option<Lend<'_, Self>> {
            Some(self.array_ref)
        }
    }

    // This test will fail if FlattenCompat stores L instead of Box<L>. In that
    // case, when Flatten<Parent> is moved, the array inside Parent is moved,
    // too, but FlattenCompat.inner will still contain a Child holding a
    // reference to the previous location.
    #[test]
    fn test_flatten() {
        let lender = Parent([0, 1, 2, 3]);
        let mut flatten = lender.flatten();
        flatten.next();
        moved_flatten(flatten);
    }

    fn moved_flatten(mut flatten: Flatten<Parent>) {
        let next_array_ref = flatten.next().unwrap() as *const _;
        let array_ref = &flatten.inner.lender.0 as *const _;
        assert_eq!(
            next_array_ref, array_ref,
            "Array references returned by the flattened lender should refer to the array in the parent lender"
        );
    }

    #[test]
    fn test_flatmap_empty() {
        use crate::traits::IteratorExt;

        let mut l = [1, 0, 2]
            .into_iter()
            .into_lender()
            .flat_map(|n| (0..n).into_lender());
        assert_eq!(l.next(), Some(0));
        assert_eq!(l.next(), Some(0));
        assert_eq!(l.next(), Some(1));
        assert_eq!(l.next(), None);
        assert_eq!(l.next(), None);
    }
}
