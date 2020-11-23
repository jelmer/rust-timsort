//! The merge algorithm. This one can merge unequal slices, allocating an n/2
//! sized temporary slice of the same type. Naturally, it can only merge slices
//! that are themselves already sorted.

#[cfg(test)]
mod tests;

use crate::gallop::{self, gallop_left, gallop_right};
use std::mem::ManuallyDrop;
use std::ptr;

/// Merge implementation switch.
pub fn merge<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    list: &mut [T],
    mut first_len: usize,
    is_greater: C,
) -> Result<(), E> {
    if first_len == 0 {
        return Ok(());
    }
    let (first, second) = list.split_at_mut(first_len);
    let second_len = gallop_left(
        first.last().unwrap(),
        second,
        gallop::Mode::Reverse,
        &is_greater,
    )?;
    let first_of_second = match second.get(0) {
        Some(x) => x,
        None => return Ok(()),
    };
    let first_off = gallop_right(first_of_second, first, gallop::Mode::Forward, &is_greater)?;
    first_len -= first_off;
    if first_len == 0 {
        return Ok(());
    }

    let nlist = &mut list[first_off..][..first_len + second_len];
    if first_len > second_len {
        merge_hi(nlist, first_len, second_len, is_greater)
    } else {
        merge_lo(nlist, first_len, is_greater)
    }
}

/// The number of times any one run can win before we try galloping.
/// Change this during testing.
const MIN_GALLOP: usize = 7;

/// Merge implementation used when the first run is smaller than the second.
pub fn merge_lo<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    list: &mut [T],
    first_len: usize,
    is_greater: C,
) -> Result<(), E> {
    MergeLo::new(list, first_len, is_greater).merge()
}

#[inline(always)]
fn md_as_inner<T>(x: &[ManuallyDrop<T>]) -> &[T] {
    // SAFETY: ManuallyDrop<T> is repr(transparent) over T
    unsafe { &*(x as *const [_] as *const [T]) }
}

