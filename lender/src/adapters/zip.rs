use crate::{
    DoubleEndedLender, ExactSizeLender, FusedLender, IntoLender, Lend, Lender, Lending,
};

/// Zips two lenders into a single lender of pairs.
///
/// This is the lender equivalent of [`Iterator::zip`].
///
/// # Examples
///
/// ```
/// use lender::prelude::*;
///
/// let a = lender::lend_iter::<lend!(&'lend i32), _>([1, 2, 3].iter());
/// let b = lender::lend_iter::<lend!(&'lend i32), _>([4, 5, 6].iter());
///
/// let mut zipped = lender::zip(a, b);
///
/// assert_eq!(zipped.next(), Some((&1, &4)));
/// assert_eq!(zipped.next(), Some((&2, &5)));
/// assert_eq!(zipped.next(), Some((&3, &6)));
/// assert_eq!(zipped.next(), None);
/// ```
pub fn zip<A, B>(a: A, b: B) -> Zip<A::Lender, B::Lender>
where
    A: IntoLender,
    B: IntoLender,
{
    Zip::new(a.into_lender(), b.into_lender())
}

/// A lender that yields pairs of elements from two underlying lenders.
///
/// This `struct` is created by [`Lender::zip`] or [`zip`]. See their
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Zip<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A, B> Zip<A, B> {
    #[inline(always)]
    pub(crate) fn new(a: A, b: B) -> Self {
        Self { a, b }
    }

    /// Returns the inner lenders.
    ///
    /// # Examples
    ///
    /// ```
    /// use lender::prelude::*;
    ///
    /// let a = lender::lend_iter::<lend!(&'lend i32), _>([1, 2, 3].iter());
    /// let b = lender::lend_iter::<lend!(&'lend i32), _>([4, 5, 6].iter());
    ///
    /// let mut zipped = lender::zip(a, b);
    ///
    /// assert_eq!(zipped.next(), Some((&1, &4)));
    ///
    /// let (mut a, mut b) = zipped.into_inner();
    /// assert_eq!(a.next(), Some(&2));
    /// assert_eq!(b.next(), Some(&5));
    /// ```
    #[inline(always)]
    pub fn into_inner(self) -> (A, B) {
        (self.a, self.b)
    }
}

impl<'lend, A, B> Lending<'lend> for Zip<A, B>
where
    A: Lender,
    B: Lender,
{
    type Lend = (Lend<'lend, A>, Lend<'lend, B>);
}

impl<A, B> Lender for Zip<A, B>
where
    A: Lender,
    B: Lender,
{
    // SAFETY: the lend is a tuple of the lends of A and B
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        Some((self.a.next()?, self.b.next()?))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        let a = self.a.nth(n)?;
        let b = self.b.nth(n)?;
        Some((a, b))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();

        let lower = core::cmp::min(a_lower, b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => Some(core::cmp::min(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        };

        (lower, upper)
    }
}

impl<A, B> DoubleEndedLender for Zip<A, B>
where
    A: DoubleEndedLender + ExactSizeLender,
    B: DoubleEndedLender + ExactSizeLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        let a_sz = self.a.len();
        let b_sz = self.b.len();
        if a_sz > b_sz {
            self.a.advance_back_by(a_sz - b_sz).ok();
        } else if b_sz > a_sz {
            self.b.advance_back_by(b_sz - a_sz).ok();
        }
        match (self.a.next_back(), self.b.next_back()) {
            (Some(x), Some(y)) => Some((x, y)),
            (None, None) => None,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        let a_sz = self.a.len();
        let b_sz = self.b.len();
        if a_sz > b_sz {
            self.a.advance_back_by(a_sz - b_sz).ok();
        } else if b_sz > a_sz {
            self.b.advance_back_by(b_sz - a_sz).ok();
        }
        match (self.a.nth_back(n), self.b.nth_back(n)) {
            (Some(x), Some(y)) => Some((x, y)),
            (None, None) => None,
            _ => unreachable!(),
        }
    }
}

impl<A, B> ExactSizeLender for Zip<A, B>
where
    A: ExactSizeLender,
    B: ExactSizeLender,
{
}

impl<A, B> FusedLender for Zip<A, B>
where
    A: FusedLender,
    B: FusedLender,
{
}
