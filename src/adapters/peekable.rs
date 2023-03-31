use crate::{Lender, Lending};
pub struct Peekable<'this, L>
where
    L: Lender + 'this,
{
    lender: L,
    peeked: Option<Option<<L as Lending<'this>>::Lend>>,
}
impl<'this, L> Peekable<'this, L>
where
    L: Lender + 'this,
{
    pub(crate) fn new(lender: L) -> Peekable<'this, L> { Peekable { lender, peeked: None } }
    pub fn peek<'peek>(&'peek mut self) -> Option<&'peek <L as Lending<'peek>>::Lend> {
        let lender = &mut self.lender;
        //@ REVIEW: Is this safe? I'm not sure if maybe there's a way to make this UB
        // SAFETY: The item only lives until the next call to next?
        unsafe {
            core::mem::transmute::<Option<&<L as Lending<'this>>::Lend>, Option<&<L as Lending<'peek>>::Lend>>(
                self.peeked
                    .get_or_insert_with(|| {
                        //@ REVIEW: Is this safe? I'm not sure if maybe there's a way to make this UB
                        // SAFETY: The item only lives until the next call to next?
                        core::mem::transmute::<Option<<L as Lending<'peek>>::Lend>, Option<<L as Lending<'this>>::Lend>>(
                            lender.next(),
                        )
                    })
                    .as_ref(),
            )
        }
    }
}
impl<'lend, 'this, L> Lending<'lend> for Peekable<'this, L>
where
    L: Lender + 'this,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<'this, L> Lender for Peekable<'this, L>
where
    L: Lender + 'this,
{
    fn next<'next>(&'next mut self) -> Option<<Self as Lending<'next>>::Lend> {
        match Option::take(&mut self.peeked) {
            //@ REVIEW: Is this safe? I'm not sure if maybe there's a way to make this UB
            // SAFETY: The item only lives until the next call to next?
            Some(peeked) => unsafe {
                core::mem::transmute::<Option<<Self as Lending<'this>>::Lend>, Option<<Self as Lending<'next>>::Lend>>(
                    peeked,
                )
            },
            None => self.lender.next(),
        }
    }
}
