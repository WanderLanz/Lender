use crate::{Lend, Lender, Lending};

/// Create a new lender that returns mutable contiguous overlapping windows of fixed size over a slice.
///
/// This is the mutable, lending variant of [`windows`](https://doc.rust-lang.org/stable/std/primitive.slice.html#method.windows).
/// The const generic equivalent is [`array_windows_mut`].
///
/// Note that the [`WindowsMutExt`] trait provides a convenient entry point for this function
/// as a method on slices and arrays.
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
pub fn windows_mut<'a, T>(slice: &'a mut [T], size: usize) -> WindowsMut<'a, T> {
    WindowsMut { slice, size, curr_pos: 0 }
}

/// This struct is returned by [`windows_mut`].
pub struct WindowsMut<'a, T> {
    pub(crate) slice: &'a mut [T],
    pub(crate) size: usize,
    pub(crate) curr_pos: usize,
}

impl<'a, 'any, T> Lending<'any> for WindowsMut<'a, T> {
    type Lend = &'any mut [T];
}

impl<'a, T> Lender for WindowsMut<'a, T> {
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // See https://github.com/danielhenrymantilla/lending-iterator.rs/blob/5353b5e6ce8be9d07d0cfd86e23e481377074780/src/lending_iterator/constructors/windows_mut_.rs
        let window = self.slice.get_mut(self.curr_pos..)?.get_mut(..self.size)?;
        self.curr_pos += 1;
        Some(window)
    }
}

/// Create a new lender that returns mutable overlapping array windows of fixed size over a slice.
///
/// This is the mutable, lending variant of [`array_windows`](https://doc.rust-lang.org/stable/std/primitive.slice.html#method.windows).
/// The non-const generic equivalent is [`windows_mut`].
///
/// Note that the [`WindowsMutExt`] trait provides a convenient entry point for this function
/// as a method on slices and arrays.
///
/// See [`array_windows`](https://doc.rust-lang.org/stable/std/primitive.slice.html#method.array_windows) for more information.
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
pub fn array_windows_mut<'a, T, const WINDOW_SIZE: usize>(slice: &'a mut [T]) -> ArrayWindowsMut<'a, T, WINDOW_SIZE> {
    ArrayWindowsMut { slice, curr_pos: 0 }
}

/// This struct is returned by [`array_windows_mut`].
pub struct ArrayWindowsMut<'a, T, const WINDOW_SIZE: usize> {
    pub(crate) slice: &'a mut [T],
    pub(crate) curr_pos: usize,
}

impl<'a, 'any, T, const WINDOW_SIZE: usize> Lending<'any> for ArrayWindowsMut<'a, T, WINDOW_SIZE> {
    type Lend = &'any mut [T; WINDOW_SIZE];
}

impl<'a, T, const WINDOW_SIZE: usize> Lender for ArrayWindowsMut<'a, T, WINDOW_SIZE> {
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // See https://github.com/danielhenrymantilla/lending-iterator.rs/blob/5353b5e6ce8be9d07d0cfd86e23e481377074780/src/lending_iterator/constructors/windows_mut_.rs
        let window = self.slice.get_mut(self.curr_pos..)?.get_mut(..WINDOW_SIZE)?;
        self.curr_pos += 1;
        Some(window.try_into().unwrap())
    }
}

/// Extension trait adding to slice and arrays the methods [`windows_mut`](WindowsMutExt::windows_mut)
/// and [`array_windows_mut`](WindowsMutExt::array_windows_mut).
pub trait WindowsMutExt<T> {
    fn windows_mut(&mut self, size: usize) -> WindowsMut<'_, T>;
    fn array_windows_mut<const WINDOW_SIZE: usize>(&mut self) -> ArrayWindowsMut<'_, T, WINDOW_SIZE>;
}

impl<T> WindowsMutExt<T> for [T] {
    /// This method is a convenient entry point for [`windows_mut`](crate::windows_mut).
    fn windows_mut(&mut self, size: usize) -> WindowsMut<'_, T> {
        windows_mut(self, size)
    }
    /// This method is a convenient entry point for [`array_windows_mut`](crate::array_windows_mut).
    fn array_windows_mut<const WINDOW_SIZE: usize>(&mut self) -> ArrayWindowsMut<'_, T, WINDOW_SIZE> {
        array_windows_mut(self)
    }
}

impl<T, const N: usize> WindowsMutExt<T> for [T; N] {
    /// This method is a convenient entry point for [`windows_mut`](crate::windows_mut).
    fn windows_mut(&mut self, size: usize) -> WindowsMut<'_, T> {
        windows_mut(self, size)
    }
    /// This method is a convenient entry point for [`array_windows_mut`](crate::array_windows_mut).
    fn array_windows_mut<const WINDOW_SIZE: usize>(&mut self) -> ArrayWindowsMut<'_, T, WINDOW_SIZE> {
        array_windows_mut(self)
    }
}

#[cfg(test)]
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
fn test_windows_mut() {
    let mut s = [0, 1, 2, 3];
    let mut lender = windows_mut(&mut s, 2);
    assert_eq!(lender.next(), Some(&mut [0, 1][..]));
    assert_eq!(lender.next(), Some(&mut [1, 2][..]));
    assert_eq!(lender.next(), Some(&mut [2, 3][..]));
    assert_eq!(lender.next(), None);
}
