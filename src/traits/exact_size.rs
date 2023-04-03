use crate::*;

pub trait ExactSizeLender: Lender {
    #[inline]
    fn len(&self) -> usize {
        let (lower, upper) = self.size_hint();
        assert_eq!(upper, Some(lower));
        lower
    }
    #[inline]
    fn is_empty(&self) -> bool { self.len() == 0 }
}
impl<I: ExactSizeLender> ExactSizeLender for &mut I {
    fn len(&self) -> usize { (**self).len() }
    fn is_empty(&self) -> bool { (**self).is_empty() }
}
