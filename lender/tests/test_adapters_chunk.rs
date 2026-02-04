//! Tests for chunking adapters: Peekable, Chunk, Chunky

#![allow(clippy::unnecessary_fold)]

mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// Peekable adapter tests (Lender)
// Semantics: peek() returns a reference to the next element without consuming it.
// ============================================================================

#[test]
fn peekable_basic() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();

    // peek() returns reference to next element
    assert_eq!(peekable.peek(), Some(&&1));
    // peek() again returns same element (not consumed)
    assert_eq!(peekable.peek(), Some(&&1));
    // next() consumes it
    assert_eq!(peekable.next(), Some(&1));

    // Now peek sees 2
    assert_eq!(peekable.peek(), Some(&&2));
    assert_eq!(peekable.next(), Some(&2));

    // Continue
    assert_eq!(peekable.peek(), Some(&&3));
    assert_eq!(peekable.next(), Some(&3));

    // Exhausted
    assert_eq!(peekable.peek(), None);
    assert_eq!(peekable.next(), None);
}

#[test]
fn peekable_peek_mut() {
    // Note: VecLender now yields &i32, so peek_mut() returns &mut &i32.
    // We can't modify the underlying value, but can still test peek_mut exists
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();

    // peek_mut returns a mutable reference to the lend
    assert!(peekable.peek_mut().is_some());
    assert_eq!(peekable.next(), Some(&1));
    assert_eq!(peekable.next(), Some(&2));
}

#[test]
fn peekable_next_if() {
    let mut peekable = VecLender::new(vec![1, 2, 3, 4]).peekable();

    // next_if returns Some if predicate matches
    assert_eq!(peekable.next_if(|x| **x == 1), Some(&1));
    // next_if returns None if predicate doesn't match, element is put back
    assert_eq!(peekable.next_if(|x| **x == 10), None);
    // Element wasn't consumed
    assert_eq!(peekable.peek(), Some(&&2));
    assert_eq!(peekable.next(), Some(&2));
}

#[test]
fn peekable_next_if_eq() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();

    // next_if_eq returns Some if element equals given value
    assert_eq!(peekable.next_if_eq(&&1), Some(&1));
    // next_if_eq returns None if not equal
    assert_eq!(peekable.next_if_eq(&&10), None);
    assert_eq!(peekable.next(), Some(&2));
}

#[test]
fn peekable_empty() {
    let mut peekable = VecLender::new(vec![]).peekable();
    assert_eq!(peekable.peek(), None);
    assert_eq!(peekable.next(), None);
}

#[test]
fn peekable_count() {
    // count() should include peeked element
    let mut peekable = VecLender::new(vec![1, 2, 3, 4, 5]).peekable();
    peekable.peek(); // peek first element
    assert_eq!(peekable.count(), 5);
}

#[test]
fn peekable_nth() {
    let mut peekable = VecLender::new(vec![1, 2, 3, 4, 5]).peekable();

    // nth(2) should skip 1, 2 and return 3
    assert_eq!(peekable.nth(2), Some(&3));
    assert_eq!(peekable.next(), Some(&4));
}

#[test]
fn peekable_last() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.last(), Some(&3));
}

#[test]
fn peekable_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4])
        .peekable()
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 10);
}

#[test]
fn peekable_size_hint() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.size_hint(), (3, Some(3)));

    peekable.peek();
    // After peek, size_hint should still be correct
    assert_eq!(peekable.size_hint(), (3, Some(3)));

    peekable.next();
    assert_eq!(peekable.size_hint(), (2, Some(2)));
}

#[test]
fn peekable_into_inner() {
    let peekable = VecLender::new(vec![1, 2, 3]).peekable();
    let inner = peekable.into_inner();
    assert_eq!(inner.data, vec![1, 2, 3]);
}

#[test]
fn peekable_peek_multiple() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.peek(), Some(&&1));
    assert_eq!(peekable.peek(), Some(&&1)); // Peeking again returns same value
    assert_eq!(peekable.next(), Some(&1));
    assert_eq!(peekable.peek(), Some(&&2));
}

#[test]
fn peekable_peek_mut_additional() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    // VecLender yields &i32, so peek_mut returns &mut &i32 - can't modify underlying value
    assert!(peekable.peek_mut().is_some());
    assert_eq!(peekable.next(), Some(&1));
    assert_eq!(peekable.next(), Some(&2)); // Original unchanged
}

#[test]
fn peekable_next_if_additional() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.next_if(|x| **x < 2), Some(&1));
    assert_eq!(peekable.next_if(|x| **x < 2), None); // 2 is not < 2
    assert_eq!(peekable.next(), Some(&2));
}

#[test]
fn peekable_next_if_eq_additional() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.next_if_eq(&&1), Some(&1));
    assert_eq!(peekable.next_if_eq(&&1), None); // Next is 2, not 1
    assert_eq!(peekable.next_if_eq(&&2), Some(&2));
}

#[test]
fn peekable_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3])
        .peekable()
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 6);
}

#[test]
fn peekable_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .peekable()
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(6));
}

