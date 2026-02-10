use core::ops::ControlFlow;

use crate::{
    Chunk, ExactSizeFallibleLender, ExactSizeLender, FallibleLender, FusedLender, Lend, Lender,
    Lending, try_trait_v2::Try,
};

/// A lender yielding lenders ([`Chunk`]s) returning the next
/// `chunk_size` lends.
///
/// This is the closest lending approximation to
/// `core::iter::ArrayChunks` (unstable), as we cannot accumulate
/// the lends into an array. Unlike `ArrayChunks`, which yields
/// fixed-size arrays, `Chunky` yields [`Chunk`] lenders that
/// must be consumed to access the elements.
///
/// This struct is created by [`Lender::chunky`] or
/// [`FallibleLender::chunky`].
///
/// # Important: Partial Chunk Consumption
///
/// **Each [`Chunk`] yielded by `Chunky` must be fully consumed
/// before requesting the next chunk.** If a chunk is not fully
/// consumed, the unconsumed elements are effectively skipped,
/// and the next chunk will start from whatever position the
/// underlying lender is at.
///
/// This behavior differs from [`core::slice::Chunks`] where
/// each chunk is a complete view. With `Chunky`, you are
/// borrowing from a single underlying lender, so partial
/// consumption affects subsequent chunks.
///
/// Partial chunk consumption also has the consequence of not
/// enumerating entirely the elements returned by the underlying
/// lender, as the number of chunks is computed at the start.
/// Thus, in case of partial chunk consumption the last element
/// of the last chunk will not be the last element of the underlying
/// lender.
#[derive(Debug, Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Chunky<L> {
    pub(crate) lender: L,
    pub(crate) chunk_size: usize,
    pub(crate) len: usize,
}

impl<L> Chunky<L>
where
    L: Lender + ExactSizeLender,
{
    #[inline]
    pub(crate) fn new(lender: L, chunk_size: usize) -> Self {
        assert!(chunk_size != 0, "chunk size must be non-zero");
        let len = lender.len().div_ceil(chunk_size);
        Self {
            lender,
            chunk_size,
            len,
        }
    }
}

impl<L> Chunky<L>
where
    L: FallibleLender + ExactSizeFallibleLender,
{
    #[inline]
    pub(crate) fn new_fallible(lender: L, chunk_size: usize) -> Self {
        assert!(chunk_size != 0, "chunk size must be non-zero");
        let len = lender.len().div_ceil(chunk_size);
        Self {
            lender,
            chunk_size,
            len,
        }
    }
}

impl<L> Chunky<L> {
    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the chunk size.
    #[inline(always)]
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
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        if n < self.len {
            // Skip n chunks by advancing the inner lender
            let skip = n
                .checked_mul(self.chunk_size)
                .expect("overflow in Chunky::nth");
            self.len -= n;
            if self.lender.advance_by(skip).is_err() {
                unreachable!();
            }
            self.next()
        } else {
            // Exhaust
            if self.len > 0 {
                let skip = self
                    .len
                    .checked_mul(self.chunk_size)
                    .expect("overflow in Chunky::nth");
                let _ = self.lender.advance_by(skip);
                self.len = 0;
            }
            None
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.len
    }

    #[inline]
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

// Note: Chunky deliberately does not implement ExactSizeLender (nor
// ExactSizeFallibleLender). The `len` field is pre-computed from the
// underlying lender's length at construction time and counts *chunks*,
// not elements. If a chunk is only partially consumed, the remaining
// elements are silently skipped when the next chunk is requested, so
// the pre-computed count may overestimate the number of lends that
// `next()` will actually produce. This violates the ExactSizeLender
// contract, which requires `len()` to match the actual remaining
// count. The chunk count is still available through `size_hint()`.
