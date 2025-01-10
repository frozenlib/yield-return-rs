use std::{
    sync::{Arc, OnceLock},
    task::{Wake, Waker},
};

struct FakeWake;
impl Wake for FakeWake {
    fn wake(self: Arc<Self>) {}
    fn wake_by_ref(self: &Arc<Self>) {}
}

pub fn fake_waker() -> Waker {
    static WAKER: OnceLock<Arc<FakeWake>> = OnceLock::new();
    WAKER.get_or_init(|| Arc::new(FakeWake)).clone().into()
}
