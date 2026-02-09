use core::{fmt, num::NonZero};

use crate::{DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending};

/// Creates a new lender that returns mutable contiguous
/// overlapping windows of fixed size over a slice.
///
/// This is the mutable, lending variant of
/// [`windows`](https://doc.rust-lang.org/stable/std/primitive.slice.html#method.windows).
/// The const generic equivalent is
/// [`array_windows_mut`].
///
/// Note that the [`WindowsMutExt`] trait provides a convenient
/// entry point for this function as a method on slices and
/// arrays.
///
/// # Panics
///
/// Panics if `size` is zero.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut s = [0, 1, 2, 3];
/// let mut lender = lender::windows_mut(&mut s, 2);
/// assert_eq!(lender.next(), Some(&mut [0, 1][..]));
///
/// // Using the extension trait
/// let mut lender = s.windows_mut(2);
/// assert_eq!(lender.next(), Some(&mut [0, 1][..]));
/// ```
#[inline]
pub fn windows_mut<T>(slice: &mut [T], size: usize) -> WindowsMut<'_, T> {
    let size = NonZero::new(size).expect("window size must be non-zero");
    WindowsMut {
        slice,
        size,
        position: WindowPosition::Init,
    }
}

/// A lender over mutable overlapping windows of a slice.
///
/// This `struct` is created by the [`windows_mut()`] function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct WindowsMut<'a, T> {
    slice: &'a mut [T],
    size: NonZero<usize>,
    position: WindowPosition,
}

impl<T: fmt::Debug> fmt::Debug for WindowsMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WindowsMut")
            .field("slice", &self.slice)
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

/// Tracks which position was most recently returned.
#[derive(Clone, Copy)]
enum WindowPosition {
    Init,
    Front,
    Back,
}

impl WindowPosition {
    /// Drop the end of the slice that we most recently returned.
    fn update_slice<T>(self, slice: &mut &mut [T]) {
        match self {
            WindowPosition::Init => {}
            WindowPosition::Front => {
                // slice.split_off_first_mut();
                if let [_, tail @ ..] = core::mem::take(slice) {
                    *slice = tail;
                }
            }
            WindowPosition::Back => {
                // slice.split_off_last_mut();
                if let [init @ .., _] = core::mem::take(slice) {
                    *slice = init;
                }
            }
        }
    }
}

impl<'any, T> Lending<'any> for WindowsMut<'_, T> {
    type Lend = &'any mut [T];
}

impl<T> Lender for WindowsMut<'_, T> {
    crate::check_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.position.update_slice(&mut self.slice);
        self.position = WindowPosition::Front;
        self.slice.get_mut(..self.size.get())
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T> DoubleEndedLender for WindowsMut<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.position.update_slice(&mut self.slice);
        self.position = WindowPosition::Back;
        let index = self.slice.len().checked_sub(self.size.get())?;
        self.slice.get_mut(index..)
    }
}

impl<T> ExactSizeLender for WindowsMut<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        let base = self.slice.len().saturating_sub(self.size.get() - 1);
        // If position is Front or Back, a window was returned but the slice
        // hasn't been updated yet (update happens on the next call).
        // We need to subtract 1 to reflect the consumed window.
        match self.position {
            WindowPosition::Init => base,
            WindowPosition::Front | WindowPosition::Back => base.saturating_sub(1),
        }
    }
}

impl<T> FusedLender for WindowsMut<'_, T> {}

/// Creates a new lender that returns mutable overlapping array
/// windows of fixed size over a slice.
///
/// This is the mutable, lending variant of
/// [`array_windows`](https://doc.rust-lang.org/stable/std/primitive.slice.html#method.windows).
/// The non-const generic equivalent is
/// [`windows_mut`].
///
/// Note that the [`WindowsMutExt`] trait provides a convenient
/// entry point for this function as a method on slices and
/// arrays.
///
/// See
/// [`array_windows`](https://doc.rust-lang.org/stable/std/primitive.slice.html#method.array_windows)
/// for more information.
///
/// # Panics
///
/// Panics if `WINDOW_SIZE` is zero.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut s = [0, 1, 2, 3];
/// let mut lender = lender::array_windows_mut::<_, 2>(&mut s);
/// assert_eq!(lender.next(), Some(&mut [0, 1]));
///
/// // Using the extension trait
/// let mut lender = s.array_windows_mut::<2>();
/// assert_eq!(lender.next(), Some(&mut [0, 1]));
/// ```
#[inline]
pub fn array_windows_mut<T, const WINDOW_SIZE: usize>(
    slice: &mut [T],
) -> ArrayWindowsMut<'_, T, WINDOW_SIZE> {
    assert!(WINDOW_SIZE != 0, "window size must be non-zero");
    ArrayWindowsMut {
        slice,
        position: WindowPosition::Init,
    }
}

