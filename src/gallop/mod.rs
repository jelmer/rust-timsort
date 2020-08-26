//! The galloping search algorithm.

#[cfg(test)]
mod tests;

use std::cmp::Ordering;

#[derive(Copy, Clone)]
pub enum Mode {
    Forward,
    Reverse,
}

/// Returns the index where key should be inserted, assuming it shoul be placed
/// at the beginning of any cluster of equal items.
pub fn gallop_left<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    key: &T,
    list: &[T],
    mode: Mode,
    is_greater: C,
) -> Result<usize, E> {
    let (mut base, mut lim) = gallop(key, list, mode, &is_greater)?;
    while lim != 0 {
        let ix = base + (lim / 2);
        match ordering(&is_greater, &list[ix], key)? {
            Ordering::Less => {
                base = ix + 1;
                lim -= 1;
            }
            Ordering::Greater => (),
            Ordering::Equal => {
                if ix == 0 || is_greater(key, &list[ix - 1])? {
                    base = ix;
                    break;
                }
            }
        };
        lim /= 2;
    }
    Ok(base)
}

/// Returns the index where key should be inserted, assuming it shoul be placed
/// at the end of any cluster of equal items.
pub fn gallop_right<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    key: &T,
    list: &[T],
    mode: Mode,
    is_greater: C,
) -> Result<usize, E> {
    let list_len = list.len();
    let (mut base, mut lim) = gallop(key, list, mode, &is_greater)?;
    while lim != 0 {
        let ix = base + (lim / 2);
        match ordering(&is_greater, &list[ix], key)? {
            Ordering::Less => {
                base = ix + 1;
                lim -= 1;
            }
            Ordering::Greater => (),
            Ordering::Equal => {
                base = ix + 1;
                if ix == list_len - 1 || is_greater(&list[ix + 1], key)? {
                    break;
                } else {
                    lim -= 1;
                }
            }
        };
        lim /= 2;
    }
    Ok(base)
}

fn gallop<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    key: &T,
    list: &[T],
    mode: Mode,
    is_greater: C,
) -> Result<(usize, usize), E> {
    let list_len = list.len();
    if list_len == 0 {
        return Ok((0, 0));
    }
    let ret = match mode {
        Mode::Forward => {
            let mut prev_val = 0;
            let mut next_val = 1;
            while next_val < list_len {
                match ordering(&is_greater, &list[next_val], key)? {
                    Ordering::Less => {
                        prev_val = next_val;
                        next_val = ((next_val + 1) * 2) - 1;
                    }
                    Ordering::Greater => {
                        break;
                    }
                    Ordering::Equal => {
                        next_val += 1;
                        break;
                    }
                }
            }
            if next_val > list_len {
                next_val = list_len;
            }
            (prev_val, next_val - prev_val)
        }
        Mode::Reverse => {
            let mut prev_val = list_len;
            let mut next_val = ((prev_val + 1) / 2) - 1;
            while is_greater(&list[next_val], key)? {
                prev_val = next_val + 1;
                next_val = (next_val + 1) / 2;
                if next_val != 0 {
                    next_val -= 1;
                } else {
                    break;
                }
            }
            (next_val, prev_val - next_val)
        }
    };
    Ok(ret)
}

fn ordering<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    is_greater: &C,
    a: &T,
    b: &T,
) -> Result<Ordering, E> {
    let ord = if is_greater(a, b)? {
        Ordering::Greater
    } else if is_greater(b, a)? {
        Ordering::Less
    } else {
        Ordering::Equal
    };
    Ok(ord)
}
