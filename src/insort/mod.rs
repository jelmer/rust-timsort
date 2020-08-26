//! The bottom sorting algorithm (we could just have 1-element runs and do all
//! the sorting with the merge algorithm, but that would be much slower).

#[cfg(test)]
mod tests;

/// Sorts the list using insertion sort.
///
/// `c(a, b)` should return std::cmp::Ordering::Greater when `a` is greater than `b`.
// This version was almost completely copied from libcollections/slice.rs
pub fn sort<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    list: &mut [T],
    is_greater: C,
) -> Result<(), E> {
    if list.len() < 2 {
        return Ok(());
    }
    for i in 0..list.len() {
        let i_el = &list[i];
        // find the index just above the element that is in order wrt list[i]
        let mut j = 0;
        for (jj, j_el) in list[..i].iter().enumerate().rev() {
            if !is_greater(j_el, i_el)? {
                j = jj + 1;
                break;
            }
        }
        if i != j {
            // SAFETY: j<i, i<list.len
            unsafe { list.get_unchecked_mut(j..=i).rotate_right(1) };
        }
    }
    Ok(())
}