/// Implementation of `merge_lo`. We need to have an object in order to
/// implement panic safety.
struct MergeLo<'a, T: 'a, E, C: Fn(&T, &T) -> Result<bool, E>> {
    list_len: usize,
    first_pos: usize,
    first_len: usize,
    second_pos: usize,
    dest_pos: usize,
    list: &'a mut [T],
    tmp: Vec<ManuallyDrop<T>>,
    is_greater: C,
}
impl<'a, T: 'a, E, C: Fn(&T, &T) -> Result<bool, E>> MergeLo<'a, T, E, C> {
    /// Constructor for a lower merge.
    fn new(list: &'a mut [T], first_len: usize, is_greater: C) -> Self {
        let mut ret_val = MergeLo {
            list_len: list.len(),
            first_pos: 0,
            first_len,
            second_pos: first_len,
            dest_pos: 0,
            list,
            tmp: Vec::with_capacity(first_len),
            is_greater,
        };
        // First, move the smallest run into temporary storage, leaving the
        // original contents uninitialized.
        unsafe {
            ret_val.tmp.set_len(first_len);
            ptr::copy_nonoverlapping(
                ret_val.list.as_ptr() as *const ManuallyDrop<T>,
                ret_val.tmp.as_mut_ptr(),
                first_len,
            );
        }
        ret_val
    }
    /// Perform the one-by-one comparison and insertion.
    fn merge(mut self) -> Result<(), E> {
        let is_greater = &self.is_greater;
        let mut first_count = 0;
        let mut second_count = 0;
        while self.second_pos > self.dest_pos && self.second_pos < self.list_len {
            debug_assert!(self.first_pos + (self.second_pos - self.first_len) == self.dest_pos);
            if (second_count | first_count) < MIN_GALLOP {
                // One-at-a-time mode.
                unsafe {
                    if is_greater(
                        self.tmp.get_unchecked(self.first_pos),
                        self.list.get_unchecked(self.second_pos),
                    )? {
                        ptr::copy_nonoverlapping(
                            self.list.get_unchecked(self.second_pos),
                            self.list.get_unchecked_mut(self.dest_pos),
                            1,
                        );
                        self.second_pos += 1;
                        second_count += 1;
                        first_count = 0;
                    } else {
                        ptr::copy_nonoverlapping(
                            &**self.tmp.get_unchecked(self.first_pos),
                            self.list.get_unchecked_mut(self.dest_pos),
                            1,
                        );
                        self.first_pos += 1;
                        first_count += 1;
                        second_count = 0;
                    }
                }
                self.dest_pos += 1;
            } else {
                // Galloping mode.
                second_count = gallop_left(
                    unsafe { md_as_inner(&self.tmp).get_unchecked(self.first_pos) },
                    &self.list[self.second_pos..],
                    gallop::Mode::Forward,
                    is_greater,
                )?;
                unsafe {
                    ptr::copy(
                        self.list.get_unchecked(self.second_pos),
                        self.list.get_unchecked_mut(self.dest_pos),
                        second_count,
                    )
                }
                self.dest_pos += second_count;
                self.second_pos += second_count;
                debug_assert!(self.first_pos + (self.second_pos - self.first_len) == self.dest_pos);
                if self.second_pos > self.dest_pos && self.second_pos < self.list_len {
                    first_count = gallop_right(
                        unsafe { self.list.get_unchecked(self.second_pos) },
                        md_as_inner(&self.tmp[self.first_pos..]),
                        gallop::Mode::Forward,
                        is_greater,
                    )?;
                    unsafe {
                        ptr::copy_nonoverlapping(
                            md_as_inner(&self.tmp).get_unchecked(self.first_pos),
                            self.list.get_unchecked_mut(self.dest_pos),
                            first_count,
                        )
                    };
                    self.dest_pos += first_count;
                    self.first_pos += first_count;
                }
            }
        }
        Ok(())
    }
}
impl<'a, T: 'a, E, C: Fn(&T, &T) -> Result<bool, E>> Drop for MergeLo<'a, T, E, C> {
    /// Copy all remaining items in the temporary storage into the list.
    /// If the comparator panics, the result will not be sorted, but will still
    /// contain no duplicates or uninitialized spots.
    fn drop(&mut self) {
        unsafe {
            // Make sure that the entire tmp storage is consumed. Since there are no uninitialized
            // spaces before dest_pos, and no uninitialized space after first_pos, this will ensure
            // that there are no uninitialized spaces inside the slice after we drop. Thus, the
            // function is safe.
            if self.first_pos < self.first_len {
                ptr::copy_nonoverlapping(
                    self.tmp.get_unchecked(self.first_pos) as *const _ as *const T,
                    self.list.get_unchecked_mut(self.dest_pos),
                    self.first_len - self.first_pos,
                );
            }
            // The temporary storage is now full of nothing but uninitialized.
            // We want to deallocate the space, but not call the destructors.
            self.tmp.set_len(0);
        }
    }
}

/// Merge implementation used when the first run is larger than the second.
pub fn merge_hi<T, E, C: Fn(&T, &T) -> Result<bool, E>>(
    list: &mut [T],
    first_len: usize,
    second_len: usize,
    is_greater: C,
) -> Result<(), E> {
    MergeHi::new(list, first_len, second_len, is_greater).merge()
}

/// Implementation of `merge_hi`. We need to have an object in order to
/// implement panic safety.
struct MergeHi<'a, T: 'a, E, C: Fn(&T, &T) -> Result<bool, E>> {
    first_pos: isize,
    second_pos: isize,
    dest_pos: isize,
    list: &'a mut [T],
    tmp: Vec<ManuallyDrop<T>>,
    is_greater: C,
}

