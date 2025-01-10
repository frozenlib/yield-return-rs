use std::{
    future::Future,
    iter::FusedIterator,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use crate::utils::fake_waker;

struct Sender<T>(Arc<Mutex<Option<T>>>);

impl<T> Sender<T> {
    #[track_caller]
    fn set(&self, value: T) {
        let mut guard = self.0.lock().unwrap();
        assert!(guard.is_none(), "The result of `ret` is not await.");
        *guard = Some(value);
    }
}

impl<T> Future for Sender<T> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        if self.0.lock().unwrap().is_some() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// A context for asynchronous function to implement iterator.
pub struct IterContext<T>(Sender<T>);

impl<T> IterContext<T>
where
    T: Send,
{
    /// Yields a single value. Similar to C#'s `yield return` or Python's `yield`.
    #[track_caller]
    pub fn ret(&mut self, value: T) -> impl Future<Output = ()> + Send + Sync + '_ {
        self.0.set(value);
        &mut self.0
    }

    /// Yields all values from an iterator. Similar to Python's `yield from` or JavaScript's `yield*`.
    pub async fn ret_iter(&mut self, iter: impl IntoIterator<Item = T> + Send + Sync) {
        for value in iter {
            self.ret(value).await;
        }
    }
}

struct Data<'a, T> {
    value: Arc<Mutex<Option<T>>>,
    fut: Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
    waker: Waker,
}

/// An iterator implemented by asynchronous function.
pub struct Iter<'a, T>(Option<Data<'a, T>>);

impl<'a, T: 'a + Send> Iter<'a, T> {
    /// Create an iterator from an asynchronous function.
    ///
    /// # Example
    /// ```
    /// use yield_return::Yield;
    /// let iter = Yield::new(|mut y| async move {
    ///     y.ret(1).await;
    ///     y.ret(2).await;
    /// });
    /// let list: Vec<_> = iter.collect();
    /// assert_eq!(list, vec![1, 2]);
    /// ```
    pub fn new<Fut: Future<Output = ()> + Send + 'a>(
        f: impl FnOnce(IterContext<T>) -> Fut + Send,
    ) -> Self {
        let value = Arc::new(Mutex::new(None));
        let cx = IterContext(Sender(value.clone()));
        let fut = Box::pin(f(cx));
        let waker = fake_waker();
        Self(Some(Data { value, fut, waker }))
    }
}

impl<T> Iterator for Iter<'_, T> {
    type Item = T;
    #[track_caller]
    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.0.as_mut()?;
        let poll = raw.fut.as_mut().poll(&mut Context::from_waker(&raw.waker));
        match poll {
            Poll::Ready(_) => {
                assert!(
                    raw.value.lock().unwrap().is_none(),
                    "The result of `ret` is not await."
                );
                self.0 = None;
                None
            }
            Poll::Pending => {
                if let Some(value) = raw.value.lock().unwrap().take() {
                    Some(value)
                } else {
                    panic!("`YieldContext::ret` is not called.");
                }
            }
        }
    }
}
impl<T> FusedIterator for Iter<'_, T> {}
