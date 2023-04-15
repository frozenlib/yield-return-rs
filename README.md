# yield-return-rs

Implement a coroutine like C#'s `yield return` using Rust's `async`, `await`.

## Exmaple

```rust
use yield_return::Yield;
let iter = Yield::new(|mut y| async move {
    y.ret(1).await;
    y.ret(2).await;
});
let list: Vec<_> = iter.collect();
assert_eq!(list, vec![1, 2]);
```

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
