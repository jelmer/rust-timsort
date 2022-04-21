//! The run finder algorithm. Takes an unsorted slice, and returns the number
//! of sequential elements in a row.

#[cfg(test)]
mod tests;

use crate::Comparator;

/// Find a run, reversing if necessary.
pub(crate) fn get_run<T, C: Comparator<T>>(list: &mut [T], cmp: &C) -> Result<usize, C::Error> {
    let (ord, len) = find_run(list, cmp)?;
    if ord {
        list[..len].reverse();
    }
    Ok(len)
}

/// Find a run. Returns true if it needs reversed, and false otherwise.
pub(crate) fn find_run<T, C: Comparator<T>>(
    list: &[T],
    cmp: &C,
) -> Result<(bool, usize), C::Error> {
    let (first, second) = match list {
        [a, b, ..] => (a, b),
        _ => return Ok((false, list.len())),
    };
    let mut pos = 1;
    let gt = if cmp.is_gt(first, second)? {
        for pair in list[1..].windows(2) {
            if !cmp.is_gt(&pair[0], &pair[1])? {
                break;
            }
            pos += 1;
        }
        true
    } else {
        for pair in list[1..].windows(2) {
            if cmp.is_gt(&pair[0], &pair[1])? {
                break;
            }
            pos += 1;
        }
        false
    };
    Ok((gt, pos + 1))
}