#[test]
fn peekable_size_hint_after_peek() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.size_hint(), (3, Some(3)));
    peekable.peek();
    // After peeking, size_hint should still be accurate
    assert_eq!(peekable.size_hint(), (3, Some(3)));
    peekable.next();
    assert_eq!(peekable.size_hint(), (2, Some(2)));
}

// Peekable::nth with peeked value when n == 0 (covers unsafe transmute in nth)
#[test]
fn peekable_nth_zero_with_peeked() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    // Peek to store a value
    assert_eq!(peekable.peek(), Some(&&1));
    // nth(0) should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.nth(0), Some(&1));
    assert_eq!(peekable.next(), Some(&2));
}

// Peekable::last with peeked value (covers unsafe transmute in last)
#[test]
fn peekable_last_with_peeked_only() {
    let mut peekable = VecLender::new(vec![1]).peekable();
    // Peek the only value
    assert_eq!(peekable.peek(), Some(&&1));
    // last() should return the peeked value through the unsafe transmute path
    // when the underlying lender returns None
    assert_eq!(peekable.last(), Some(&1));
}

// Peekable::next_back with peeked value when underlying lender is empty
// (covers unsafe transmute in next_back)
#[test]
fn peekable_next_back_with_peeked_exhausted() {
    use lender::DoubleEndedLender;

    let mut peekable = VecLender::new(vec![1]).peekable();
    // Peek the only value
    assert_eq!(peekable.peek(), Some(&&1));
    // next_back should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.next_back(), Some(&1));
    assert_eq!(peekable.next(), None);
}

#[test]
fn peekable_rfold_with_peeked() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.peek(), Some(&&1));
    // rfold processes back-to-front: 3, 2, then peeked 1
    let result = peekable.rfold(Vec::new(), |mut acc, &x| {
        acc.push(x);
        acc
    });
    assert_eq!(result, vec![3, 2, 1]);
}

#[test]
fn peekable_try_rfold_with_peeked_complete() {
    use lender::DoubleEndedLender;

    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.peek(), Some(&&1));
    // try_rfold processes back-to-front: 3, 2, then peeked 1
    let result: Option<Vec<i32>> = peekable.try_rfold(Vec::new(), |mut acc, &x| {
        acc.push(x);
        Some(acc)
    });
    assert_eq!(result, Some(vec![3, 2, 1]));
}

// Covers the ControlFlow::Break path in try_rfold where the peeked value
// is stored back.
#[test]
fn peekable_try_rfold_with_peeked_break() {
    use lender::DoubleEndedLender;

    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.peek(), Some(&&1));
    // Inner lender has [2, 3]. try_rfold processes back-to-front:
    // 3 (continue, acc=3), then 2 (break via None).
    let result: Option<i32> =
        peekable.try_rfold(0, |acc, &x| if x == 2 { None } else { Some(acc + x) });
    assert_eq!(result, None);
    // The peeked value should have been stored back
    assert_eq!(peekable.next(), Some(&1));
    // Inner lender was fully consumed by try_rfold
    assert_eq!(peekable.next(), None);
}

// ============================================================================
// Chunk adapter tests
// ============================================================================

#[test]
fn chunk_basic() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let mut chunk = lender.next_chunk(3);

    assert_eq!(chunk.next(), Some(&1));
    assert_eq!(chunk.next(), Some(&2));
    assert_eq!(chunk.next(), Some(&3));
    assert_eq!(chunk.next(), None);

    // Remaining elements
    assert_eq!(lender.next(), Some(&4));
    assert_eq!(lender.next(), Some(&5));
}

#[test]
fn chunk_larger_than_remaining() {
    let mut lender = VecLender::new(vec![1, 2]);
    let mut chunk = lender.next_chunk(5);

    assert_eq!(chunk.next(), Some(&1));
    assert_eq!(chunk.next(), Some(&2));
    assert_eq!(chunk.next(), None);
}

#[test]
fn chunk_empty_lender() {
    let mut lender = VecLender::new(vec![]);
    let mut chunk = lender.next_chunk(3);
    assert_eq!(chunk.next(), None);
}

#[test]
fn chunk_size_hint() {
    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(3);
    let chunk = chunky.next().unwrap();
    // Chunk has max 3 elements, underlying has 5
    assert_eq!(chunk.size_hint(), (3, Some(3)));
}

#[test]
fn chunk_into_parts() {
    let mut chunky = VecLender::new(vec![1, 2, 3]).chunky(2);
    let chunk = chunky.next().unwrap();
    let (lender, remaining) = chunk.into_parts();
    assert_eq!(remaining, 2);
    assert_eq!(lender.count(), 3); // Original lender still has elements
}

// ============================================================================
// Chunky adapter tests
// Semantics: yields lenders that each return chunk_size elements
// ============================================================================

