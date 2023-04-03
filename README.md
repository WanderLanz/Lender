# Lender (DO NOT USE, THIS IS JUST FOR PRACTICE)

Please see [`lending-iterator`] for an actual implementation of lending iterators.

This is primarily just my own personal practice for learning more about the Rust borrow checker and Higher-Ranked Trait Bounds,
and what better way than a problem I'm way out of my depth with?

An iterator that yields items bound by the lifetime of each iteration (no `dyn Lender`).

Check out the [`lending-iterator`] crate for a more complete and ergonomic implementation of lending iterators,
it's written by the great [Daniel Henry-Mantilla][1], who knows much more about this than I do. The [`streaming-iterator`] crate is also a great crate if you only need to iterate over a reference to an item.

I'm currently targetting a no-std blanket implementation of the standard library's `iter` module.
Although, I am omitting dyn traits and other things that would require a much more complex implementation that [`lending-iterator`] already provides.

## People & Resources

I couldn't have even hoped to begin trying to use this problem
as practice without relying on help from the following people and resources:

- [Daniel Henry-Mantilla][1] by writing [`lending-iterator`] and many other great crates for complex Rust problems
- [Sabrina Jewson's Blog](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats) for their awesome
blog post on why lifetime GATs are not (yet) the solution to this problem, I highly recommend reading it if you're interested in HRTBs.
- [@vague](https://users.rust-lang.org/u/vague), [Cole Miller](https://users.rust-lang.org/u/cole-miller),
and awesome people on the [Rust Users Forum](https://users.rust-lang.org/) in helping me understand the borrow checker and HRTBs better
and being patient with me and other aspiring rustaceans as we try to learn more about Rust.

<!-- markdownlint-disable MD026 -->
## Unsafe & Transmutes Beware!!!

But please, if you see any unsafe code that can be made safe, please let me know! I am still learning Rust and I'm sure I've made many mistakes.

[1]: https://github.com/danielhenrymantilla
[`lending-iterator`]: https://crates.io/crate/lending-iterator
[`streaming-iterator`]: https://crates.io/crates/streaming-iterator