/// A lender over mutable overlapping windows of a slice
/// as fixed-size arrays.
///
/// This `struct` is created by the [`array_windows_mut()`]
/// function.
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct ArrayWindowsMut<'a, T, const WINDOW_SIZE: usize> {
    slice: &'a mut [T],
    position: WindowPosition,
}

impl<T: fmt::Debug, const WINDOW_SIZE: usize> fmt::Debug for ArrayWindowsMut<'_, T, WINDOW_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArrayWindowsMut")
            .field("slice", &self.slice)
            .finish_non_exhaustive()
    }
}

impl<'any, T, const WINDOW_SIZE: usize> Lending<'any> for ArrayWindowsMut<'_, T, WINDOW_SIZE> {
    type Lend = &'any mut [T; WINDOW_SIZE];
}

impl<T, const WINDOW_SIZE: usize> Lender for ArrayWindowsMut<'_, T, WINDOW_SIZE> {
    crate::check_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.position.update_slice(&mut self.slice);
        self.position = WindowPosition::Front;
        self.slice.first_chunk_mut()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T, const WINDOW_SIZE: usize> DoubleEndedLender for ArrayWindowsMut<'_, T, WINDOW_SIZE> {
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.position.update_slice(&mut self.slice);
        self.position = WindowPosition::Back;
        self.slice.last_chunk_mut()
    }
}

impl<T, const WINDOW_SIZE: usize> ExactSizeLender for ArrayWindowsMut<'_, T, WINDOW_SIZE> {
    #[inline]
    fn len(&self) -> usize {
        let base = self.slice.len().saturating_sub(WINDOW_SIZE - 1);
        // If position is Front or Back, a window was returned but the slice
        // hasn't been updated yet (update happens on the next call).
        // We need to subtract 1 to reflect the consumed window.
        match self.position {
            WindowPosition::Init => base,
            WindowPosition::Front | WindowPosition::Back => base.saturating_sub(1),
        }
    }
}

impl<T, const WINDOW_SIZE: usize> FusedLender for ArrayWindowsMut<'_, T, WINDOW_SIZE> {}

/// Extension trait adding to slices and arrays the methods
/// [`windows_mut`](WindowsMutExt::windows_mut) and
/// [`array_windows_mut`](WindowsMutExt::array_windows_mut).
pub trait WindowsMutExt<T> {
    /// Returns a lender over mutable contiguous overlapping windows of `size` elements.
    ///
    /// See [`windows_mut`] for more details.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    fn windows_mut(&mut self, size: usize) -> WindowsMut<'_, T>;
    /// Returns a lender over mutable overlapping array
    /// windows of `WINDOW_SIZE` elements.
    ///
    /// See [`array_windows_mut`] for more details.
    ///
    /// # Panics
    ///
    /// Panics if `WINDOW_SIZE` is zero.
    fn array_windows_mut<const WINDOW_SIZE: usize>(
        &mut self,
    ) -> ArrayWindowsMut<'_, T, WINDOW_SIZE>;
}

impl<T> WindowsMutExt<T> for [T] {
    /// This method is a convenient entry point for [`windows_mut`].
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    #[inline]
    fn windows_mut(&mut self, size: usize) -> WindowsMut<'_, T> {
        windows_mut(self, size)
    }
    /// This method is a convenient entry point for [`array_windows_mut`].
    ///
    /// # Panics
    ///
    /// Panics if `WINDOW_SIZE` is zero.
    #[inline]
    fn array_windows_mut<const WINDOW_SIZE: usize>(
        &mut self,
    ) -> ArrayWindowsMut<'_, T, WINDOW_SIZE> {
        array_windows_mut(self)
    }
}

impl<T, const N: usize> WindowsMutExt<T> for [T; N] {
    /// This method is a convenient entry point for [`windows_mut`].
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    #[inline]
    fn windows_mut(&mut self, size: usize) -> WindowsMut<'_, T> {
        windows_mut(self, size)
    }
    /// This method is a convenient entry point for [`array_windows_mut`].
    ///
    /// # Panics
    ///
    /// Panics if `WINDOW_SIZE` is zero.
    #[inline]
    fn array_windows_mut<const WINDOW_SIZE: usize>(
        &mut self,
    ) -> ArrayWindowsMut<'_, T, WINDOW_SIZE> {
        array_windows_mut(self)
    }
}

