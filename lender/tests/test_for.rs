/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later OR MIT
 */

use lender::{from_into_iter, from_iter};
use lender_derive::for_;

#[test]
// Test with Lender and IntoLender, even if they are not imported
pub fn test_for_lender() {
    for_!(x in from_into_iter(0..10) {
        println!("{}", x);
    });

    for_!(x in from_iter(0..10) {
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
// Test that | works in patterns
pub fn test_bar() {
    for_!(x @ (Three::A | Three::B) in from_into_iter([Three::A, Three::B, Three::C]) {
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
