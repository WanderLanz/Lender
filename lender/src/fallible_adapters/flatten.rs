use alloc::boxed::Box;
use core::fmt;

use crate::{FallibleLend, FallibleLender, FallibleLending, IntoFallibleLender, Map};

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
    pub(crate) fn new(lender: L) -> Self {
        Self { inner: FlattenCompat::new(lender) }
    }
    pub fn into_inner(self) -> L {
        *self.inner.lender
    }
}
impl<L: FallibleLender + Clone> Clone for Flatten<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
    for<'all> <FallibleLend<'all, L> as IntoFallibleLender>::FallibleLender: Clone,
{
    fn clone(&self) -> Self {
        Flatten { inner: self.inner.clone() }
    }
}
impl<L: FallibleLender + fmt::Debug> fmt::Debug for Flatten<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
    for<'all> <FallibleLend<'all, L> as IntoFallibleLender>::FallibleLender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flatten").field("inner", &self.inner).finish()
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
    crate::inherit_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

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
    pub(crate) fn new(lender: L, f: F) -> Self {
        Self { inner: FlattenCompat::new(Map::new(lender, f)) }
    }
}
impl<L: FallibleLender + Clone, F: Clone> Clone for FlatMap<'_, L, F>
where
    Map<L, F>: FallibleLender + Clone,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender,
    for<'all> <FallibleLend<'all, Map<L, F>> as IntoFallibleLender>::FallibleLender: Clone,
{
    fn clone(&self) -> Self {
        FlatMap { inner: self.inner.clone() }
    }
}
impl<L: FallibleLender + fmt::Debug, F> fmt::Debug for FlatMap<'_, L, F>
where
    Map<L, F>: FallibleLender + Clone,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender,
    for<'all> <FallibleLend<'all, Map<L, F>> as IntoFallibleLender>::FallibleLender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlatMap").field("inner", &self.inner).finish()
    }
}
impl<'lend, 'this, L: FallibleLender, F> FallibleLending<'lend> for FlatMap<'this, L, F>
where
    Map<L, F>: FallibleLender,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender,
{
    type Lend = FallibleLend<'lend, <FallibleLend<'this, Map<L, F>> as IntoFallibleLender>::FallibleLender>;
}
impl<L: FallibleLender, F> FallibleLender for FlatMap<'_, L, F>
where
    Map<L, F>: FallibleLender<Error = L::Error>,
    for<'all> FallibleLend<'all, Map<L, F>>: IntoFallibleLender<Error = L::Error>,
{
    type Error = L::Error;
    crate::inherit_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.inner.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct FlattenCompat<'this, L: FallibleLender>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
{
    lender: Box<L>,
    inner: Option<<FallibleLend<'this, L> as IntoFallibleLender>::FallibleLender>,
}
impl<L: FallibleLender> FlattenCompat<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
{
    pub(crate) fn new(lender: L) -> Self {
        Self { lender: Box::new(lender), inner: None }
    }
}
impl<L: FallibleLender + Clone> Clone for FlattenCompat<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
    for<'all> <FallibleLend<'all, L> as IntoFallibleLender>::FallibleLender: Clone,
{
    fn clone(&self) -> Self {
        Self { lender: self.lender.clone(), inner: self.inner.clone() }
    }
}
impl<L: FallibleLender + fmt::Debug> fmt::Debug for FlattenCompat<'_, L>
where
    for<'all> FallibleLend<'all, L>: IntoFallibleLender,
    for<'all> <FallibleLend<'all, L> as IntoFallibleLender>::FallibleLender: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlattenCompat").field("lender", &self.lender).field("inner", &self.inner).finish()
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
    crate::inherit_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        loop {
            // SAFETY: Polonius return
            #[allow(clippy::deref_addrof)]
            let reborrow = unsafe { &mut *(&raw mut self.inner) };
            if let Some(inner) = reborrow {
                if let Some(x) = inner.next()? {
                    return Ok(Some(x));
                }
            }
            // SAFETY: inner is manually guaranteed to be the only FallibleLend alive of the inner iterator
            self.inner = self.lender.next()?.map(|l| unsafe {
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
            match &self.inner {
                Some(inner) => inner.size_hint().0,
                None => self.lender.size_hint().0,
            },
            None,
        )
    }
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

    fn moved_flatten(mut flatten: Flatten<IntoFallible<Infallible, Parent>>) -> Result<(), Infallible> {
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
            .into_fallible::<Infallible>()
            .flat_map(|n| Ok((0..n).into_lender().into_fallible()));
        assert_eq!(l.next(), Ok(Some(0)));
        assert_eq!(l.next(), Ok(Some(0)));
        assert_eq!(l.next(), Ok(Some(1)));
        assert_eq!(l.next(), Ok(None));
        assert_eq!(l.next(), Ok(None));
    }
}
