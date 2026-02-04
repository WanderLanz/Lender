use aliasable::boxed::AliasableBox;
use core::fmt;
use maybe_dangling::MaybeDangling;

use crate::{
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, IntoFallibleLender, Map,
    try_trait_v2::Try,
};

/// A fallible lender that flattens one level of nesting in a lender of lenders.
///
/// This `struct` is created by the [`flatten()`](crate::FallibleLender::flatten) method on
/// [`FallibleLender`]. See its documentation for more.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Flatten<'this, L: FallibleLender>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
{
    inner: FlattenCompat<'this, L>,
}

impl<L: FallibleLender> Flatten<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
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

impl<L: FallibleLender + fmt::Debug> fmt::Debug for Flatten<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
    for<'all> <FallibleLend<'all, L> as IntoFallibleLender>::FallibleLender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flatten")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'lend, 'this, L: FallibleLender> FallibleLending<'lend> for Flatten<'this, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
{
    type Lend = FallibleLend<'lend, <FallibleLend<'this, L> as IntoFallibleLender>::FallibleLender>;
}

impl<L: FallibleLender> FallibleLender for Flatten<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender<Error = L::Error>,
{
    type Error = L::Error;
    // SAFETY: the lend is that of the inner lender
    crate::unsafe_assume_covariance_fallible!();

    #[inline(always)]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.inner.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        self.inner.try_fold(init, f)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.inner.fold(init, f)
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.inner.count()
    }
}

impl<L: FusedFallibleLender> FusedFallibleLender for Flatten<'_, L> where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender<Error = L::Error>
{
}

/// A fallible lender that maps each element to a lender, and yields the elements of the produced lenders.
///
/// This `struct` is created by the [`flat_map()`](crate::FallibleLender::flat_map) method on
/// [`FallibleLender`]. See its documentation for more.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FlatMap<'this, L: FallibleLender, F>
where
    Map<L, F>: FallibleLender,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender,
{
    inner: FlattenCompat<'this, Map<L, F>>,
}

impl<L: FallibleLender, F> FlatMap<'_, L, F>
where
    Map<L, F>: FallibleLender,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender,
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

impl<L: FallibleLender + fmt::Debug, F> fmt::Debug for FlatMap<'_, L, F>
where
    Map<L, F>: FallibleLender + Clone,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender,
    for<'all> <FallibleLend<'all, Map<L, F>> as IntoFallibleLender>::FallibleLender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlatMap")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'lend, 'this, L: FallibleLender, F> FallibleLending<'lend> for FlatMap<'this, L, F>
where
    Map<L, F>: FallibleLender,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender,
{
    type Lend =
        FallibleLend<'lend, <FallibleLend<'this, Map<L, F>> as IntoFallibleLender>::FallibleLender>;
}

impl<L: FallibleLender, F> FallibleLender for FlatMap<'_, L, F>
where
    Map<L, F>: FallibleLender<Error = L::Error>,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender<Error = L::Error>,
{
    type Error = L::Error;
    // SAFETY: the lend is that of the inner lender
    crate::unsafe_assume_covariance_fallible!();

    #[inline(always)]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.inner.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn try_fold<B, G, R>(&mut self, init: B, f: G) -> Result<R, Self::Error>
    where
        Self: Sized,
        G: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        self.inner.try_fold(init, f)
    }

    #[inline]
    fn fold<B, G>(self, init: B, f: G) -> Result<B, Self::Error>
    where
        Self: Sized,
        G: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.inner.fold(init, f)
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.inner.count()
    }
}

impl<L: FusedFallibleLender, F> FusedFallibleLender for FlatMap<'_, L, F>
where
    Map<L, F>: FallibleLender<Error = L::Error>,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender<Error = L::Error>,
{
}

/// The internal implementation backing both [`Flatten`] and [`FlatMap`] for fallible lenders.
pub(crate) struct FlattenCompat<'this, L: FallibleLender>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
{
    // MaybeDangling wraps the inner lender to indicate it may reference data
    // from the outer lender. AliasableBox eliminates noalias retagging that would
    // invalidate the inner reference when the struct is moved.
    // Field order ensures outer lender drops last.
    //
    // See https://github.com/WanderLanz/Lender/issues/34
    inner: MaybeDangling<Option<<FallibleLend<'this, L> as IntoFallibleLender>::FallibleLender>>,
    lender: AliasableBox<L>,
}

impl<L: FallibleLender> FlattenCompat<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
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

impl<L: FallibleLender + fmt::Debug> fmt::Debug for FlattenCompat<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
    for<'all> <FallibleLend<'all, L> as IntoFallibleLender>::FallibleLender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlattenCompat")
            .field("lender", &self.lender)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'lend, 'this, L: FallibleLender> FallibleLending<'lend> for FlattenCompat<'this, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
{
    type Lend = FallibleLend<'lend, <FallibleLend<'this, L> as IntoFallibleLender>::FallibleLender>;
}

