use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{
        Context,
        Poll::{self, Pending, Ready},
        Wake, Waker,
    },
};

struct ValueStore<T>(Rc<RefCell<Option<T>>>);

impl<T> ValueStore<T> {
    #[track_caller]
    fn set(&self, value: T) {
        let mut b = self.0.borrow_mut();
        assert!(
            b.is_none(),
            "The result of the previous `ret` is not await."
        );
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

pub struct YieldContext<T>(ValueStore<T>);

impl<T> YieldContext<T> {
    #[track_caller]
    pub fn ret(&mut self, value: T) -> impl Future<Output = ()> + '_ {
        self.0.set(value);
        &mut self.0
    }
}

struct RawIter<'a, T> {
    value: Rc<RefCell<Option<T>>>,
    fut: Pin<Box<dyn Future<Output = ()> + 'a>>,
    waker: Waker,
}

pub struct Yield<'a, T>(Option<RawIter<'a, T>>);

impl<'a, T: 'a> Yield<'a, T> {
    pub fn new<Fut: Future<Output = ()> + 'a>(f: impl FnOnce(YieldContext<T>) -> Fut) -> Self {
        let value = Rc::new(RefCell::new(None));
        let cx = YieldContext(ValueStore(value.clone()));
        let fut = Box::pin(f(cx));
        let waker = Arc::new(FakeWake).into();
        Self(Some(RawIter { value, fut, waker }))
    }
}
impl<T> Iterator for Yield<'_, T> {
    type Item = T;
    #[track_caller]
    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.0.as_mut()?;
        let poll = raw.fut.as_mut().poll(&mut Context::from_waker(&raw.waker));
        match poll {
            Ready(_) => {
                assert!(
                    raw.value.borrow().is_none(),
                    "The result of `ret` is not await."
                );
                self.0 = None;
                None
            }
            Pending => {
                if let Some(value) = raw.value.borrow_mut().take() {
                    Some(value)
                } else {
                    panic!("`YieldContext::ret` is not called.");
                }
            }
        }
    }
}
struct FakeWake;
impl Wake for FakeWake {
    fn wake(self: Arc<Self>) {}
    fn wake_by_ref(self: &Arc<Self>) {}
}
