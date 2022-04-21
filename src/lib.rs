//! This crate is a stable sorting algorithm with O(n) worst-case storage
//! requirements, O(n log n) worst-case comparisons, and O(n) comparisons
//! on an already-sorted list, smoothly becoming O(n log n) as the sorted
//! sections (runs) get smaller and smaller.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod find_run;
mod gallop;
mod insort;
mod merge;
mod sort;

use core::cmp::Ordering;
use core::convert::Infallible;
use sort::try_sort_by as try_sort_by_cmp;

type NeverResult<T> = Result<T, Infallible>;
#[inline(always)]
fn never<T>(x: Infallible) -> T {
    match x {}
}

pub fn try_sort_by_gt<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    list: &mut [T],
    cmp: C,
) -> Result<(), E> {
    try_sort_by_cmp(list, cmp)
}

#[inline]
pub fn try_sort_by<T, E, C: Fn(&T, &T) -> Result<Ordering, E>>(
    list: &mut [T],
    cmp: C,
) -> Result<(), E> {
    try_sort_by_cmp(list, ord_comparator(cmp))
}

#[inline]
pub fn sort_by_gt<T, C: Fn(&T, &T) -> bool>(list: &mut [T], is_greater: C) {
    try_sort_by_gt(list, move |a, b| -> NeverResult<_> { Ok(is_greater(a, b)) })
        .unwrap_or_else(never)
}

#[inline]
pub fn sort_by<T, C: Fn(&T, &T) -> Ordering>(list: &mut [T], cmp: C) {
    try_sort_by(list, move |a, b| -> NeverResult<_> { Ok(cmp(a, b)) }).unwrap_or_else(never)
}

#[inline]
pub fn sort<T: Ord>(list: &mut [T]) {
    sort_by(list, Ord::cmp)
}

trait Comparator<T> {
    type Error;
    fn is_gt(&self, lhs: &T, rhs: &T) -> Result<bool, Self::Error>;
    fn ordering(&self, lhs: &T, rhs: &T) -> Result<Ordering, Self::Error> {
        let ord = if self.is_gt(lhs, rhs)? {
            Ordering::Greater
        } else if self.is_gt(rhs, lhs)? {
            Ordering::Less
        } else {
            Ordering::Equal
        };
        Ok(ord)
    }
}

impl<F, T, E> Comparator<T> for F
where
    F: Fn(&T, &T) -> Result<bool, E>,
{
    type Error = E;
    fn is_gt(&self, lhs: &T, rhs: &T) -> Result<bool, E> {
        self(lhs, rhs)
    }
}

// really weird, idk why this is necessary...
#[cfg(test)]
pub(crate) fn comparator<T>(
    f: impl Fn(&T, &T) -> NeverResult<bool>,
) -> impl Comparator<T, Error = Infallible> {
    f
}
#[cfg(test)]
pub(crate) fn ord_t_comparator<T: Ord>() -> impl Comparator<T, Error = Infallible> {
    ord_comparator(|a: &T, b| Ok(a.cmp(b)))
}

pub(crate) fn ord_comparator<T, E, F: Fn(&T, &T) -> Result<Ordering, E>>(
    f: F,
) -> impl Comparator<T, Error = E> {
    struct OrdComparator<F>(F);
    impl<T, E, F: Fn(&T, &T) -> Result<Ordering, E>> Comparator<T> for OrdComparator<F> {
        type Error = E;
        fn is_gt(&self, lhs: &T, rhs: &T) -> Result<bool, E> {
            (self.0)(lhs, rhs).map(|ord| ord == Ordering::Greater)
        }
        fn ordering(&self, lhs: &T, rhs: &T) -> Result<Ordering, Self::Error> {
            (self.0)(lhs, rhs)
        }
    }
    OrdComparator(f)
}