#[test]
fn chunky_basic() {
    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);

    // First chunk: 1, 2
    let mut chunk1 = chunky.next().unwrap();
    assert_eq!(chunk1.next(), Some(&1));
    assert_eq!(chunk1.next(), Some(&2));
    assert_eq!(chunk1.next(), None);

    // Second chunk: 3, 4
    let mut chunk2 = chunky.next().unwrap();
    assert_eq!(chunk2.next(), Some(&3));
    assert_eq!(chunk2.next(), Some(&4));
    assert_eq!(chunk2.next(), None);

    // Third chunk: 5, 6
    let mut chunk3 = chunky.next().unwrap();
    assert_eq!(chunk3.next(), Some(&5));
    assert_eq!(chunk3.next(), Some(&6));
    assert_eq!(chunk3.next(), None);

    // No more chunks
    assert!(chunky.next().is_none());
}

#[test]
fn chunky_uneven() {
    // 5 elements with chunk size 2 = 3 chunks (2, 2, 1)
    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(2);

    let mut chunk1 = chunky.next().unwrap();
    assert_eq!(chunk1.next(), Some(&1));
    assert_eq!(chunk1.next(), Some(&2));
    assert_eq!(chunk1.next(), None);

    let mut chunk2 = chunky.next().unwrap();
    assert_eq!(chunk2.next(), Some(&3));
    assert_eq!(chunk2.next(), Some(&4));
    assert_eq!(chunk2.next(), None);

    // Last chunk has only 1 element
    let mut chunk3 = chunky.next().unwrap();
    assert_eq!(chunk3.next(), Some(&5));
    assert_eq!(chunk3.next(), None);

    assert!(chunky.next().is_none());
}

#[test]
fn chunky_size_hint() {
    let chunky = VecLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    assert_eq!(chunky.size_hint(), (3, Some(3)));

    let chunky2 = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(2);
    assert_eq!(chunky2.size_hint(), (3, Some(3))); // ceil(5/2) = 3
}

#[test]
fn chunky_count() {
    let chunky = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7]).chunky(3);
    assert_eq!(chunky.count(), 3); // ceil(7/3) = 3
}

#[test]
fn chunky_fold() {
    // Count number of chunks
    let num_chunks = VecLender::new(vec![1, 2, 3, 4, 5])
        .chunky(2)
        .fold(0, |acc, _chunk| acc + 1);
    assert_eq!(num_chunks, 3);
}

#[test]
fn chunky_size_hint_decreases() {
    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    // 6 elements, chunk_size 2 -> 3 chunks
    assert_eq!(chunky.size_hint(), (3, Some(3)));
    chunky.next();
    assert_eq!(chunky.size_hint(), (2, Some(2)));
    chunky.next();
    assert_eq!(chunky.size_hint(), (1, Some(1)));
    chunky.next();
    assert_eq!(chunky.size_hint(), (0, Some(0)));

    // With uneven division: 5 elements, chunk_size 2 -> ceil(5/2) = 3 chunks
    let chunky2 = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(2);
    assert_eq!(chunky2.size_hint(), (3, Some(3)));
}

#[test]
fn chunky_try_fold() {
    // Test try_fold - if we only consume part of each chunk, the unconsumed
    // elements are lost and the next chunk starts from the current position.
    // This tests the "partial consumption" case.
    let result: Option<i32> =
        VecLender::new(vec![1, 2, 3, 4, 5, 6])
            .chunky(2)
            .try_fold(0, |acc, mut chunk| {
                // Only consume one element from each chunk
                if let Some(first) = chunk.next() {
                    Some(acc + first)
                } else {
                    Some(acc)
                }
            });
    // Since we only consume 1 element per chunk iteration:
    // - Iteration 1: chunk.next() gets 1
    // - Iteration 2: chunk.next() gets 2 (not 3, because we didn't consume element 2)
    // - Iteration 3: chunk.next() gets 3
    // This is expected behavior - Chunky tracks the number of chunks to yield,
    // but doesn't force consumption of all elements in each chunk.
    assert_eq!(result, Some(6)); // 1 + 2 + 3 = 6
}

#[test]
fn chunky_try_fold_full_consumption() {
    // When each chunk is fully consumed, we get the expected chunked results
    let result: Option<i32> =
        VecLender::new(vec![1, 2, 3, 4, 5, 6])
            .chunky(2)
            .try_fold(0, |acc, chunk| {
                // Consume all elements in the chunk
                let chunk_sum = chunk.fold(0, |a, x| a + x);
                Some(acc + chunk_sum)
            });
    // Chunks: [1,2], [3,4], [5,6] with full consumption
    // Sums: 3, 7, 11 -> total: 21
    assert_eq!(result, Some(21));
}

#[test]
fn chunky_into_parts() {
    let chunky = VecLender::new(vec![1, 2, 3]).chunky(2);
    let (lender, chunk_size) = chunky.into_parts();
    assert_eq!(chunk_size, 2);
    assert_eq!(lender.count(), 3);
}

#[test]
fn chunky_into_inner() {
    let chunky = VecLender::new(vec![1, 2, 3]).chunky(2);
    let lender = chunky.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
#[should_panic(expected = "chunk size must be non-zero")]
fn chunky_zero_panics() {
    let _ = VecLender::new(vec![1, 2, 3]).chunky(0);
}