impl<'a, T: 'a, E, C: Fn(&T, &T) -> Result<bool, E>> MergeHi<'a, T, E, C> {
    /// Constructor for a higher merge.
    fn new(list: &'a mut [T], first_len: usize, second_len: usize, is_greater: C) -> Self {
        let mut ret_val = MergeHi {
            first_pos: first_len as isize - 1,
            second_pos: second_len as isize - 1,
            dest_pos: list.len() as isize - 1,
            list,
            tmp: Vec::with_capacity(second_len),
            is_greater,
        };
        // First, move the smallest run into temporary storage, leaving the
        // original contents uninitialized.
        unsafe {
            ret_val.tmp.set_len(second_len);
            ptr::copy_nonoverlapping(
                ret_val.list.as_ptr().add(first_len),
                ret_val.tmp.as_mut_ptr() as *mut T,
                second_len,
            );
        }
        ret_val
    }
    /// Perform the one-by-one comparison and insertion.
    fn merge(mut self) -> Result<(), E> {
        let is_greater = &self.is_greater;
        let mut first_count: usize = 0;
        let mut second_count: usize = 0;
        while self.first_pos < self.dest_pos && self.first_pos >= 0 {
            debug_assert!(self.first_pos + self.second_pos + 1 == self.dest_pos);
            if (second_count | first_count) < MIN_GALLOP {
                // One-at-a-time mode.
                unsafe {
                    if is_greater(
                        self.list.get_unchecked(self.first_pos as usize),
                        self.tmp.get_unchecked(self.second_pos as usize),
                    )? {
                        ptr::copy_nonoverlapping(
                            self.list.get_unchecked(self.first_pos as usize),
                            self.list.get_unchecked_mut(self.dest_pos as usize),
                            1,
                        );
                        self.first_pos -= 1;
                    } else {
                        ptr::copy_nonoverlapping(
                            md_as_inner(&self.tmp).get_unchecked(self.second_pos as usize),
                            self.list.get_unchecked_mut(self.dest_pos as usize),
                            1,
                        );
                        self.second_pos -= 1;
                    }
                }
                self.dest_pos -= 1;
            } else {
                // Galloping mode.
                first_count = self.first_pos as usize + 1
                    - gallop_right(
                        unsafe { md_as_inner(&self.tmp).get_unchecked(self.second_pos as usize) },
                        &self.list[..=self.first_pos as usize],
                        gallop::Mode::Reverse,
                        is_greater,
                    )?;
                unsafe {
                    copy_backwards(
                        self.list.get_unchecked(self.first_pos as usize),
                        self.list.get_unchecked_mut(self.dest_pos as usize),
                        first_count,
                    )
                }
                self.dest_pos -= first_count as isize;
                self.first_pos -= first_count as isize;
                debug_assert!(self.first_pos + self.second_pos + 1 == self.dest_pos);
                if self.first_pos < self.dest_pos && self.first_pos >= 0 {
                    second_count = self.second_pos as usize + 1
                        - gallop_left(
                            unsafe { self.list.get_unchecked(self.first_pos as usize) },
                            md_as_inner(&self.tmp[..=self.second_pos as usize]),
                            gallop::Mode::Reverse,
                            is_greater,
                        )?;
                    unsafe {
                        copy_nonoverlapping_backwards(
                            md_as_inner(&self.tmp).get_unchecked(self.second_pos as usize),
                            self.list.get_unchecked_mut(self.dest_pos as usize),
                            second_count,
                        )
                    }
                    self.dest_pos -= second_count as isize;
                    self.second_pos -= second_count as isize;
                }
            }
        }
        Ok(())
    }
}

/// Perform a backwards `ptr::copy_nonoverlapping`. Behave identically when size = 1, but behave
/// differently all other times
unsafe fn copy_backwards<T>(src: *const T, dest: *mut T, size: usize) {
    ptr::copy(
        src.sub(size.wrapping_sub(1)),
        dest.sub(size.wrapping_sub(1)),
        size,
    )
}

/// Perform a backwards `ptr::copy_nonoverlapping`. Behave identically when size = 1, but behave
/// differently all other times
unsafe fn copy_nonoverlapping_backwards<T>(src: *const T, dest: *mut T, size: usize) {
    ptr::copy_nonoverlapping(
        src.sub(size.wrapping_sub(1)),
        dest.sub(size.wrapping_sub(1)),
        size,
    )
}

impl<'a, T: 'a, E, C: Fn(&T, &T) -> Result<bool, E>> Drop for MergeHi<'a, T, E, C> {
    /// Copy all remaining items in the temporary storage into the list.
    /// If the comparator panics, the result will not be sorted, but will still
    /// contain no duplicates or uninitialized spots.
    fn drop(&mut self) {
        unsafe {
            // Make sure that the entire tmp storage is consumed. Since there are no uninitialized
            // spaces before dest_pos, and no uninitialized space after first_pos, this will ensure
            // that there are no uninitialized spaces inside the slice after we drop. Thus, the
            // function is safe.
            if self.second_pos >= 0 {
                copy_nonoverlapping_backwards(
                    md_as_inner(&self.tmp).get_unchecked(self.second_pos as usize),
                    self.list.get_unchecked_mut(self.dest_pos as usize),
                    self.second_pos as usize + 1,
                );
            }
        }
    }
}
