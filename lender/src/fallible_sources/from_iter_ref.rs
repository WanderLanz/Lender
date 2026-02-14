use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

use crate::{DoubleEndedFallibleLender, FallibleLend, FallibleLender, FallibleLending};

/// Creates a fallible lender that stores each element from a
/// fallible iterator and lends a reference to it.
///
/// This function can be conveniently accessed using the
/// [`into_fallible_ref_lender`](crate::traits::FallibleIteratorRefExt::into_fallible_ref_lender)
/// extension method.
///
/// Unlike [`from_fallible_iter`](crate::from_fallible_iter),
/// which passes items through transparently, this source
/// stores each element internally and lends a reference to it,
/// turning a `FallibleIterator<Item = T>` into a
/// `FallibleLender` with `FallibleLend<'lend> = &'lend T`.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// # use fallible_iterator::IteratorExt;
/// let mut lender = lender::from_fallible_iter_ref(
///     [1, 2, 3].into_iter().into_fallible(),
/// );
/// let item: &i32 = lender.next().unwrap().unwrap();
/// assert_eq!(*item, 1);
/// ```
#[inline]
pub fn from_iter_ref<I: FallibleIterator>(iter: I) -> FromIterRef<I> {
    FromIterRef {
        iter,
        current: None,
    }
}

/// A fallible lender that stores each element from a fallible
/// iterator and lends a reference to it.
///
/// This `struct` is created by the
/// [`from_fallible_iter_ref()`](crate::from_fallible_iter_ref)
/// function.
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FromIterRef<I: FallibleIterator> {
    iter: I,
    current: Option<I::Item>,
}

impl<'lend, I: FallibleIterator> FallibleLending<'lend> for FromIterRef<I> {
    type Lend = &'lend I::Item;
}

impl<I: FallibleIterator> FallibleLender for FromIterRef<I> {
    type Error = I::Error;
    crate::check_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.current = self.iter.next()?;
        Ok(self.current.as_ref())
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.current = self.iter.nth(n)?;
        Ok(self.current.as_ref())
    }

    #[inline(always)]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.iter.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut current = self.current;
        self.iter.fold(init, |acc, item| {
            current = Some(item);
            f(acc, current.as_ref().unwrap())
        })
    }
}

impl<I: DoubleEndedFallibleIterator> DoubleEndedFallibleLender for FromIterRef<I> {
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.current = self.iter.next_back()?;
        Ok(self.current.as_ref())
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut current = self.current;
        self.iter.rfold(init, |acc, item| {
            current = Some(item);
            f(acc, current.as_ref().unwrap())
        })
    }
}

// Note: FusedFallibleLender and ExactSizeFallibleLender are not
// implemented for FromIterRef because the fallible_iterator crate
// does not expose FusedFallibleIterator or
// ExactSizeFallibleIterator marker traits.

impl<I: FallibleIterator> From<I> for FromIterRef<I> {
    #[inline(always)]
    fn from(iter: I) -> Self {
        from_iter_ref(iter)
    }
}
