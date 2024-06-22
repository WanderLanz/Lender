use core::marker::PhantomData;

use crate::{prelude::*, FusedLender};

/// Creates a new iterator that repeats elements of type `A` endlessly by
/// applying the provided closure, the repeater, `F: FnMut() -> A`.
///
/// The `repeat_with()` function calls the repeater over and over again.
///
/// See [`iter::repeat_with()`](core::iter::repeat_with) for more information.
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::repeat_with::<lend!(&'lend u8), _>(|| &0u8);
/// assert_eq!(lender.next(), Some(&0));
/// ```
pub fn repeat_with<'a, L, F>(f: F) -> RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    RepeatWith { f, _marker: PhantomData }
}

/// A lender that repeats an element endlessly by applying a closure.
///
/// This `struct` is created by the [`repeat_with()`] function.
pub struct RepeatWith<'a, L: ?Sized, F> {
    f: F,
    _marker: core::marker::PhantomData<&'a L>,
}

impl<'lend, 'a, L, F> Lending<'lend> for RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L, F> Lender for RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend
        Some(unsafe { core::mem::transmute::<Lend<'a, L>, Lend<'_, L>>((self.f)()) })
    }
    #[inline]
    fn advance_by(&mut self, _n: usize) -> Result<(), core::num::NonZeroUsize> { Ok(()) }
}

impl<'a, L, F> DoubleEndedLender for RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> { self.next() }
    #[inline]
    fn advance_back_by(&mut self, _n: usize) -> Result<(), core::num::NonZeroUsize> { Ok(()) }
}

impl<'a, L, F> FusedLender for RepeatWith<'a, L, F>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    F: FnMut() -> Lend<'a, L>,
{
}
