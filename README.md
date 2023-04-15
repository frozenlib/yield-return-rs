# yield-return-rs

[![Actions Status](https://github.com/frozenlib/yield-return-rs/workflows/CI/badge.svg)](https://github.com/frozenlib/yield-return-rs/actions)

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

## Compare with [`genawaiter`](https://github.com/whatisaphone/genawaiter)

`genawaiter` already exists as a crate with the same purpose as this `yield-return-rs`.

Compared to `genawaiter`, `yield-return-rs` is very simple.

No dependencies, no macros, no unsafe code.

The code is short, with only one file, `lib.rs`. You can copy and paste the contents of `lib.rs` and use it as is.

|                            | yield-return-rs | genawaiter |
| -------------------------- | --------------- | ---------- |
| `Rc` based implementation  | ✔               | ✔          |
| stack based implementation |                 | ✔          |
| `Sync` implementation      |                 | ✔          |
| `Iterator` support         | ✔               | ✔          |
| `Generator` support        |                 | ✔          |
| no-dependencies            | ✔               |            |
| no-macros                  | ✔               |            |
| safe code only             | ✔               |            |
| `lib.rs` only              | ✔               |            |
| number of public types     | 2               | many       |

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
