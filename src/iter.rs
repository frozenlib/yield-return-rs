use std::{
    future::Future,
    iter::FusedIterator,
    ops::{Deref, DerefMut},
    pin::{Pin, pin},
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures::{Stream, StreamExt, stream::FusedStream};

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

/// `Send` iterator context.
///
/// This type implements `Send`.
pub struct IterContext<T>(Sender<T>);

impl<T> IterContext<T>
where
    T: Send,
{
    /// Yields a single value. Similar to C#'s `yield return` or Python's `yield`.
    #[track_caller]
    pub fn ret(&mut self, value: T) -> impl Future<Output = ()> + Send + Sync {
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
    fut: Option<Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>>>,
}
impl<T> Data<'_, T> {
    fn poll_next(&mut self, cx: &mut Context) -> Poll<Option<T>> {
        let Some(fut) = &mut self.fut else {
            return Poll::Ready(None);
        };
        let poll = fut.as_mut().poll(cx);
        match poll {
            Poll::Ready(_) => {
                assert!(
                    self.value.lock().unwrap().is_none(),
                    "The result of `ret` is not await."
                );
                self.fut = None;
                Poll::Ready(None)
            }
            Poll::Pending => {
                if let Some(value) = self.value.lock().unwrap().take() {
                    Poll::Ready(Some(value))
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

/// `Send` iterator implemented using async functions.
///
/// This type implements `Send`.
pub struct Iter<'a, T>(Data<'a, T>);

impl<'a, T: 'a + Send> Iter<'a, T> {
    /// Create an iterator from an asynchronous function.
    ///
    /// # Example
    ///
    /// ```
    /// use yield_return::Yield;
    /// let iter = Yield::new(|mut y| async move {
    ///     y.ret(1).await;
    ///     y.ret(2).await;
    /// });
    /// let list: Vec<_> = iter.collect();
    /// assert_eq!(list, vec![1, 2]);
    /// ```
    pub fn new<Fut: Future<Output = ()> + Send + Sync + 'a>(
        f: impl FnOnce(IterContext<T>) -> Fut,
    ) -> Self {
        let value = Arc::new(Mutex::new(None));
        let cx = IterContext(Sender(value.clone()));
        let fut: Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>> = Box::pin(f(cx));
        let fut = Some(fut);
        Self(Data { value, fut })
    }
}

impl<T> Iterator for Iter<'_, T> {
    type Item = T;
    #[track_caller]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.poll_next(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("`YieldContext::ret` is not called."),
        }
    }
}
impl<T> FusedIterator for Iter<'_, T> {}

/// `Send` stream context.
///
/// This type implements `Send`.
pub struct AsyncIterContext<T>(IterContext<T>);

impl<T: Send> AsyncIterContext<T> {
    /// Yields all values from a stream.
    pub async fn ret_stream(&mut self, stream: impl Stream<Item = T> + Send) {
        let mut stream = pin!(stream);
        while let Some(value) = stream.next().await {
            self.0.ret(value).await;
        }
    }
}

impl<T> Deref for AsyncIterContext<T> {
    type Target = IterContext<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for AsyncIterContext<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// `Send` stream implemented using async functions.
///
/// This type implements `Send`.
pub struct AsyncIter<'a, T>(Iter<'a, T>);

impl<'a, T: Send + 'a> AsyncIter<'a, T> {
    /// Create a stream from an asynchronous function.
    ///
    /// # Example
    /// ```
    /// use yield_return::AsyncIter;
    /// # futures::executor::block_on(async {
    /// let iter = AsyncIter::new(|mut y| async move {
    ///     y.ret(1).await;
    ///     y.ret(2).await;
    /// });
    /// let list: Vec<_> = futures::StreamExt::collect(iter).await;
    /// assert_eq!(list, vec![1, 2]);
    /// # });
    /// ```
    pub fn new<Fut: Future<Output = ()> + Send + Sync + 'a>(
        f: impl FnOnce(AsyncIterContext<T>) -> Fut + Send + Sync,
    ) -> Self {
        Self(Iter::new(|cx| f(AsyncIterContext(cx))))
    }
}

impl<T> Stream for AsyncIter<'_, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.0.poll_next(cx)
    }
}
impl<T> FusedStream for AsyncIter<'_, T> {
    fn is_terminated(&self) -> bool {
        self.0.0.fut.is_none()
    }
}