#[test]
fn test_array_windows_mut() {
    let mut s = [0, 1, 2, 3];
    let mut lender = array_windows_mut::<_, 2>(&mut s);
    assert_eq!(lender.next(), Some(&mut [0, 1]));
    assert_eq!(lender.next(), Some(&mut [1, 2]));
    assert_eq!(lender.next(), Some(&mut [2, 3]));
    assert_eq!(lender.next(), None);
}

#[test]
fn test_array_windows_mut_rev() {
    let mut s = [0, 1, 2, 3];
    let mut lender = array_windows_mut::<_, 2>(&mut s).rev();
    assert_eq!(lender.next(), Some(&mut [2, 3]));
    assert_eq!(lender.next(), Some(&mut [1, 2]));
    assert_eq!(lender.next(), Some(&mut [0, 1]));
    assert_eq!(lender.next(), None);
}

#[test]
fn test_array_windows_mut_mixed() {
    let mut s = [0, 1, 2, 3, 4];
    let mut lender = array_windows_mut::<_, 2>(&mut s);
    assert_eq!(lender.next_back(), Some(&mut [3, 4]));
    assert_eq!(lender.next(), Some(&mut [0, 1]));
    assert_eq!(lender.next_back(), Some(&mut [2, 3]));
    assert_eq!(lender.next(), Some(&mut [1, 2]));
    assert_eq!(lender.next_back(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn test_windows_mut() {
    let mut s = [0, 1, 2, 3];
    let mut lender = windows_mut(&mut s, 2);
    assert_eq!(lender.next(), Some(&mut [0, 1][..]));
    assert_eq!(lender.next(), Some(&mut [1, 2][..]));
    assert_eq!(lender.next(), Some(&mut [2, 3][..]));
    assert_eq!(lender.next(), None);
}

#[test]
fn test_windows_mut_rev() {
    let mut s = [0, 1, 2, 3];
    let mut lender = windows_mut(&mut s, 2).rev();
    assert_eq!(lender.next(), Some(&mut [2, 3][..]));
    assert_eq!(lender.next(), Some(&mut [1, 2][..]));
    assert_eq!(lender.next(), Some(&mut [0, 1][..]));
    assert_eq!(lender.next(), None);
}

#[test]
fn test_windows_mut_back() {
    let mut s = [0, 1, 2, 3, 4];
    let mut lender = windows_mut(&mut s, 2);
    assert_eq!(lender.next_back(), Some(&mut [3, 4][..]));
    assert_eq!(lender.next(), Some(&mut [0, 1][..]));
    assert_eq!(lender.next_back(), Some(&mut [2, 3][..]));
    assert_eq!(lender.next(), Some(&mut [1, 2][..]));
    assert_eq!(lender.next_back(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn test_windows_mut_exact_size() {
    use crate::ExactSizeLender;

    let mut s = [0, 1, 2, 3, 4];
    let mut lender = windows_mut(&mut s, 2);
    assert_eq!(lender.len(), 4);
    lender.next();
    assert_eq!(lender.len(), 3);
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next();
    assert_eq!(lender.len(), 1);
    lender.next();
    assert_eq!(lender.len(), 0);
    lender.next(); // returns None
    assert_eq!(lender.len(), 0);

    // Edge cases
    let mut empty: [i32; 0] = [];
    assert_eq!(windows_mut(&mut empty, 2).len(), 0);

    let mut single = [1];
    assert_eq!(windows_mut(&mut single, 2).len(), 0);
    assert_eq!(windows_mut(&mut single, 1).len(), 1);
}

#[test]
#[should_panic(expected = "window size must be non-zero")]
fn test_windows_mut_zero_size_panics() {
    let mut arr = [1, 2, 3];
    let _ = windows_mut(&mut arr, 0);
}

#[test]
fn test_array_windows_mut_exact_size() {
    use crate::ExactSizeLender;

    let mut s = [0, 1, 2, 3, 4];
    let mut lender = array_windows_mut::<_, 2>(&mut s);
    assert_eq!(lender.len(), 4);
    lender.next();
    assert_eq!(lender.len(), 3);
    lender.next();
    assert_eq!(lender.len(), 2);

    // Edge cases
    let mut single = [1];
    assert_eq!(array_windows_mut::<_, 2>(&mut single).len(), 0);
    assert_eq!(array_windows_mut::<_, 1>(&mut single).len(), 1);
}

#[test]
fn test_windows_mut_exact_size_mixed() {
    use crate::ExactSizeLender;

    let mut s = [0, 1, 2, 3, 4];
    let mut lender = windows_mut(&mut s, 2);
    assert_eq!(lender.len(), 4);
    lender.next_back();
    assert_eq!(lender.len(), 3);
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next_back();
    assert_eq!(lender.len(), 1);
    lender.next();
    assert_eq!(lender.len(), 0);
}
