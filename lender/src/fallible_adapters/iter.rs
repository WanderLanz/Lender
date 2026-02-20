use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

use crate::{DoubleEndedFallibleLender, FallibleLend, FallibleLender, Iter};

impl<'this, L: 'this> FallibleIterator for Iter<'this, L>
where
    L: FallibleLender,
    for<'all> FallibleLend<'all, L>: 'this,
{
    type Item = FallibleLend<'this, L>;
    type Error = L::Error;
    #[inline]
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        Ok(
            // SAFETY: for<'all> FallibleLend<'all, L>: 'this
            unsafe {
                core::mem::transmute::<Option<FallibleLend<'_, L>>, Option<FallibleLend<'this, L>>>(
                    self.lender.next()?,
                )
            },
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
}

impl<'this, L: 'this> DoubleEndedFallibleIterator for Iter<'this, L>
where
    L: DoubleEndedFallibleLender,
    for<'all> FallibleLend<'all, L>: 'this,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        Ok(
            // SAFETY: for<'all> FallibleLend<'all, L>: 'this
            unsafe {
                core::mem::transmute::<Option<FallibleLend<'_, L>>, Option<FallibleLend<'this, L>>>(
                    self.lender.next_back()?,
                )
            },
        )
    }
}
