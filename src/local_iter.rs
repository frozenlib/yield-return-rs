use std::{
    cell::RefCell,
    future::Future,
    iter::FusedIterator,
    pin::Pin,
    rc::Rc,
    sync::{Arc, OnceLock},
    task::{Context, Poll, Wake, Waker},
};

struct ValueStore<T>(Rc<RefCell<Option<T>>>);

impl<T> ValueStore<T> {
    #[track_caller]
    fn set(&self, value: T) {
        let mut b = self.0.borrow_mut();
        assert!(b.is_none(), "The result of `ret` is not await.");
        *b = Some(value);
    }
}
impl<T> Future for ValueStore<T> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        if self.0.borrow().is_some() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// A context for asynchronous function to implement iterator.
pub struct LocalIterContext<T>(ValueStore<T>);

impl<T> LocalIterContext<T> {
    /// Yields a single value. Similar to C#'s `yield return` or Python's `yield`.
    #[track_caller]
    pub fn ret(&mut self, value: T) -> impl Future<Output = ()> + '_ {
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

struct RawYield<'a, T> {
    value: Rc<RefCell<Option<T>>>,
    fut: Pin<Box<dyn Future<Output = ()> + 'a>>,
    waker: Waker,
}

/// An iterator implemented by asynchronous function.
pub struct LocalIter<'a, T>(Option<RawYield<'a, T>>);

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
        let cx = LocalIterContext(ValueStore(value.clone()));
        let fut = Box::pin(f(cx));
        static WAKER: OnceLock<Arc<FakeWake>> = OnceLock::new();
        let waker = WAKER.get_or_init(|| Arc::new(FakeWake)).clone().into();
        Self(Some(RawYield { value, fut, waker }))
    }
}
impl<T> Iterator for LocalIter<'_, T> {
    type Item = T;
    #[track_caller]
    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.0.as_mut()?;
        let poll = raw.fut.as_mut().poll(&mut Context::from_waker(&raw.waker));
        match poll {
            Poll::Ready(_) => {
                assert!(
                    raw.value.borrow().is_none(),
                    "The result of `ret` is not await."
                );
                self.0 = None;
                None
            }
            Poll::Pending => {
                if let Some(value) = raw.value.borrow_mut().take() {
                    Some(value)
                } else {
                    panic!("`YieldContext::ret` is not called.");
                }
            }
        }
    }
}
impl<T> FusedIterator for LocalIter<'_, T> {}

struct FakeWake;
impl Wake for FakeWake {
    fn wake(self: Arc<Self>) {}
    fn wake_by_ref(self: &Arc<Self>) {}
}
