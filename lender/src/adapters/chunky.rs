use core::ops::ControlFlow;

use crate::{
    Chunk, ExactSizeFallibleLender, ExactSizeLender, FallibleLend, FallibleLender, FallibleLending,
    FusedFallibleLender, FusedLender, Lend, Lender, Lending, try_trait_v2::Try,
};

/// A lender yielding lenders ([`Chunk`]s) returning the next `chunk_size` lends.
///
/// This is the closest lending approximation to [`core::iter::ArrayChunks`], as
/// we cannot accumulate the lends into an array. Unlike [`ArrayChunks`](core::iter::ArrayChunks), which
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
    lender: L,
    done: bool,
    chunk_size: usize,
}

impl<L> Chunky<L>
where
    L: Lender,
{
    pub(crate) fn new(lender: L, chunk_size: usize) -> Self {
        assert!(chunk_size != 0, "chunk size must be non-zero");
        Self {
            lender,
            chunk_size,
            done: false,
        }
    }
}

impl<L> Chunky<L>
where
    L: FallibleLender,
{
    pub(crate) fn new_fallible(lender: L, chunk_size: usize) -> Self {
        assert!(chunk_size != 0, "chunk size must be non-zero");
        Self {
            lender,
            chunk_size,
            done: false,
        }
    }
}

/// Helper to compute the number of chunks from a number of elements and a chunk size.
#[inline]
fn div_ceil(n: usize, chunk_size: usize) -> usize {
    let q = n / chunk_size;
    if n % chunk_size > 0 { q + 1 } else { q }
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
        if self.done {
            return None;
        }
        // Peek at the underlying lender via size_hint to check if exhausted.
        // This works correctly because Chunk fully consumes its allocation
        // before the next call to Chunky::next().
        let hint = self.lender.size_hint();
        if hint.1 == Some(0) {
            self.done = true;
            return None;
        }
        Some(self.lender.next_chunk(self.chunk_size))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }
        let (lower, upper) = self.lender.size_hint();
        (div_ceil(lower, self.chunk_size), upper.map(|n| div_ceil(n, self.chunk_size)))
    }

    #[inline]
    fn count(self) -> usize {
        if self.done { return 0; }
        div_ceil(self.lender.count(), self.chunk_size)
    }

    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut acc = init;
        let sz = self.chunk_size;
        while !self.done {
            if self.lender.size_hint().1 == Some(0) {
                self.done = true;
                break;
            }
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
        while !self.done {
            if self.lender.size_hint().1 == Some(0) {
                self.done = true;
                break;
            }
            accum = f(accum, self.lender.next_chunk(sz));
        }
        accum
    }
}

impl<L> FusedLender for Chunky<L> where L: FusedLender {}

impl<L> ExactSizeLender for Chunky<L>
where
    L: ExactSizeLender,
{
    #[inline]
    fn len(&self) -> usize {
        if self.done { return 0; }
        div_ceil(self.lender.len(), self.chunk_size)
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
        if self.done {
            return Ok(None);
        }
        let hint = self.lender.size_hint();
        if hint.1 == Some(0) {
            self.done = true;
            return Ok(None);
        }
        Ok(Some(self.lender.next_chunk(self.chunk_size)))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }
        let (lower, upper) = self.lender.size_hint();
        (div_ceil(lower, self.chunk_size), upper.map(|n| div_ceil(n, self.chunk_size)))
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error> {
        if self.done { return Ok(0); }
        Ok(div_ceil(self.lender.count()?, self.chunk_size))
    }

    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let mut acc = init;
        let sz = self.chunk_size;
        while !self.done {
            if self.lender.size_hint().1 == Some(0) {
                self.done = true;
                break;
            }
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
        while !self.done {
            if self.lender.size_hint().1 == Some(0) {
                self.done = true;
                break;
            }
            accum = f(accum, self.lender.next_chunk(sz))?;
        }
        Ok(accum)
    }
}

impl<L> FusedFallibleLender for Chunky<L> where L: FusedFallibleLender {}

impl<L> ExactSizeFallibleLender for Chunky<L>
where
    L: ExactSizeFallibleLender,
{
    #[inline]
    fn len(&self) -> usize {
        if self.done { return 0; }
        div_ceil(self.lender.len(), self.chunk_size)
    }
}
