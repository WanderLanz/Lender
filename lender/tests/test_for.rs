/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later OR MIT
 */

use lender::{from_into_iter, from_iter, IntoLender, Lender};
use lender_derive::for_;

#[test]
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
pub fn test_bar() {
    for_!(x @ (Three::A | Three::B) in from_into_iter([Three::A, Three::B, Three::C]) {
        dbg!(x);
    });
}
