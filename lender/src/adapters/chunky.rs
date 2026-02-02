use core::ops::ControlFlow;

use crate::{
    Chunk, ExactSizeFallibleLender, ExactSizeLender, FallibleLender, FusedLender, Lend, Lender,
    Lending, try_trait_v2::Try,
};

/// A lender yielding lenders ([`Chunk`]s) returning the next `chunk_size` lends.
///
/// This is the closest lending approximation to `core::iter::ArrayChunks` (unstable), as
/// we cannot accumulate the lends into an array. Unlike `ArrayChunks`, which
/// yields fixed-size arrays, `Chunky` yields [`Chunk`] lenders that must be
/// consumed to access the elements.
///
/// # Important: Partial Chunk Consumption
///
/// **Each [`Chunk`] yielded by `Chunky` must be fully consumed before requesting
/// the next chunk.** If a chunk is not fully consumed, the unconsumed elements
/// are effectively skipped, and the next chunk will start from whatever position
/// the underlying lender is at.
///
/// This behavior differs from [`core::slice::Chunks`] where each chunk is a
/// complete view. With `Chunky`, you are borrowing from a single underlying
/// lender, so partial consumption affects subsequent chunks.
///
/// # Examples
///
/// Correct usage (fully consuming each chunk):
/// ```rust
/// # use lender::prelude::*;
/// let mut data = [1, 2, 3, 4, 5, 6];
/// let mut chunky = lender::windows_mut(&mut data, 1).chunky(2);
///
/// // First chunk: elements 1, 2
/// let mut chunk1 = chunky.next().unwrap();
/// assert_eq!(chunk1.next().map(|s| s[0]), Some(1));
/// assert_eq!(chunk1.next().map(|s| s[0]), Some(2));
/// assert_eq!(chunk1.next(), None); // Chunk exhausted
///
/// // Second chunk: elements 3, 4
/// let mut chunk2 = chunky.next().unwrap();
/// assert_eq!(chunk2.next().map(|s| s[0]), Some(3));
/// assert_eq!(chunk2.next().map(|s| s[0]), Some(4));
/// ```
///
/// Partial consumption (demonstrating the behavior):
/// ```rust
/// # use lender::prelude::*;
/// let data = vec![1, 2, 3, 4, 5, 6];
/// // Sum the first element of each chunk (partial consumption)
/// let sum = lender::from_iter(data.iter())
///     .chunky(2)
///     .fold(0, |acc, mut chunk| {
///         // Only consume first element of each chunk
///         acc + chunk.next().copied().unwrap_or(0)
///     });
/// // Since we only consume 1 element per chunk iteration,
/// // and chunky tracks chunk count (not element position),
/// // we get: 1 (chunk 1), 2 (chunk 2), 3 (chunk 3) = 6
/// // NOT: 1, 3, 5 as you might expect!
/// assert_eq!(sum, 6);
/// ```
///
/// This struct is created by [`Lender::chunky`](crate::Lender::chunky) or
/// [`FallibleLender::chunky`](crate::FallibleLender::chunky).
#[derive(Debug, Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Chunky<L> {
    pub(crate) lender: L,
    pub(crate) len: usize,
    pub(crate) chunk_size: usize,
}

impl<L> Chunky<L>
where
    L: Lender + ExactSizeLender,
{
    #[inline(always)]
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
    #[inline(always)]
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
