use core::ops::ControlFlow;

use crate::{try_trait_v2::Try, Chunk, ExactSizeLender, FusedLender, Lender, Lending};

/// A lender over big, plumpy, chunky chunks of elements of the lender at a time.
///
/// The chunks do not overlap.
///
/// This `struct` is created by [`chunky`][Lender::array_chunks]
/// method on [`Lender`]. See its documentation for more.
#[derive(Debug, Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Chunky<L: Lender> {
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
        let len = lender.len();
        Self { lender, chunk_size, len }
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
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        self.len = self.len.saturating_sub(1);
        if self.len == 0 {
            None
        } else {
            Some(self.lender.next_chunk(self.chunk_size))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.lender.size_hint();
        let sz = self.chunk_size;
        (
            if lower % sz == 0 { lower / sz } else { (lower / sz) + 1 },
            upper.map(|n| if n % sz == 0 { n / sz } else { (n / sz) + 1 }),
        )
    }

    #[inline]
    fn count(self) -> usize {
        let cnt = self.lender.count();
        let sz = self.chunk_size;
        if cnt % sz == 0 {
            cnt / sz
        } else {
            (cnt / sz) + 1
        }
    }

    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> R,
        R: Try<Output = B>,
    {
        let mut acc = init;
        let sz = self.chunk_size;
        for _ in 0..self.len {
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
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        let mut accum = init;
        let sz = self.chunk_size;
        for _ in 0..self.len {
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
        let sz = self.chunk_size;
        let len = self.len;
        if len % sz == 0 {
            len / sz
        } else {
            (len / sz) + 1
        }
    }
}
