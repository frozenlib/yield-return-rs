# yield-return-rs

[![Crates.io](https://img.shields.io/crates/v/yield-return.svg)](https://crates.io/crates/yield-return)
[![Docs.rs](https://docs.rs/yield-return/badge.svg)](https://docs.rs/yield-return/)
[![Actions Status](https://github.com/frozenlib/yield-return-rs/workflows/CI/badge.svg)](https://github.com/frozenlib/yield-return-rs/actions)

Implement a coroutine like C#'s `yield return` using Rust's `async`, `await`.

## Example

```rust
use yield_return::Iter;
let iter = Iter::new(|mut y| async move {
    y.ret(1).await;
    y.ret(2).await;
});
let list: Vec<_> = iter.collect();
assert_eq!(list, vec![1, 2]);
```

## Available Types

This crate provides several iterator types that differ based on two characteristics:

- Whether they implement [`Iterator`] or [`Stream`]
- Whether they require and implement [`Send`]

The following table shows the available types:

|              | `Send`        | Not `Send`         |
| ------------ | ------------- | ------------------ |
| [`Iterator`] | [`Iter`]      | [`LocalIter`]      |
| [`Stream`]   | [`AsyncIter`] | [`LocalAsyncIter`] |

[`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
[`Stream`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
[`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
[`Iter`]: https://docs.rs/yield-return/latest/yield_return/struct.Iter.html
[`LocalIter`]: https://docs.rs/yield-return/latest/yield_return/struct.LocalIter.html
[`AsyncIter`]: https://docs.rs/yield-return/latest/yield_return/struct.AsyncIter.html
[`LocalAsyncIter`]: https://docs.rs/yield-return/latest/yield_return/struct.LocalAsyncIter.html

## Compare with other crates

While [async-stream] and [genawaiter] serve similar purposes, [yield-return] focuses on usability over performance. This design philosophy is reflected in two key characteristics:

- Type parameters only expose the external interface, not implementation details
- Clean API using only types and functions (no macros)

[async-stream]: https://crates.io/crates/async-stream
[genawaiter]: https://crates.io/crates/genawaiter
[yield-return]: https://crates.io/crates/yield-return

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
