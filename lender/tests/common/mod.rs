#![allow(dead_code)]

use ::lender::prelude::*;

// ============================================================================
// Helper struct for Lender tests — yields &i32 references, exercising
// the core lending pattern (items that borrow from the lender).
// ============================================================================
#[derive(Clone)]
pub struct VecLender {
    pub data: Vec<i32>,
    pub front: usize,
    pub back: usize,
}

impl VecLender {
    pub fn new(data: Vec<i32>) -> Self {
        let len = data.len();
        Self {
            data,
            front: 0,
            back: len,
        }
    }
}

impl<'lend> Lending<'lend> for VecLender {
    type Lend = &'lend i32;
}

impl Lender for VecLender {
    // SAFETY: &'lend i32 is covariant in 'lend
    unsafe_assume_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.front < self.back {
            let item = &self.data[self.front];
            self.front += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl DoubleEndedLender for VecLender {
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        if self.front < self.back {
            self.back -= 1;
            Some(&self.data[self.back])
        } else {
            None
        }
    }
}

impl ExactSizeLender for VecLender {
    fn len(&self) -> usize {
        self.back - self.front
    }
}

impl lender::FusedLender for VecLender {}

pub struct WindowsMut<'a, T> {
    pub slice: &'a mut [T],
    pub begin: usize,
    pub len: usize,
}
impl<'lend, T> Lending<'lend> for WindowsMut<'_, T> {
    type Lend = &'lend mut [T];
}
impl<T> Lender for WindowsMut<'_, T> {
    check_covariance!();
    fn next(&mut self) -> Option<&mut [T]> {
        let begin = self.begin;
        self.begin = self.begin.saturating_add(1);
        self.slice.get_mut(begin..begin + self.len)
    }
}

// Helper struct for testing fallible traits — yields &i32 references.
pub struct VecFallibleLender {
    data: Vec<i32>,
    front: usize,
    back: usize,
}

impl VecFallibleLender {
    pub fn new(data: Vec<i32>) -> Self {
        let len = data.len();
        Self {
            data,
            front: 0,
            back: len,
        }
    }
}

impl<'lend> lender::FallibleLending<'lend> for VecFallibleLender {
    type Lend = &'lend i32;
}

impl lender::FallibleLender for VecFallibleLender {
    type Error = std::convert::Infallible;
    // SAFETY: &'lend i32 is covariant in 'lend
    unsafe_assume_covariance_fallible!();

    fn next(&mut self) -> Result<Option<lender::FallibleLend<'_, Self>>, Self::Error> {
        if self.front < self.back {
            let item = &self.data[self.front];
            self.front += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl lender::DoubleEndedFallibleLender for VecFallibleLender {
    fn next_back(&mut self) -> Result<Option<lender::FallibleLend<'_, Self>>, Self::Error> {
        if self.front < self.back {
            self.back -= 1;
            Ok(Some(&self.data[self.back]))
        } else {
            Ok(None)
        }
    }
}

impl lender::ExactSizeFallibleLender for VecFallibleLender {
    fn len(&self) -> usize {
        self.back - self.front
    }
}

impl lender::FusedFallibleLender for VecFallibleLender {}

/// A fallible lender that yields `&i32` references and errors at a specific index.
/// Elements before `error_at` yield `Ok(Some(&data[i]))`,
/// the element at `error_at` yields `Err("error at index {error_at}")`,
/// and elements after continue normally (the error is not terminal).
pub struct ErrorAtLender {
    pub data: Vec<i32>,
    pub front: usize,
    pub error_at: usize,
}

impl ErrorAtLender {
    pub fn new(data: Vec<i32>, error_at: usize) -> Self {
        Self {
            data,
            front: 0,
            error_at,
        }
    }
}

impl<'lend> lender::FallibleLending<'lend> for ErrorAtLender {
    type Lend = &'lend i32;
}

impl lender::FallibleLender for ErrorAtLender {
    type Error = String;
    unsafe_assume_covariance_fallible!();

    fn next(&mut self) -> Result<Option<lender::FallibleLend<'_, Self>>, Self::Error> {
        if self.front >= self.data.len() {
            Ok(None)
        } else if self.front == self.error_at {
            self.front += 1;
            Err(format!("error at index {}", self.error_at))
        } else {
            let item = &self.data[self.front];
            self.front += 1;
            Ok(Some(item))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.data.len() - self.front;
        (0, Some(remaining))
    }
}

/// A collector that copies `&i32` lend values into a `Vec<i32>`.
/// Used for testing `collect_into` (which requires `ExtendLender`).
#[derive(Debug)]
pub struct I32Collector(pub Vec<i32>);

impl<L: lender::IntoLender> lender::ExtendLender<L> for I32Collector
where
    L::Lender: for<'all> Lending<'all, Lend = &'all i32>,
{
    fn extend_lender(&mut self, lender: L) {
        lender.into_lender().for_each(|x| self.0.push(*x));
    }

    fn extend_lender_one(&mut self, item: &i32) {
        self.0.push(*item);
    }
}

/// Helper lender that yields VecLenders
pub struct VecOfVecLender {
    data: Vec<Vec<i32>>,
    index: usize,
}

impl VecOfVecLender {
    pub fn new(data: Vec<Vec<i32>>) -> Self {
        Self { data, index: 0 }
    }
}

impl<'lend> Lending<'lend> for VecOfVecLender {
    type Lend = VecLender;
}

impl Lender for VecOfVecLender {
    check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.index < self.data.len() {
            let vec = self.data[self.index].clone();
            self.index += 1;
            Some(VecLender::new(vec))
        } else {
            None
        }
    }
}
