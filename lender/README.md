# `lender` 🙂

[![downloads](https://img.shields.io/crates/d/lender)](https://crates.io/crates/lender)
[![dependents](https://img.shields.io/librariesio/dependents/cargo/lender)](https://crates.io/crates/lender/reverse_dependencies)
[![miri](https://img.shields.io/github/actions/workflow/status/WanderLanz/Lender/test.yml?label=miri)](https://github.com/WanderLanz/Lender/actions/workflows/test.yml)
![license](https://img.shields.io/crates/l/lender)

A _lender_, also called a _lending iterator_, is an iterator that lends mutable borrows
to the items it returns. In particular, this means that the reference to an item is
invalidated by the subsequent call to `next`. Niko Matsakis has an interesting
[blog post](https://smallcultfollowing.com/babysteps/blog/2023/05/09/giving-lending-and-async-closures/)
explaining a general view of giving vs. lending traits.

The typical example that cannot be written with standard Rust iterators,
but is covered by lenders, is that of a lender returning
[mutable, overlapping windows of a slice or array](https://docs.rs/lender/latest/lender/trait.WindowsMutExt.html).

But lenders are more general than that, as they might return items that depend on
some mutable state stored in the iterator. For example, a lender might
return references to the lines of a file reusing an internal buffer; also,
starting from an iterator on pairs of integers lexicographically sorted, a lender
might return iterators on pairs with the same first coordinate without any copying;
clearly, in all these cases any call on `next` would invalidate the reference
returned by the previous call.

This crate provides a lender trait and an associated library of utility methods,
“utilizing” [#84533](https://github.com/rust-lang/rust/issues/84533) and [#25860](https://github.com/rust-lang/rust/issues/25860)
to implement the [lender design based on higher-rank trait bounds proposed by Sabrina Jewson](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats).

Similarly to what happens with standard iterators, besides the fundamental  [`Lender`] trait there is an
[`IntoLender`] trait, and methods such as [`for_each`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.for_each).

Indeed, the crate implements for [`Lender`]
all of the methods as `Iterator`, except `partition_in_place` and `array_chunks` (the latter being replaced by [`chunky`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.chunky)),
and most methods provide the same functionality as the equivalent `Iterator` method.

Notable differences in behavior include [`next_chunk`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.next_chunk) providing a lender instead of an array
and certain closures requiring usage of the [`hrc!`](https://docs.rs/lender/latest/lender/macro.hrc.html), [`hrc_mut!`](https://docs.rs/lender/latest/lender/macro.hrc_mut.html), [`hrc_once!`](https://docs.rs/lender/latest/lender/macro.hrc_once.html) (higher-rank closure) macros, which provide a stable replacement for the `closure_lifetime_binder` feature.

Turn a lender into an iterator with [`cloned`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.cloned)
where lend is `Clone`, [`copied`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.copied) where lend is `Copy`,
[`owned`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.owned) where lend is `ToOwned`, or
[`iter`](https://docs.rs/lender/latest/lender/trait.Lender.html#method.iter) where the lender already satisfies the restrictions of `Iterator`.

## Features

The `derive` feature (enabled by default) provides the
[`for_!`](https://docs.rs/lender-derive/latest/lender_derive/macro.for_.html) procedural macro
from the [`lender-derive`](https://docs.rs/lender-derive) crate.

## Usage

The Rust `for` syntax for iterating over types implementing `IntoIterator` will not work with lenders. The idiomatic way
of iterating over a lender is to use a `while let` loop, as in:

```text
while let Some(item) = lender.next() {
    // Do something with item
}
```

Note that the expression after the equal sign cannot be a method call returning a lender,
as you would iterate over the first element forever.

To simplify usage, we provide a function-like procedural macro
[`for_!`](https://docs.rs/lender-derive/latest/lender_derive/macro.for_.html) that makes it
possible to use a `for`-like syntax with types implementing [`IntoLender`]:

```text
for_!(item in into_lender {
    // Do something with item
});
```

Finally, you can use the `for_each` method, which takes a closure as argument, but managing lifetimes in closures can be
challenging:

```text
lender.for_each{
    hrc_mut!(for<'lend> |item: &'lend mut TYPE| {
        // do something with item of type TYPE 
    })
};
```

## Fallible Lenders

[Fallible lenders] offer the same semantics of [fallible
iterators](https://crates.io/crates/fallible-iterator), where the `next` method
returns a [`Result`] type, for lenders. They offer more flexibility than a
lender returning `Option<Result<…>>`: for example, they can short-circuit the
iteration on errors using the `?` operator; moreover, some adapters are
difficult or impossible to write for a lender returning `Option<Result<…>>`.

The idiomatic way of iterating over a fallible lender, propagating the error, is
to use a `while let` loop, as in:

```text
while let Some(item) = lender.next()? {
    // Do something with item
}
```

If you want to handle the error, you can use a three-armed match statement: 

```text
loop {
    match lender.next() {
        Err(e) => { /* handle error */ },
        Ok(None) => { /* end of iteration */ },
        Ok(Some(item)) => { /* do something with item */ },  
    }
}
```

If you have a [`Lender`] you can make it into a [`FallibleLender`] with
[`into_fallible`], and analogously for an [`IntoLender`]. You can also 
[obtain a fallible lender from a fallible iterator]. In general, all 
reasonable automatic conversions between iterators and lenders (fallible or not)
are provided.

## Binding the Lend

When writing methods accepting a [`Lender`], to bind the
type of the returned lend you need to use a higher-rank trait bound, as in:

```rust
use lender::*;

fn read_lender<L>(mut lender: L)
where
    L: Lender + for<'lend> Lending<'lend, Lend = &'lend str>,
{
    let _: Option<&'_ str> = lender.next(); 
}
```

You can also bind the lend using traits:

```rust
use lender::*;

fn read_lender<L>(mut lender: L)
where
    L: Lender + for<'lend> Lending<'lend, Lend: AsRef<str>>,
{
    let lend = lender.next();
    let _: Option<&'_ str> = lend.as_ref().map(AsRef::as_ref);
}
```

In this case, you can equivalently use the [`Lend`] type alias, which might be
more concise:

```rust
use lender::*;

fn read_lender<L>(mut lender: L)
where
    L: Lender,
    for<'lend> Lend<'lend, L>: AsRef<str>,
{
    let lend = lender.next();
    let _: Option<&'_ str> = lend.as_ref().map(AsRef::as_ref);
}
```

## Caveats

If a `dyn Lender` trait object is in your future, this crate is not going
to work.

Finally, note that, as a general rule, if you can avoid using lenders, you should.
You should heed the counsel of Polonius: “Neither a borrower nor a lender be”.

## Examples

Let us compute the Fibonacci numbers using mutable windows:

```rust
use ::lender::prelude::*;
use lender_derive::for_;

// Fibonacci sequence
let mut data = vec![0u32; 3 * 3];
data[1] = 1;

// Fibonacci sequence, most ergonomic usage: for_! procedural macro.
for_!(w in data.array_windows_mut::<3>() {
   w[2] = w[0] + w[1];
});
assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);

// You can use destructuring assignments with for_!.
for_!([a, b, c] in data.array_windows_mut::<3>() {
   *c = *a + *b;
});
assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);

// Fibonacci sequence, explicit while let loop: you MUST assign the lender to a variable.
let mut windows = data.array_windows_mut::<3>();
while let Some(w) = windows.next() {
   w[2] = w[0] + w[1];
}
assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);

// Fibonacci sequence, for_each with hrc_mut!
data.array_windows_mut::<3>()
    .for_each(hrc_mut!(for<'lend> |w: &'lend mut [u32; 3]| {
         w[2] = w[0] + w[1]
    }));
assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
```

This is a quite contrived example, but it shows how lenders can be used to mutate a slice in place.

So, let's look at a slightly more interesting example, `LinesStr`, an `io::Lines` with an `Item` of `&str` instead of `String`.
It's a good example of borrowing from the iterator itself.

```rust
use std::io;
use ::lender::prelude::*;

struct LinesStr<B> {
    buf: B,
    line: String,
}
impl<'lend, B: io::BufRead> Lending<'lend> for LinesStr<B> {
    type Lend = io::Result<&'lend str>;
}
impl<B: io::BufRead> Lender for LinesStr<B> {
    check_covariance!();
    fn next(&mut self) -> Option<io::Result<&'_ str>> {
        self.line.clear();
        match self.buf.read_line(&mut self.line) {
            Err(e) => return Some(Err(e)),
            Ok(0) => return None,
            Ok(_nread) => (),
        };
        if self.line.ends_with('\n') {
            self.line.pop();
            if self.line.ends_with('\r') {
                self.line.pop();
            }
        }
        Some(Ok(&self.line))
    }
}

let buf = io::BufReader::with_capacity(10, "Hello\nWorld\n".as_bytes());
let mut lines = LinesStr { buf, line: String::new() };
assert_eq!(lines.next().unwrap().unwrap(), "Hello");
assert_eq!(lines.next().unwrap().unwrap(), "World");
```

Note the [`check_covariance!`] macro invocation, which ensures that the lend is
covariant.

## Implementing Lender

To implement [`Lender`] first you'll need to implement the [`Lending`] trait for your type.
This is the equivalent provider of `Iterator::Item`:

```rust
use ::lender::prelude::*;
struct StrRef<'a>(&'a str);
impl<'this, 'lend> Lending<'lend> for StrRef<'this> {
    type Lend = &'lend str;
}
```

The lifetime parameter `'lend` describes the lifetime of the `Lend`.
It works by using under the hood a default generic of `&'lend Self` which induces an implicit
reference lifetime bound `'lend: 'this`, which is necessary for usage of
higher-rank trait bounds with `Lend`.

Next, you'll need to implement the [`Lender`]
trait for your type, the lending equivalent of `Iterator`.

```rust
use ::lender::prelude::*;
struct StrRef<'a>(&'a str);
impl<'this, 'lend> Lending<'lend> for StrRef<'this> {
    type Lend = &'lend str;
}
impl<'this> Lender for StrRef<'this> {
    check_covariance!();
    fn next(&mut self) -> Option<&'_ str> {
        Some(self.0)
    }
}
```

Note the [`check_covariance!`] macro invocation, which ensures that the lend is
covariant. There is an additional [`unsafe_assume_covariance!`] macro
that can be used when the lender wraps another lender to propagate covariance.

The [`Lend`] type alias can be used to avoid specifying twice the type of the lend;
combined with lifetime elision, it can make your implementations
more concise and less prone to errors:

```rust
use ::lender::prelude::*;
struct StrRef<'a>(&'a str);
impl<'this, 'lend> Lending<'lend> for StrRef<'this> {
    type Lend = &'lend str;
}
impl<'this> Lender for StrRef<'this> {
    check_covariance!();
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        Some(self.0)
    }
}
```

Implementing a [`FallibleLender`] is similar, using the
[`check_covariance_fallible!`] or [`unsafe_assume_covariance_fallible!`] macros
instead.

## Why Not GATs?

_Generic associated types_ (GATs) were introduced [exactly having lending iterators
as a use case in mind](https://rust-lang.github.io/rfcs/1598-generic_associated_types.html).
With GATs, a lender trait could be easily defined as

```rust
pub trait Lender {
    type Lend<'lend>: where Self: 'lend;
    fn next(&mut self) -> Option<Self::Lend<'_>>;
}
```

This looks all nice and cozy, and you can even write a full-fledged library around it.
But you will hit a wall when trying to specify trait bounds on the lend type, something that
can be done only using [higher-rank trait bounds](https://doc.rust-lang.org/nomicon/hrtb.html):

```rust
pub trait Lender {
    type Lend<'lend>: where Self: 'lend;
    fn next(&mut self) -> Option<Self::Lend<'_>>;
}

fn read_lender<L: Lender>(lender: L) 
    where for<'lend> L::Lend<'lend>: AsRef<str> {}
```

Again, this will compile without problems, but as you try to use `read_lender`
with a type implementing `Lender`, since the `where` clause specifies that
that trait bound must hold for all lifetimes, that means it must be valid
for `'static`, and since the lender must outlive the lend,
also the lender must be `'static`. Thus, until there is some syntax that makes it
possible to restrict the lifetime variable that appears in a higher-rank trait bound,
GAT-based lending iterators are of little practical use.

## Why Isn't [`CovariantLending`] In between [`Lending`] and [`Lender`]?

## Resources

Please check out the great resources below that helped us and many others learn
about Rust and the lending iterator problem. Thank you to everyone!

- [Sabrina Jewson's Blog](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats)
  for her awesome blog post on why lifetime GATs are not (yet)
  the solution to this problem, we highly recommend reading it.
- The awesome people on the [Rust Users Forum](https://users.rust-lang.org/) in
  helping us understand the borrow checker and HRTBs better and being patient
  with us and other aspiring rustaceans as we try to learn more about Rust.
- [Daniel Henry-Mantilla](https://github.com/danielhenrymantilla) for writing
  [`lending-iterator`] and many other great crates and sharing their great work.
- Everyone who's contributed to Rust for making such a great language and
  iterator library.

## Unsafe & Transmutes

Many patterns in lenders require polonius-emulating unsafe code,
but if you see any unsafe code that can be made safe, please let us know!

[`Lender`]: https://docs.rs/lender/latest/lender/trait.Lender.html
[`Lend`]: https://docs.rs/lender/latest/lender/type.Lend.html
[`Lending`]: https://docs.rs/lender/latest/lender/trait.Lending.html
[`CovariantLending`]: https://docs.rs/lender/latest/lender/trait.CovariantLending.html
[`FallibleLender`]: https://docs.rs/lender/latest/lender/trait.FallibleLender.html
[`IntoLender`]: https://docs.rs/lender/latest/lender/trait.IntoLender.html
[`into_fallible`]: https://docs.rs/lender/latest/lender/trait.Lender.html#method.into_fallible
[`lending-iterator`]: https://crates.io/crates/lending-iterator/
[fallible iterators]: https://crates.io/crates/fallible-iterator/
[`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
[Fallible lenders]: https://docs.rs/lender/latest/lender/trait.FallibleLender.html
[obtain a fallible lender from a fallible iterator]: https://docs.rs/lender/latest/lender/trait.FallibleIteratorExt.html#tymethod.into_fallible_lender
[`check_covariance!`]: https://docs.rs/lender/latest/lender/macro.check_covariance.html
[`check_covariance_fallible!`]: https://docs.rs/lender/latest/lender/macro.check_covariance_fallible.html
[`unsafe_assume_covariance!`]: https://docs.rs/lender/latest/lender/macro.unsafe_assume_covariance.html
[`unsafe_assume_covariance_fallible!`]: https://docs.rs/lender/latest/lender/macro.unsafe_assume_covariance_fallible.html
