//! This crate is a stable sorting algorithm with O(n) worst-case storage
//! requirements, O(n log n) worst-case comparisons, and O(n) comparisons
//! on an already-sorted list, smoothly becoming O(n log n) as the sorted
//! sections (runs) get smaller and smaller.

mod find_run;
mod gallop;
mod insort;
mod merge;
mod sort;

pub use sort::try_sort_by as try_sort_by_gt;
use std::cmp::Ordering;
use std::convert::Infallible;

type NeverResult<T> = Result<T, Infallible>;
#[inline(always)]
fn never<T>(x: Infallible) -> T {
    match x {}
}

#[inline]
pub fn try_sort_by<T, E, C: Fn(&T, &T) -> Result<Ordering, E>>(
    list: &mut [T],
    cmp: C,
) -> Result<(), E> {
    try_sort_by_gt(list, move |a, b| {
        cmp(a, b).map(|ord| ord == Ordering::Greater)
    })
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
pub fn sort<T: PartialOrd>(list: &mut [T]) {
    sort_by_gt(list, |a, b| a > b)
}

trait Comparator<T> {
    type Error;
    fn is_gt(&self, lhs: &T, rhs: &T) -> Result<bool, Self::Error>;
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
