use std::{cell::Cell, ptr::null, time::Duration};

use futures::{StreamExt, stream};
use rt_local::runtime::core::test;
use utils::sleep;
use yield_return::LocalAsyncIter;

mod utils;

#[test]
async fn no_value() {
    let iter = LocalAsyncIter::<u32>::new(|mut _y| async move {});
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![]);
}

#[test]
async fn values() {
    let iter = LocalAsyncIter::new(|mut y| async move {
        y.ret(1).await;
        y.ret(2).await;
    });
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![1, 2]);
}

#[test]
async fn values_ret_iter() {
    let iter = LocalAsyncIter::new(|mut y| async move {
        y.ret_iter([1, 2]).await;
    });
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![1, 2]);
}

#[test]
async fn values_ret_stream() {
    let iter = LocalAsyncIter::new(|mut y| async move {
        y.ret_stream(stream::iter([1, 2])).await;
    });
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![1, 2]);
}

#[test]
async fn fused() {
    let mut iter = LocalAsyncIter::new(|mut y| async move {
        y.ret(1).await;
        y.ret(2).await;
    });
    assert_eq!(iter.next().await, Some(1));
    assert_eq!(iter.next().await, Some(2));
    assert_eq!(iter.next().await, None);
    assert_eq!(iter.next().await, None);
}

#[test]
async fn values_with_lifetime() {
    let items = vec![1, 2];
    let items = &items;
    let iter = LocalAsyncIter::new(|mut y| async move {
        y.ret(&items[0]).await;
        y.ret(&items[1]).await;
    });
    let list: Vec<&i32> = iter.collect().await;
    assert_eq!(list, vec![&1, &2]);
}

#[test]
async fn use_sleep() {
    let iter = LocalAsyncIter::<u32>::new(|mut y| async move {
        y.ret(1).await;
        sleep(Duration::from_millis(100)).await;
        y.ret(2).await;
    });
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![1, 2]);
}

#[test]
#[allow(unused_must_use)]
#[should_panic]
async fn no_await_1() {
    let iter = LocalAsyncIter::new(|mut y| async move {
        y.ret(1);
    });
    let _: Vec<_> = iter.collect().await;
}

#[test]
#[allow(unused_must_use)]
#[should_panic]
async fn no_await_2() {
    let iter = LocalAsyncIter::new(|mut y| async move {
        y.ret(1);
        y.ret(2);
    });
    let _: Vec<_> = iter.collect().await;
}

#[test]
async fn check_not_send() {
    struct NotSend(#[allow(unused)] *const ());
    impl Drop for NotSend {
        fn drop(&mut self) {}
    }
    let iter = LocalAsyncIter::new(|mut y| async move {
        let _not_send = NotSend(null());
        y.ret(1).await;
        y.ret(2).await;
    });
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![1, 2]);
}

#[test]
async fn ret_not_sync() {
    let iter = LocalAsyncIter::new(|mut y| async move {
        y.ret(Cell::new(1)).await;
    });
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![Cell::new(1)]);
}
