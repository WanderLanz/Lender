use crate::{
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender,
    Lending,
};

#[derive(Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Chunk<'s, T> {
    lender: &'s mut T,
    len: usize,
}
impl<'s, T> Chunk<'s, T> {
    pub(crate) fn new(lender: &'s mut T, len: usize) -> Self {
        Self { lender, len }
    }

    pub fn into_inner(self) -> &'s mut T {
        self.lender
    }

    pub fn into_parts(self) -> (&'s mut T, usize) {
        (self.lender, self.len)
    }
}
impl<'lend, T> Lending<'lend> for Chunk<'_, T>
where
    T: Lender,
{
    type Lend = Lend<'lend, T>;
}
impl<T> Lender for Chunk<'_, T>
where
    T: Lender,
{
    // SAFETY: the lend is that of T
    crate::unsafe_assume_covariance!();
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            self.lender.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.lender.size_hint();
        (lower.min(self.len), upper.map(|x| x.min(self.len)))
    }
}
impl<L> FusedLender for Chunk<'_, L> where L: FusedLender {}

impl<'lend, T> FallibleLending<'lend> for Chunk<'_, T>
where
    T: FallibleLender,
{
    type Lend = FallibleLend<'lend, T>;
}
impl<T> FallibleLender for Chunk<'_, T>
where
    T: FallibleLender,
{
    type Error = T::Error;
    // SAFETY: the lend is that of T
    crate::unsafe_assume_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            self.lender.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.lender.size_hint();
        (lower.min(self.len), upper.map(|x| x.min(self.len)))
    }
}
impl<L> FusedFallibleLender for Chunk<'_, L> where L: FusedFallibleLender {}
