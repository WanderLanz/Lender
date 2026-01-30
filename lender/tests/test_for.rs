use lender::{from_into_iter, from_iter, prelude::*};

#[test]
// Test with Lender and IntoLender. Note that iterators are converted automagically.
fn test_for_lender() {
    let mut sum = 0;
    for_!(x in from_into_iter(0..10) {
        sum += x;
    });
    assert_eq!(sum, 45);

    let mut sum = 0;
    for_!(x in from_iter(0..10) {
        sum += x;
    });
    assert_eq!(sum, 45);

    let mut sum = 0;
    for_!(x in 0..10 {
        sum += x;
    });
    assert_eq!(sum, 45);
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
fn test_bar() {
    let mut count = 0;
    for_!(_x @ (Three::A | Three::B) in [Three::A, Three::B, Three::C].into_into_lender() {
        count += 1;
    });
    assert_eq!(count, 2);
}

#[test]
// Test that we parse without eager brace
// https://docs.rs/syn/latest/syn/enum.Expr.html#method.parse_without_eager_brace
fn test_brace() {
    let lender = from_iter(0..10);
    let mut sum = 0;
    for_!(x in lender {
        sum += x;
    });
    assert_eq!(sum, 45);
}
