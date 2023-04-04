use std::future::pending;

use yield_return::Yield;

#[test]
fn no_value() {
    let iter = Yield::<u32>::new(|mut _y| async move {});
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![]);
}

#[test]
fn values() {
    let iter = Yield::new(|mut y| async move {
        y.ret(1).await;
        y.ret(2).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![1, 2]);
}

#[test]
fn values_with_lifetime() {
    let items = vec![1, 2];
    let items = &items;
    let iter = Yield::new(|mut y| async move {
        y.ret(&items[0]).await;
        y.ret(&items[1]).await;
    });
    let list: Vec<_> = iter.collect();
    assert_eq!(list, vec![&1, &2]);
}

#[test]
#[allow(unused_must_use)]
#[should_panic]
fn no_await_1() {
    let iter = Yield::new(|mut y| async move {
        y.ret(1);
    });
    let _: Vec<_> = iter.collect();
}

#[test]
#[allow(unused_must_use)]
#[should_panic]
fn no_await_2() {
    let iter = Yield::new(|mut y| async move {
        y.ret(1);
        y.ret(2);
    });
    let _: Vec<_> = iter.collect();
}

#[test]
#[should_panic]
fn invalid_await() {
    let iter = Yield::<u32>::new(|_y| async move {
        pending::<()>().await;
    });
    let _: Vec<_> = iter.collect();
}
