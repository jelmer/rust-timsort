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
        list.split_at_mut(len).0.reverse();
    }
    Ok(len)
}

/// Find a run. Returns true if it needs reversed, and false otherwise.
pub fn find_run<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    list: &[T],
    is_greater: C,
) -> Result<(bool, usize), E> {
    let list_len = list.len();
    if list_len < 2 {
        return Ok((false, list_len));
    }
    let mut pos = 1;
    let ret = if is_greater(&list[0], &list[1])? {
        while pos < list_len - 1 && is_greater(&list[pos], &list[pos + 1])? {
            pos += 1;
        }
        (true, pos + 1)
    } else {
        while pos < list_len - 1 && !is_greater(&list[pos], &list[pos + 1])? {
            pos += 1;
        }
        (false, pos + 1)
    };
    Ok(ret)
}
