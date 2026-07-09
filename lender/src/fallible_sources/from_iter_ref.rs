use core::num::NonZeroUsize;
use core::ops::ControlFlow;

use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

use crate::try_trait_v2::Try;
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
pub const fn from_iter_ref<I: FallibleIterator>(iter: I) -> FromIterRef<I> {
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
        self.current = None;
        self.current = self.iter.next()?;
        Ok(self.current.as_ref())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.current = None;
        self.current = self.iter.nth(n)?;
        Ok(self.current.as_ref())
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.iter.count()
    }

    #[inline]
    fn last<'call>(&'call mut self) -> Result<Option<FallibleLend<'call, Self>>, Self::Error>
    where
        Self: Sized,
    {
        self.current = None;
        while let Some(item) = self.iter.next()? {
            self.current = Some(item);
        }
        Ok(self.current.as_ref())
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        for i in 0..n {
            if self.iter.next()?.is_none() {
                // SAFETY: `i` is always less than `n`.
                return Ok(Err(unsafe { NonZeroUsize::new_unchecked(n - i) }));
            }
        }
        Ok(Ok(()))
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, mut init: B, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        while let Some(item) = self.iter.next()? {
            match f(init, &item)?.branch() {
                ControlFlow::Break(residual) => return Ok(R::from_residual(residual)),
                ControlFlow::Continue(output) => init = output,
            }
        }
        Ok(R::from_output(init))
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.iter.fold(init, |acc, item| f(acc, &item))
    }
}

impl<I: DoubleEndedFallibleIterator> DoubleEndedFallibleLender for FromIterRef<I> {
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.current = None;
        self.current = self.iter.next_back()?;
        Ok(self.current.as_ref())
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        for i in 0..n {
            if self.iter.next_back()?.is_none() {
                // SAFETY: `i` is always less than `n`.
                return Ok(Err(unsafe { NonZeroUsize::new_unchecked(n - i) }));
            }
        }
        Ok(Ok(()))
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, mut init: B, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        while let Some(item) = self.iter.next_back()? {
            match f(init, &item)?.branch() {
                ControlFlow::Break(residual) => return Ok(R::from_residual(residual)),
                ControlFlow::Continue(output) => init = output,
            }
        }
        Ok(R::from_output(init))
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.iter.rfold(init, |acc, item| f(acc, &item))
    }
}

// Note: FusedFallibleLender and ExactSizeFallibleLender are not
// implemented for FromIterRef because the fallible_iterator crate
// does not expose FusedFallibleIterator or
// ExactSizeFallibleIterator marker traits.

impl<I: FallibleIterator> From<I> for FromIterRef<I> {
    #[inline]
    fn from(iter: I) -> Self {
        from_iter_ref(iter)
    }
}

#[cfg(test)]
mod test {
    use core::cell::{RefCell, RefMut};
    use core::convert::Infallible;

    use fallible_iterator::FallibleIterator;

    use super::from_iter_ref;
    use crate::FallibleLender;

    // A fallible iterator whose items each hold an exclusive borrow of `cell`.
    struct BorrowIter<'a> {
        cell: &'a RefCell<i32>,
        n: usize,
    }
    impl<'a> FallibleIterator for BorrowIter<'a> {
        type Item = RefMut<'a, i32>;
        type Error = Infallible;
        fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
            if self.n == 0 {
                return Ok(None);
            }
            self.n -= 1;
            Ok(Some(self.cell.borrow_mut()))
        }
    }

    #[test]
    fn test_drops_previous_item_before_fetch() -> Result<(), Infallible> {
        let cell = RefCell::new(0);
        let mut l = from_iter_ref(BorrowIter { cell: &cell, n: 3 });
        assert!(l.next()?.is_some());
        assert!(l.next()?.is_some()); // buggy code panics "already mutably borrowed"
        assert!(l.next()?.is_some());
        assert!(l.next()?.is_none());
        Ok(())
    }
}
