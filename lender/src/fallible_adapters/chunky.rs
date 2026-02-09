use crate::{
    Chunk, Chunky, FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender,
    try_trait_v2::Try,
};
use core::ops::ControlFlow;

impl<'lend, L> FallibleLending<'lend> for Chunky<L>
where
    L: FallibleLender,
{
    type Lend = Chunk<'lend, L>;
}

impl<L> FallibleLender for Chunky<L>
where
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is a Chunk wrapping L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.len > 0 {
            self.len -= 1;
            Ok(Some(self.lender.next_chunk(self.chunk_size)))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if n < self.len {
            // Skip n chunks by advancing the inner lender
            let skip = n * self.chunk_size;
            self.len -= n;
            if self.lender.advance_by(skip)?.is_err() {
                unreachable!();
            }
            self.next()
        } else {
            // Exhaust
            if self.len > 0 {
                let skip = self.len * self.chunk_size;
                let _ = self.lender.advance_by(skip)?;
                self.len = 0;
            }
            Ok(None)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    #[inline(always)]
    fn count(self) -> Result<usize, Self::Error> {
        Ok(self.len)
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let mut acc = init;
        let sz = self.chunk_size;
        while self.len > 0 {
            self.len -= 1;
            acc = match f(acc, self.lender.next_chunk(sz))?.branch() {
                ControlFlow::Break(x) => return Ok(R::from_residual(x)),
                ControlFlow::Continue(x) => x,
            };
        }
        Ok(R::from_output(acc))
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut accum = init;
        let sz = self.chunk_size;
        while self.len > 0 {
            self.len -= 1;
            accum = f(accum, self.lender.next_chunk(sz))?;
        }
        Ok(accum)
    }
}

impl<L> FusedFallibleLender for Chunky<L> where L: FusedFallibleLender {}
