use core::iter::FusedIterator;

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, FallibleLend, FallibleLender,
    FusedFallibleLender, FusedLender, Lend, Lender, Lending,
};

// The user must check that the lender did not produce an error after use
// as a lender or iterator.
// This should never be available to users of this crate, and is only exposed
// so that it can be used as a bound for methods in `FallibleLender`.
/// An adapter to use a [`FallibleLender`] as a [`Lender`] or [`Iterator`] over items.
#[derive(Debug)]
#[must_use = "iterators/lenders are lazy and do nothing unless consumed"]
pub struct NonFallibleAdapter<'this, L>
where
    L: FallibleLender + 'this,
{
    lender: L,
    error: &'this mut Option<L::Error>,
}

pub(crate) fn process<L, F, U>(lender: L, mut f: F) -> Result<U, (U, L::Error)>
where
    L: FallibleLender,
    for<'all> F: FnMut(NonFallibleAdapter<'all, L>) -> U,
{
    let mut error = None;
    let adapter = NonFallibleAdapter {
        lender,
        error: &mut error,
    };
    let value = f(adapter);
    match error {
        None => Ok(value),
        Some(err) => Err((value, err)),
    }
}

impl<'this, L> Iterator for NonFallibleAdapter<'this, L>
where
    L: FallibleLender,
    for<'all> FallibleLend<'all, L>: 'this,
{
    type Item = FallibleLend<'this, L>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.error.is_some() {
            return None;
        }
        match self.lender.next() {
            Ok(next) => {
                // SAFETY: for<'all> Lend<'all, L>: 'this
                unsafe {
                    core::mem::transmute::<
                        Option<FallibleLend<'_, L>>,
                        Option<FallibleLend<'this, L>>,
                    >(next)
                }
            }
            Err(err) => {
                *self.error = Some(err);
                None
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}

impl<'this, L> DoubleEndedIterator for NonFallibleAdapter<'this, L>
where
    L: DoubleEndedFallibleLender,
    for<'all> FallibleLend<'all, L>: 'this,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.error.is_some() {
            return None;
        }
        match self.lender.next_back() {
            Ok(next) => {
                // SAFETY: for<'all> Lend<'all, L>: 'this
                unsafe {
                    core::mem::transmute::<
                        Option<FallibleLend<'_, L>>,
                        Option<FallibleLend<'this, L>>,
                    >(next)
                }
            }
            Err(err) => {
                *self.error = Some(err);
                None
            }
        }
    }
}

impl<'lend, 'this, L> Lending<'lend> for NonFallibleAdapter<'this, L>
where
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'this, L> Lender for NonFallibleAdapter<'this, L>
where
    L: FallibleLender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.error.is_some() {
            return None;
        }
        match self.lender.next() {
            Ok(next) => next,
            Err(err) => {
                *self.error = Some(err);
                None
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }
}

impl<'this, L> DoubleEndedLender for NonFallibleAdapter<'this, L>
where
    L: DoubleEndedFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        if self.error.is_some() {
            return None;
        }
        match self.lender.next_back() {
            Ok(next) => next,
            Err(err) => {
                *self.error = Some(err);
                None
            }
        }
    }
}

impl<'this, L> FusedIterator for NonFallibleAdapter<'this, L>
where
    L: FusedFallibleLender,
    for<'all> FallibleLend<'all, L>: 'this,
{
}

impl<'this, L> FusedLender for NonFallibleAdapter<'this, L> where L: FusedFallibleLender {}
