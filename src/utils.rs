use std::{
    sync::{Arc, OnceLock},
    task::{Wake, Waker},
};

struct NoopWake;
impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
    fn wake_by_ref(self: &Arc<Self>) {}
}

// TODO: Remove this function when `std::task::Waker::noop` is stabilized.
pub fn noop_waker() -> Waker {
    static WAKER: OnceLock<Arc<NoopWake>> = OnceLock::new();
    WAKER.get_or_init(|| Arc::new(NoopWake)).clone().into()
}