impl<'this, L: FallibleLender> FallibleLender for FlattenCompat<'this, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender<Error = L::Error>,
{
    type Error = L::Error;
    // SAFETY: the lend is that of the inner lender
    crate::unsafe_assume_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        loop {
            // SAFETY: Polonius return
            #[allow(clippy::deref_addrof)]
            let reborrow = unsafe { &mut *(&raw mut *self.inner) };
            if let Some(inner) = reborrow {
                if let Some(x) = inner.next()? {
                    return Ok(Some(x));
                }
            }
            // SAFETY: inner is manually guaranteed to be the only FallibleLend alive of the inner iterator
            *self.inner = self.lender.next()?.map(|l| unsafe {
                core::mem::transmute::<
                    <FallibleLend<'_, L> as IntoFallibleLender>::FallibleLender,
                    <FallibleLend<'this, L> as IntoFallibleLender>::FallibleLender,
                >(l.into_fallible_lender())
            });

            if self.inner.is_none() {
                return Ok(None);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            match &*self.inner {
                Some(inner) => inner.size_hint().0,
                None => 0,
            },
            None,
        )
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        use core::ops::ControlFlow;
        let mut acc = init;
        if let Some(ref mut inner) = *self.inner {
            match inner.try_fold(acc, &mut f)?.branch() {
                ControlFlow::Continue(b) => acc = b,
                ControlFlow::Break(r) => return Ok(R::from_residual(r)),
            }
        }
        *self.inner = None;
        loop {
            let Some(l) = self.lender.next()? else { break };
            // SAFETY: inner is manually guaranteed to be the only lend alive of the inner lender
            *self.inner = Some(unsafe {
                core::mem::transmute::<
                    <FallibleLend<'_, L> as IntoFallibleLender>::FallibleLender,
                    <FallibleLend<'this, L> as IntoFallibleLender>::FallibleLender,
                >(l.into_fallible_lender())
            });
            if let Some(ref mut inner) = *self.inner {
                match inner.try_fold(acc, &mut f)?.branch() {
                    ControlFlow::Continue(b) => acc = b,
                    ControlFlow::Break(r) => return Ok(R::from_residual(r)),
                }
            }
            *self.inner = None;
        }
        Ok(R::from_output(acc))
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        if let Some(inner) = self.inner.take() {
            acc = inner.fold(acc, &mut f)?;
        }
        while let Some(l) = self.lender.next()? {
            // SAFETY: inner is manually guaranteed to be the only lend alive of the inner lender
            let sub = unsafe {
                core::mem::transmute::<
                    <FallibleLend<'_, L> as IntoFallibleLender>::FallibleLender,
                    <FallibleLend<'this, L> as IntoFallibleLender>::FallibleLender,
                >(l.into_fallible_lender())
            };
            acc = sub.fold(acc, &mut f)?;
        }
        Ok(acc)
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.fold(0, |count, _| Ok(count + 1))
    }
}

impl<L: FusedFallibleLender> FusedFallibleLender for FlattenCompat<'_, L> where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender<Error = L::Error>
{
}

#[cfg(test)]
mod test {
    use core::convert::Infallible;

    use super::*;
    use crate::{IntoFallible, Lend, Lender, Lending};

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

    impl<'a, 'lend> FallibleLending<'lend> for Child<'a> {
        type Lend = &'lend [i32; 4];
    }

    impl<'a> FallibleLender for Child<'a> {
        type Error = Infallible;
        crate::check_covariance_fallible!();

        fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
            Ok(Some(self.array_ref))
        }
    }

    // This test will fail if FlattenCompat stores L instead of Box<L>. In that
    // case, when Flatten<Parent> is moved, the array inside Parent is moved,
    // too, but FlattenCompat.inner will still contain a Child holding a
    // reference to the previous location.
    #[test]
    fn test_flatten() -> Result<(), Infallible> {
        let lender = Parent([0, 1, 2, 3]);
        let mut flatten = lender.into_fallible().flatten();
        let _ = flatten.next();
        moved_flatten(flatten)
    }

    fn moved_flatten(
        mut flatten: Flatten<IntoFallible<Parent>>,
    ) -> Result<(), Infallible> {
        let next_array_ref = flatten.next()?.unwrap() as *const _;
        let array_ref = &flatten.inner.lender.lender.0 as *const _;
        assert_eq!(
            next_array_ref, array_ref,
            "Array references returned by the flattened FallibleLender should refer to the array in the parent FallibleLender"
        );
        Ok(())
    }

    #[test]
    fn test_flat_map_empty() {
        use crate::traits::IteratorExt;

        let mut l = [1, 0, 2]
            .into_iter()
            .into_lender()
            .into_fallible()
            .flat_map(|n| Ok((0..n).into_lender().into_fallible()));
        assert_eq!(l.next(), Ok(Some(0)));
        assert_eq!(l.next(), Ok(Some(0)));
        assert_eq!(l.next(), Ok(Some(1)));
        assert_eq!(l.next(), Ok(None));
        assert_eq!(l.next(), Ok(None));
    }
}
