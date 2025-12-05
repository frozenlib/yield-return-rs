use std::{cell::Cell, future::pending};

use yield_return::Iter;

#[test]
fn no_value() {
    let iter = Iter::<u32>::new(|mut _y| async move {});
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![]);
}

#[test]
fn values() {
    let iter = Iter::new(|mut y| async move {
        y.ret(1).await;
        y.ret(2).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![1, 2]);
}

#[test]
fn values_ret_iter() {
    let iter = Iter::new(|mut y| async move {
        y.ret_iter([1, 2]).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![1, 2]);
}

#[test]
fn fused() {
    let mut iter = Iter::new(|mut y| async move {
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
    let iter = Iter::new(|mut y| async move {
        y.ret(&items[0]).await;
        y.ret(&items[1]).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![&1, &2]);
}

#[test]
#[should_panic]
fn use_pending() {
    let iter = Iter::<u32>::new(|mut y| async move {
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
    let iter = Iter::new(|mut y| async move {
        y.ret(1);
    });
    let _: Vec<_> = iter.collect();
}

#[test]
#[allow(unused_must_use)]
#[should_panic]
fn no_await_2() {
    let iter = Iter::new(|mut y| async move {
        y.ret(1);
        y.ret(2);
    });
    let _: Vec<_> = iter.collect();
}

#[test]
fn check_send() {
    let iter = Iter::new(|mut y| async move {
        y.ret(1).await;
    });
    fn f(_: impl Send) {}
    f(iter);
}

#[test]
fn ret_not_sync() {
    let iter = Iter::new(|mut y| async move {
        y.ret(Cell::new(1)).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![Cell::new(1)]);
}
