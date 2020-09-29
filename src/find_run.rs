//! The run finder algorithm. Takes an unsorted slice, and returns the number
//! of sequential elements in a row.

#[cfg(test)]
mod tests;

/// Find a run, reversing if necessary.
pub fn get_run<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    list: &mut [T],
    is_greater: C,
) -> Result<usize, E> {
    let (ord, len) = find_run(list, is_greater)?;
    if ord {
        list[..len].reverse();
    }
    Ok(len)
}

/// Find a run. Returns true if it needs reversed, and false otherwise.
pub fn find_run<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    list: &[T],
    is_greater: C,
) -> Result<(bool, usize), E> {
    let (first, second) = match list {
        [a, b, ..] => (a, b),
        _ => return Ok((false, list.len())),
    };
    let mut pos = 1;
    let gt = if is_greater(first, second)? {
        for pair in list[1..].windows(2) {
            if !is_greater(&pair[0], &pair[1])? {
                break;
            }
            pos += 1;
        }
        true
    } else {
        for pair in list[1..].windows(2) {
            if is_greater(&pair[0], &pair[1])? {
                break;
            }
            pos += 1;
        }
        false
    };
    Ok((gt, pos + 1))
}
