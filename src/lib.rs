//! This crate provides a way to implement something like C#'s `yield return` using an asynchronous function.
//!
//! # Example
//!
//! ```
//! use yield_return::Yield;
//! let iter = Yield::new(|mut y| async move {
//!     y.ret(1).await;
//!     y.ret(2).await;
//! });
//! let list: Vec<_> = iter.collect();
//! assert_eq!(list, vec![1, 2]);
//! ```
//!
//! ## Available Types
//!
//! This crate provides several iterator types that differ based on two characteristics:
//!
//! - Whether they implement [`Iterator`] or [`Stream`]
//! - Whether they require and implement [`Send`]
//!
//! The following table shows the available types:
//!
//! |              | `Send`        | Not `Send`       |
//! | ------------ | ------------- | ------------------ |
//! | [`Iterator`] | [`Iter`]      | [`LocalIter`]      |
//! | [`Stream`]   | [`AsyncIter`] | [`LocalAsyncIter`] |
//!
//! [`Stream`]: futures_core::stream::Stream

mod iter;
mod local_iter;
mod utils;

#[cfg(doctest)]
mod tests_readme;

pub use iter::{AsyncIter, AsyncIterContext, Iter, IterContext};
pub use local_iter::{LocalAsyncIter, LocalAsyncIterContext, LocalIter, LocalIterContext};

#[deprecated(since = "0.2.0", note = "Use `LocalIter` instead.")]
pub type Yield<'a, T> = LocalIter<'a, T>;

#[deprecated(since = "0.2.0", note = "Use `LocalIterContext` instead.")]
pub type YieldContext<T> = LocalIterContext<T>;
