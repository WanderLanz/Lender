/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later OR MIT
 */

#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Block, Expr, Pat,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    token::In,
};

struct ForLenderInfo {
    pub pat: Pat,
    pub _in_token: In,
    pub expr: Expr,
    pub body: Block,
}

impl Parse for ForLenderInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ForLenderInfo {
            pat: Pat::parse_multi(input)?, // We allow for the | operator
            _in_token: input.parse()?,
            expr: Expr::parse_without_eager_brace(input)?, // As in the for loop syntax
            body: input.parse()?,
        })
    }
}

/**

Syntax sugar for iterating over a [`Lender`](https://docs.rs/lender/latest/lender/trait.Lender.html).

This function-like procedural macro expands a syntax of the form
```ignore
for_!(PATTERN in EXPR BLOCK);
```
where `PATTERN` is a valid pattern for a `for` loop, `EXPR` is an expression that
implements [`IntoLender`](https://docs.rs/lender/latest/lender/trait.IntoLender.html) and `BLOCK` is a block of code, into a `while let` loop that
iterates over a [`Lender`](https://docs.rs/lender/latest/lender/trait.Lender.html) obtained from the [`IntoLender`](https://docs.rs/lender/latest/lender/trait.IntoLender.html):
```ignore
let mut ___ඞඞඞlenderඞඞඞ___ = (EXPR).into_lender();
while let Some(PATTERN) = ___ඞඞඞlenderඞඞඞ___.next() BLOCK
```
For example, the following code
```ignore
for_!(x in 0..10 {
    println!("{}", x);
});
```
iterates over the integers [0. .10), printing them,
using a [`Lender`](https://docs.rs/lender/latest/lender/trait.Lender.html) obtained by
automagically adapting an [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) (in this case, a [`Range`](https://doc.rust-lang.org/std/ops/struct.Range.html)).

Note that the outer parentheses are part of the standard Rust syntax for function-like
procedural macros, and thus can be replaced, for example, with brackets.

For an example of a more complex usage, see the following code, which iterates over
the elements of an `enum`, but only on the first two variants:
```ignore
#[derive(Debug)]
enum Three {
    A,
    B,
    C,
}

#[test]
pub fn test_bar() {
    for_!(x @ (Three::A | Three::B) in [Three::A, Three::B, Three::C].into_into_lender() {
        dbg!(x);
    });
}
```
In this case, since an array is an [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html),
but not an [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html), we need to
adapt it manually.

Note that these examples have the sole purpose of showing the syntax of the macro:
in these cases a standard iterator would be simpler and more efficient.
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
        use ::lender::{Lender, IntoLender};
        let mut ___ඞඞඞlenderඞඞඞ___ = (#expr).into_lender();
        while let Some( #pat ) = ___ඞඞඞlenderඞඞඞ___.next() #body
    }}
    .into()
}
