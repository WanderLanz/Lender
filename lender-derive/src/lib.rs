/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later OR MIT
 */

/*!

Derive procedural macros for the [`lender`](https://crates.io/crates/Lender) crate.

This crate provides a [for_!] function-like macro that can be used to iterate over
a [`IntoLender`](https://docs.rs/lender/latest/lender/trait.IntoLender.html) with a
syntax similar to a `for` loop:
```[ignore]
for_!(x in into_lender {
    ...
});
```
The macro expands to a `while let` loop that iterates over a
[`Lender`](https://docs.rs/lender/latest/lender/trait.Lender.html) obtained from the
[`IntoLender`](https://docs.rs/lender/latest/lender/trait.IntoLender.html).
The full `for` syntax is suppored (patterns, etc.).
*/

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    token::In,
    Block, Expr, Pat,
};

struct ForLenderInfo {
    pub pat: Box<Pat>,
    pub _in_token: In,
    pub expr: Box<Expr>,
    pub body: Block,
}

impl Parse for ForLenderInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ForLenderInfo {
            pat: Box::new(Pat::parse_multi(input)?),
            _in_token: input.parse()?,
            expr: input.parse()?,
            body: input.parse()?,
        })
    }
}

/**

Syntax sugar for iterating over a `Lender`.

This function-like macro expands a syntax of the form
```[ignore]
for_!(PATTERN in EXPR BLOCK);
```
where `PATTERN` is a valid pattern for a `for` loop, `EXPR` is an expression that
implements [`IntoLender`](https://docs.rs/lender/latest/lender/trait.IntoLender.html) and `BLOCK` is a block of code, into a `while let` loop that
iterates over a `Lender` obtained from the [`IntoLender`](https://docs.rs/lender/latest/lender/trait.IntoLender.html):
```[ignore]
let mut ___ඞඞඞlenderඞඞඞ___ = (EXPR).into_lender();
while let Some(PATTERN) = ___ඞඞඞlenderඞඞඞ___.next() BLOCK
```
For example, the following code
```[ignore]
for_!(x in from_into_iter(0..10) {
    println!("{}", x);
});
```
iterates over the integers [0. .10), printing them,
using a [`Lender`](https://docs.rs/lender/latest/lender/trait.Lender.html) obtained by
adapting an `IntoIterator` (in this case, a `Range`).

For an example of a more complex usage, see the following code, which iterates over
the elements of an `enum`, but only on the first two variants:
```[ignore]
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
```

*/
#[proc_macro]
pub fn for_(input: TokenStream) -> TokenStream {
    let ForLenderInfo {
        pat,
        _in_token,
        expr,
        body,
    } = parse_macro_input!(input as ForLenderInfo);

    quote! {{
        let mut ___ඞඞඞlenderඞඞඞ___ = (#expr).into_lender();
        while let Some( #pat ) = ___ඞඞඞlenderඞඞඞ___.next() #body
    }}
    .into()
}