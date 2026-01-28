use core::ops::ControlFlow;

use crate::{
    try_trait_v2::Try, Chunk, ExactSizeFallibleLender, ExactSizeLender, FallibleLend, FallibleLender, FallibleLending,
    FusedFallibleLender, FusedLender, Lend, Lender, Lending,
};

/// A lender yielding lenders returning the next `chunk_size` lends.
///
/// This is the closest lending approximation to [`core::iter::ArrayChunks`], as
/// we cannot accumulate the lends into an array.
///
/// This struct is created by [`chunky`][Lender::chunky] or
/// [`chunky`][FallibleLender::chunky].
#[derive(Debug, Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Chunky<L> {
    lender: L,
    len: usize,
    chunk_size: usize,
}

impl<L> Chunky<L>
where
    L: Lender + ExactSizeLender,
{
    pub(crate) fn new(lender: L, chunk_size: usize) -> Self {
        assert!(chunk_size != 0, "chunk size must be non-zero");
        let mut len = lender.len();
        let rem = len % chunk_size;
        len /= chunk_size;
        if rem > 0 {
            len += 1;
        }
        Self { lender, chunk_size, len }
    }
}

impl<L> Chunky<L>
where
    L: FallibleLender + ExactSizeFallibleLender,
{
    pub(crate) fn new_fallible(lender: L, chunk_size: usize) -> Self {
        assert!(chunk_size != 0, "chunk size must be non-zero");
        let mut len = lender.len();
        let rem = len % chunk_size;
        len /= chunk_size;
        if rem > 0 {
            len += 1;
        }
        Self { lender, chunk_size, len }
    }
}

impl<L> Chunky<L> {
    pub fn into_inner(self) -> L {
        self.lender
    }
    pub fn into_parts(self) -> (L, usize) {
        (self.lender, self.chunk_size)
    }
}

impl<'lend, L> Lending<'lend> for Chunky<L>
where
    L: Lender,
{
    type Lend = Chunk<'lend, L>;
}

impl<L> Lender for Chunky<L>
where
    L: Lender,
{
    // SAFETY: the lend is a Chunk wrapping L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.lender.next_chunk(self.chunk_size))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut lower, mut upper) = self.lender.size_hint();
        let sz = self.chunk_size;

        let lrem = lower % sz;
        lower /= sz;
        if lrem > 0 {
            lower += 1;
        }

        upper = upper.map(|mut n| {
            let urem = n % sz;
            n /= sz;
            if urem > 0 {
                n += 1;
            }
            n
        });

        (lower, upper)
    }

    #[inline]
    fn count(self) -> usize {
        let mut cnt = self.lender.count();
        let sz = self.chunk_size;

        let rem = cnt % sz;
        cnt /= sz;
        if rem > 0 {
            cnt += 1;
        }
        cnt
    }

    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut acc = init;
        let sz = self.chunk_size;
        while self.len > 0 {
            self.len -= 1;
            acc = match f(acc, self.lender.next_chunk(sz)).branch() {
                ControlFlow::Break(x) => return R::from_residual(x),
                ControlFlow::Continue(x) => x,
            };
        }
        R::from_output(acc)
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut accum = init;
        let sz = self.chunk_size;
        while self.len > 0 {
            self.len -= 1;
            accum = f(accum, self.lender.next_chunk(sz));
        }
        accum
    }
}

impl<L> FusedLender for Chunky<L> where L: FusedLender {}

impl<L> ExactSizeLender for Chunky<L>
where
    L: Lender,
{
    #[inline]
    fn len(&self) -> usize {
        let mut len = self.len;
        let sz = self.chunk_size;

        let rem = len % sz;
        len /= sz;
        if rem > 0 {
            len += 1;
        }
        len
    }
}

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
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut lower, mut upper) = self.lender.size_hint();
        let sz = self.chunk_size;

        let lrem = lower % sz;
        lower /= sz;
        if lrem > 0 {
            lower += 1;
        }

        upper = upper.map(|mut n| {
            let urem = n % sz;
            n /= sz;
            if urem > 0 {
                n += 1;
            }
            n
        });

        (lower, upper)
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error> {
        let mut cnt = self.lender.count()?;
        let sz = self.chunk_size;

        let rem = cnt % sz;
        cnt /= sz;
        if rem > 0 {
            cnt += 1;
        }
        Ok(cnt)
    }

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

impl<L> ExactSizeFallibleLender for Chunky<L>
where
    L: FallibleLender,
{
    #[inline]
    fn len(&self) -> usize {
        let mut len = self.len;
        let sz = self.chunk_size;

        let rem = len % sz;
        len /= sz;
        if rem > 0 {
            len += 1;
        }
        len
    }
}
