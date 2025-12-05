use std::{
    cell::RefCell,
    future::Future,
    iter::FusedIterator,
    ops::{Deref, DerefMut},
    pin::{Pin, pin},
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::{Stream, StreamExt, stream::FusedStream};

struct Sender<T>(Rc<RefCell<Option<T>>>);

impl<T> Sender<T> {
    #[track_caller]
    fn set(&self, value: T) {
        let mut data = self.0.borrow_mut();
        assert!(data.is_none(), "The result of `ret` is not await.");
        *data = Some(value);
    }
}

impl<T> Future for Sender<T> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        if self.0.borrow().is_some() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// Non-`Send` iterator context.
///
/// This type does not implement `Send`.
pub struct LocalIterContext<T>(Sender<T>);

impl<T> LocalIterContext<T> {
    /// Yields a single value. Similar to C#'s `yield return` or Python's `yield`.
    #[track_caller]
    pub fn ret(&mut self, value: T) -> impl Future<Output = ()> {
        self.0.set(value);
        &mut self.0
    }

    /// Yields all values from an iterator. Similar to Python's `yield from` or JavaScript's `yield*`.
    pub async fn ret_iter(&mut self, iter: impl IntoIterator<Item = T>) {
        for value in iter {
            self.ret(value).await;
        }
    }
}

struct Data<'a, T> {
    value: Rc<RefCell<Option<T>>>,
    fut: Option<Pin<Box<dyn Future<Output = ()> + 'a>>>,
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
                    self.value.borrow().is_none(),
                    "The result of `ret` is not await."
                );
                self.fut = None;
                Poll::Ready(None)
            }
            Poll::Pending => {
                if let Some(value) = self.value.borrow_mut().take() {
                    Poll::Ready(Some(value))
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

/// Non-`Send` iterator implemented using async functions.
///
/// This type does not implement `Send`.
pub struct LocalIter<'a, T>(Data<'a, T>);

impl<'a, T: 'a> LocalIter<'a, T> {
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
    pub fn new<Fut: Future<Output = ()> + 'a>(f: impl FnOnce(LocalIterContext<T>) -> Fut) -> Self {
        let value = Rc::new(RefCell::new(None));
        let cx = LocalIterContext(Sender(value.clone()));
        let fut: Pin<Box<dyn Future<Output = ()> + 'a>> = Box::pin(f(cx));
        let fut = Some(fut);
        Self(Data { value, fut })
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_iter(iter: impl IntoIterator<Item = T, IntoIter: 'a>) -> Self {
        let iter = iter.into_iter();
        Self::new(|mut cx| async move {
            cx.ret_iter(iter).await;
        })
    }
}

impl<T> Iterator for LocalIter<'_, T> {
    type Item = T;
    #[track_caller]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.poll_next(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("`YieldContext::ret` is not called."),
        }
    }
}
impl<T> FusedIterator for LocalIter<'_, T> {}

/// Non-`Send` stream context.
///
/// This type does not implement `Send`.
pub struct LocalAsyncIterContext<T>(LocalIterContext<T>);

impl<T> LocalAsyncIterContext<T> {
    /// Yields all values from a stream.
    pub async fn ret_stream(&mut self, stream: impl Stream<Item = T>) {
        let mut stream = pin!(stream);
        while let Some(value) = stream.next().await {
            self.0.ret(value).await;
        }
    }
}
impl<T> Deref for LocalAsyncIterContext<T> {
    type Target = LocalIterContext<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for LocalAsyncIterContext<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Non-`Send` stream implemented using async functions.
///
/// This type does not implement `Send`.
pub struct LocalAsyncIter<'a, T>(LocalIter<'a, T>);

impl<'a, T: 'a> LocalAsyncIter<'a, T> {
    /// Create a stream from an asynchronous function.
    ///
    /// # Example
    /// ```
    /// use yield_return::LocalAsyncIter;
    /// # futures::executor::block_on(async {
    /// let iter = LocalAsyncIter::new(|mut y| async move {
    ///     y.ret(1).await;
    ///     y.ret(2).await;
    /// });
    /// let list: Vec<_> = futures::StreamExt::collect(iter).await;
    /// assert_eq!(list, vec![1, 2]);
    /// # });
    /// ```
    pub fn new<Fut: Future<Output = ()> + 'a>(
        f: impl FnOnce(LocalAsyncIterContext<T>) -> Fut,
    ) -> Self {
        Self(LocalIter::new(|cx| f(LocalAsyncIterContext(cx))))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_iter(iter: impl IntoIterator<Item = T, IntoIter: 'a>) -> Self {
        let iter = iter.into_iter();
        Self::new(|mut cx| async move {
            cx.ret_iter(iter).await;
        })
    }
}

impl<T> Stream for LocalAsyncIter<'_, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.0.poll_next(cx)
    }
}
impl<T> FusedStream for LocalAsyncIter<'_, T> {
    fn is_terminated(&self) -> bool {
        self.0.0.fut.is_none()
    }
}
