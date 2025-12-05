use std::{cell::Cell, future::pending, ptr::null};

use yield_return::LocalIter;

#[test]
fn no_value() {
    let iter = LocalIter::<u32>::new(|mut _y| async move {});
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![]);
}

#[test]
fn values() {
    let iter = LocalIter::new(|mut y| async move {
        y.ret(1).await;
        y.ret(2).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![1, 2]);
}

#[test]
fn values_ret_iter() {
    let iter = LocalIter::new(|mut y| async move {
        y.ret_iter([1, 2]).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![1, 2]);
}

#[test]
fn fused() {
    let mut iter = LocalIter::new(|mut y| async move {
        y.ret(1).await;
        y.ret(2).await;
    });
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn values_with_lifetime() {
    let items = vec![1, 2];
    let items = &items;
    let iter = LocalIter::new(|mut y| async move {
        y.ret(&items[0]).await;
        y.ret(&items[1]).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![&1, &2]);
}

#[test]
#[should_panic]
fn use_pending() {
    let iter = LocalIter::<u32>::new(|mut y| async move {
        y.ret(1).await;
        pending::<()>().await;
        y.ret(2).await;
    });
    let _: Vec<_> = iter.collect();
}

#[test]
#[allow(unused_must_use)]
#[should_panic]
fn no_await_1() {
    let iter = LocalIter::new(|mut y| async move {
        y.ret(1);
    });
    let _: Vec<_> = iter.collect();
}

#[test]
#[allow(unused_must_use)]
#[should_panic]
fn no_await_2() {
    let iter = LocalIter::new(|mut y| async move {
        y.ret(1);
        y.ret(2);
    });
    let _: Vec<_> = iter.collect();
}

#[test]
fn check_not_send() {
    struct NotSend(#[allow(unused)] *const ());
    impl Drop for NotSend {
        fn drop(&mut self) {}
    }
    let iter = LocalIter::new(|mut y| async move {
        let _not_send = NotSend(null());
        y.ret(1).await;
        y.ret(2).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![1, 2]);
}

#[test]
fn ret_not_sync() {
    let iter = LocalIter::new(|mut y| async move {
        y.ret(Cell::new(1)).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![Cell::new(1)]);
}
