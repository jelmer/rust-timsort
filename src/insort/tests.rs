use crate::{never, NeverResult};

/// Test the insertion sort implementation with an empty list
#[test]
fn empty() {
    let mut list: Vec<u32> = vec![];
    sort(&mut list);
    assert!(list.is_empty());
}

/// Test the insertion sort implementation with a single-element list
#[test]
fn single() {
    let mut list = vec![42];
    sort(&mut list);
    assert!(list[0] == 42);
}

/// Test the insertion sort implementation with a short unsorted list
#[test]
fn unsorted() {
    let mut list = vec![3, 1, 0, 4];
    sort(&mut list);
    assert!(list[0] == 0);
    assert!(list[1] == 1);
    assert!(list[2] == 3);
    assert!(list[3] == 4);
}

/// Test the insertion sort implementation with a short backward list
#[test]
fn reverse() {
    let mut list = vec![21, 18, 7, 1];
    sort(&mut list);
    assert!(list[0] == 1);
    assert!(list[1] == 7);
    assert!(list[2] == 18);
    assert!(list[3] == 21);
}

/// Test the insertion sort implementation with a short unsorted list
#[test]
fn sorted() {
    let mut list = vec![0, 1, 2, 3];
    sort(&mut list);
    assert!(list[0] == 0);
    assert!(list[1] == 1);
    assert!(list[2] == 2);
    assert!(list[3] == 3);
}

/// Make sure the sort is stable.
#[test]
fn stable() {
    let len = 256;
    let mut key1: usize = 0;
    let mut key2: usize = 0;
    #[derive(Debug)]
    struct Item {
        key1: usize,
        key2: usize,
    };
    let mut list: Vec<Item> = (0..len)
        .map(|_| {
            key1 += 1;
            key1 %= 5;
            key2 += 1;
            Item { key1, key2 }
        })
        .collect();
    super::sort(&mut list, |a, b| -> NeverResult<_> { Ok(a.key1 > b.key1) }).unwrap_or_else(never);
    for pair in list.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        assert!(a.key1 <= b.key1);
        if a.key1 == b.key1 {
            assert!(a.key2 <= b.key2);
        }
    }
}

/// Insertion sort implementation convenience used for tests.
pub fn sort<T: Ord>(list: &mut [T]) {
    super::sort(list, |a, b| -> NeverResult<_> { Ok(a > b) }).unwrap_or_else(never);
}
