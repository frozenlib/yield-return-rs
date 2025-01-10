use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

pub async fn sleep(d: Duration) {
    Sleep(Instant::now() + d).await;
}

struct Sleep(Instant);

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let now = Instant::now();
        if now >= self.0 {
            Poll::Ready(())
        } else {
            let waker = cx.waker().clone();
            let d = self.0 - now;
            std::thread::spawn(move || {
                std::thread::sleep(d);
                waker.wake();
            });
            Poll::Pending
        }
    }
}
