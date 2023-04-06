# Lender

Please see [`lending-iterator`] for an actual implementation of lending iterators.

This is just my own personal practice for learning more about the Rust borrow checker and Higher-Ranked Trait Bounds,
and what better way than a problem I'm way out of my depth with?

Check out the [`lending-iterator`] crate for an actual implementation of lending iterators,
it's lead by the great [Daniel Henry-Mantilla][Daniel], who knows **much** more about this than I do and is one of the lead champions of Higher-Kinded things like this in the Rust community. The [`streaming-iterator`] crate is also a great crate if you only need to iterate over a reference to an item.

If, for whatever unknown reason, you want to use this crate instead of [`lending-iterator`], this is licensed under the MIT or Apache-2.0 license. I've obviously repurposed much of the code from the Rust core lib and using the core library, so I will be using the same license as the Rust core library.

## People & Resources

I couldn't have even hoped to begin trying to use this problem
as practice without relying on help from the following people and resources,
please check them out if you're interested in learning more about Rust:

- [Daniel Henry-Mantilla][Daniel] by writing [`lending-iterator`] and many other great crates for complex Rust problems
- [Sabrina Jewson's Blog](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats) for their awesome
blog post on why lifetime GATs are not (yet) the solution to this problem, I highly recommend reading it if you're interested in HRTBs.
- [@vague](https://users.rust-lang.org/u/vague), [Cole Miller](https://users.rust-lang.org/u/cole-miller),
and awesome people on the [Rust Users Forum](https://users.rust-lang.org/) in helping me understand the borrow checker and HRTBs better
and being patient with me and other aspiring rustaceans as we try to learn more about Rust.
- Finally, it goes without saying, everyone who's contributed to Rust core library for making such a great language and iterator library.

<!-- markdownlint-disable MD026 -->
## Unsafe & Transmutes Beware!!!

But please, if you see any unsafe code that can be made safe, please let me know! I am still learning Rust and I'm sure I've made many mistakes.

[Daniel]: https://github.com/danielhenrymantilla
[`lending-iterator`]: https://crates.io/crates/lending-iterator
[`streaming-iterator`]: https://crates.io/crates/streaming-iterator
