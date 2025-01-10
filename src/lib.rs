//! This crate provides a way to implement something like C#'s `yield return` using an asynchronous function.
//!
//! See [`Iter::new`] for details.

mod iter;
mod local_iter;
mod utils;

pub use iter::{AsyncIter, AsyncIterContext, Iter, IterContext};
pub use local_iter::{LocalAsyncIter, LocalAsyncIterContext, LocalIter, LocalIterContext};

#[deprecated(since = "0.2.0", note = "Use `LocalIter` instead.")]
pub type Yield<'a, T> = LocalIter<'a, T>;

#[deprecated(since = "0.2.0", note = "Use `LocalIterContext` instead.")]
pub type YieldContext<T> = LocalIterContext<T>;
