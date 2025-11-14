use alloc::boxed::Box;
use core::{fmt, ops::ControlFlow};

use crate::{
    try_trait_v2::{FromResidual, Try},
    DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending,
};

#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Peekable<'this, L>
where
    L: Lender,
{
    lender: Box<L>,
    peeked: Option<Option<Lend<'this, L>>>,
}
impl<'this, L> Peekable<'this, L>
where
    L: Lender,
{
    pub(crate) fn new(lender: L) -> Peekable<'this, L> {
        Peekable { lender: Box::new(lender), peeked: None }
    }
    pub fn into_inner(self) -> L {
        *self.lender
    }
    pub fn peek(&mut self) -> Option<&'_ Lend<'this, L>> {
        let lender = &mut self.lender;
        self.peeked
            .get_or_insert_with(|| {
                // SAFETY: The lend is manually guaranteed to be the only one alive
                unsafe { core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(lender.next()) }
            })
            .as_ref()
    }
    pub fn peek_mut(&mut self) -> Option<&'_ mut Lend<'this, L>> {
        let lender = &mut self.lender;
        self.peeked
            .get_or_insert_with(|| {
                // SAFETY: The lend is manually guaranteed to be the only one alive
                unsafe { core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(lender.next()) }
            })
            .as_mut()
    }
    pub fn next_if<F>(&mut self, f: F) -> Option<Lend<'_, L>>
    where
        F: FnOnce(&Lend<'_, L>) -> bool,
    {
        let peeked = unsafe { &mut *(&mut self.peeked as *mut _) };
        match self.next() {
            Some(v) if f(&v) => Some(v),
            v => {
                // SAFETY: The lend is manually guaranteed to be the only one alive
                *peeked = Some(unsafe { core::mem::transmute::<Option<Lend<'_, L>>, Option<Lend<'this, L>>>(v) });
                None
            }
        }
    }
    pub fn next_if_eq<'a, T>(&'a mut self, t: &T) -> Option<Lend<'a, L>>
    where
        T: for<'all> PartialEq<Lend<'all, L>>,
    {
        self.next_if(|v| t == v)
    }
}
impl<L> Clone for Peekable<'_, L>
where
    L: Lender + Clone,
{
    fn clone(&self) -> Self {
        Peekable { lender: self.lender.clone(), peeked: None }
    }
}
impl<'this, L: fmt::Debug> fmt::Debug for Peekable<'this, L>
where
    L: Lender + fmt::Debug,
    Lend<'this, L>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Peekable").field("lender", &self.lender).field("peeked", &self.peeked).finish()
    }
}
impl<'lend, L> Lending<'lend> for Peekable<'_, L>
where
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}
impl<'this, L> Lender for Peekable<'this, L>
where
    L: Lender,
{
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        match self.peeked.take() {
            // SAFETY: The lend is manually guaranteed to be the only one alive
            Some(peeked) => unsafe { core::mem::transmute::<Option<Lend<'this, Self>>, Option<Lend<'_, Self>>>(peeked) },
            None => self.lender.next(),
        }
    }
    #[inline]
    fn count(mut self) -> usize {
        match self.peeked.take() {
            Some(None) => 0,
            Some(Some(_)) => 1 + self.lender.count(),
            None => self.lender.count(),
        }
    }
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        match self.peeked.take() {
            Some(None) => None,
            // SAFETY: The lend is manually guaranteed to be the only one alive
            Some(v @ Some(_)) if n == 0 => unsafe {
                core::mem::transmute::<Option<Lend<'this, Self>>, Option<Lend<'_, Self>>>(v)
            },
            Some(Some(_)) => self.lender.nth(n - 1),
            None => self.lender.nth(n),
        }
    }
    #[inline]
    fn last<'a>(&'a mut self) -> Option<Lend<'a, Self>>
    where
        Self: Sized,
    {
        let peek_opt = match self.peeked.take() {
            Some(None) => return None,
            // SAFETY: 'this: 'call
            Some(v) => unsafe { core::mem::transmute::<Option<Lend<'this, Self>>, Option<Lend<'a, Self>>>(v) },
            None => None,
        };
        self.lender.last().or(peek_opt)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let peek_len = match self.peeked {
            Some(None) => return (0, Some(0)),
            Some(Some(_)) => 1,
            None => 0,
        };
        let (l, r) = self.lender.size_hint();
        (l.saturating_add(peek_len), r.and_then(|r| r.checked_add(peek_len)))
    }
    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let acc = match self.peeked.take() {
            Some(None) => return Try::from_output(init),
            Some(Some(v)) => match f(init, v).branch() {
                ControlFlow::Break(b) => return FromResidual::from_residual(b),
                ControlFlow::Continue(a) => a,
            },
            None => init,
        };
        self.lender.try_fold(acc, f)
    }
    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let acc = match self.peeked.take() {
            Some(None) => return init,
            Some(Some(v)) => f(init, v),
            None => init,
        };
        self.lender.fold(acc, f)
    }
}
impl<'this, L: DoubleEndedLender> DoubleEndedLender for Peekable<'this, L> {
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        match self.peeked.as_mut() {
            // SAFETY: The lend is manually guaranteed to be the only one alive
            Some(v @ Some(_)) => self
                .lender
                .next_back()
                .or_else(|| unsafe { core::mem::transmute::<Option<Lend<'this, Self>>, Option<Lend<'_, Self>>>(v.take()) }),
            Some(None) => None,
            None => self.lender.next_back(),
        }
    }
    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        match self.peeked.take() {
            None => self.lender.try_rfold(init, f),
            Some(None) => Try::from_output(init),
            Some(Some(v)) => match self.lender.try_rfold(init, &mut f).branch() {
                ControlFlow::Continue(acc) => f(acc, v),
                ControlFlow::Break(r) => {
                    self.peeked = Some(Some(v));
                    FromResidual::from_residual(r)
                }
            },
        }
    }
    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        match self.peeked.take() {
            None => self.lender.rfold(init, f),
            Some(None) => init,
            Some(Some(v)) => {
                let acc = self.lender.rfold(init, &mut f);
                f(acc, v)
            }
        }
    }
}
impl<L: ExactSizeLender> ExactSizeLender for Peekable<'_, L> {}

impl<L: FusedLender> FusedLender for Peekable<'_, L> {}

#[cfg(test)]
mod test {
    use super::*;

    struct ArrayLender {
        array: [i32; 4],
    }

    impl<'lend> Lending<'lend> for ArrayLender {
        type Lend = &'lend i32;
    }

    impl<'lend> Lender for ArrayLender {
        fn next(&mut self) -> Option<Lend<'_, Self>> {
            Some(&self.array[0])
        }
    }

    // This test will fail if Peekable stores L instead of Box<L>. In that case,
    // when Peekable<ArrayLender> is moved, the array inside ArrayLender is
    // moved, too, but Peekable.peeked will still contain a reference to the
    // previous location.
    #[test]
    fn test_peekable() {
        let lender = ArrayLender { array: [-1, 1, 2, 3] };
        let mut peekable = lender.peekable();
        assert_eq!(**peekable.peek().unwrap(), -1);
        assert_eq!(peekable.peeked.unwrap().unwrap() as *const _, &peekable.lender.array[0] as *const _);
        moved_peekable(peekable);
    }

    fn moved_peekable(peekable: Peekable<ArrayLender>) {
        let peeked = peekable.peeked.unwrap().unwrap() as *const _;
        let array = &peekable.lender.array[0] as *const _;
        assert_eq!(peeked, array, "Peeked element pointer should point to the first element of the array");
    }
}
