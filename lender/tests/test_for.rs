use lender::prelude::*;

#[test]
// Test with Lender and IntoLender. Note that iterators are converted automagically.
pub fn test_for_lender() {
    for_!(x in from_into_iter(0..10) {
        println!("{}", x);
    });

    for_!(x in from_iter(0..10) {
        println!("{}", x);
    });

    for_!(x in 0..10 {
        println!("{}", x);
    });
}

#[derive(Debug)]
enum Three {
    A,
    B,
    C,
}

#[test]
// Test that | works in patterns. Note that an array is an IntoIterator, but not
// an Iterator, so we need to convert it manually.
pub fn test_bar() {
    for_!(x @ (Three::A | Three::B) in [Three::A, Three::B, Three::C].into_into_lender() {
        dbg!(x);
    });
}

#[test]
// Test that we parse without eager brace
// https://docs.rs/syn/latest/syn/enum.Expr.html#method.parse_without_eager_brace
pub fn test_brace() {
    let lender = from_iter(0..10);
    for_!(x in lender {
        println!("{}", x);
    });
}
