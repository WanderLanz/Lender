// Test that the covariance check catches types that LOOK covariant but aren't.
//
// MightLookCovariant<'a> = &'a Invariant<'a> appears covariant because the outer
// reference is covariant, but Invariant<'a> is invariant due to &'a mut &'a str.
// The Deref implementation to Covariant<'a> doesn't help - Rust correctly
// identifies the invariance.

use std::ops::Deref;

use lender::{Lend, Lender, Lending};

struct Invariant<'a>(&'a mut &'a str);
struct Covariant<'a>(&'a str);

impl<'a> Deref for Invariant<'a> {
    type Target = Covariant<'a>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0 as *const &str as *const Covariant<'a>) }
    }
}

type MightLookCovariant<'a> = &'a Invariant<'a>;

struct TestLender;

impl<'lend> Lending<'lend> for TestLender {
    type Lend = MightLookCovariant<'lend>;
}

impl Lender for TestLender {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
