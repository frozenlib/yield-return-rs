use std::time::Duration;

use futures::StreamExt;
use rt_local::runtime::core::test;
use utils::sleep;
use yield_return::AsyncIter;

mod utils;

#[test]
async fn no_value() {
    let iter = AsyncIter::<u32>::new(|mut _y| async move {});
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![]);
}

#[test]
async fn values() {
    let iter = AsyncIter::new(|mut y| async move {
        y.ret(1).await;
        y.ret(2).await;
    });
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![1, 2]);
}

#[test]
async fn values_ret_iter() {
    let iter = AsyncIter::new(|mut y| async move {
        y.ret_iter([1, 2]).await;
    });
    let list: Vec<_> = iter.collect().await;
    assert_eq!(list, vec![1, 2]);
}

#[test]
async fn fused() {
    let mut iter = AsyncIter::new(|mut y| async move {
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
    let iter = AsyncIter::new(|mut y| async move {
        y.ret(&items[0]).await;
        y.ret(&items[1]).await;
    });
    let list: Vec<&i32> = iter.collect().await;
    assert_eq!(list, vec![&1, &2]);
}

#[test]
async fn use_sleep() {
    let iter = AsyncIter::<u32>::new(|mut y| async move {
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
    let iter = AsyncIter::new(|mut y| async move {
        y.ret(1);
    });
    let _: Vec<_> = iter.collect().await;
}

#[test]
#[allow(unused_must_use)]
#[should_panic]
async fn no_await_2() {
    let iter = AsyncIter::new(|mut y| async move {
        y.ret(1);
        y.ret(2);
    });
    let _: Vec<_> = iter.collect().await;
}

#[test]
fn check_sync_send() {
    let iter = AsyncIter::new(|mut y| async move {
        y.ret(1).await;
    });
    fn f(_: impl Send + Sync) {}
    f(iter);
}
