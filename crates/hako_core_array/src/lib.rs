//! hako_core_array — shared helpers for Array semantics

/// Return length as i64
pub fn length(len: usize) -> i64 {
    len as i64
}

/// Compute clamped slice bounds [start,end) for given length (element count)
/// Policy (Phase 15.7):
/// - start < 0 => 0; start > len => len
/// - end < 0 => len (treat negative end as "till end")
/// - end > len => len
/// - if start > end => empty range [end,end]
pub fn slice_bounds(len: usize, start: i64, end: i64) -> (usize, usize) {
    let l = len as i64;
    let mut i0 = start.max(0).min(l) as usize;
    let i1 = if end < 0 {
        len
    } else {
        end.max(0).min(l) as usize
    };
    if i0 > i1 {
        i0 = i1;
    }
    (i0, i1)
}

/// Safe index for get(): returns Some(index) if 0 <= idx < len, otherwise None.
pub fn safe_get_index(len: usize, idx: i64) -> Option<usize> {
    if idx < 0 {
        return None;
    }
    let u = idx as usize;
    if u < len {
        Some(u)
    } else {
        None
    }
}

/// Classification for set(index, value) semantics.
/// - Replace(i): 0 <= idx < len
/// - Append:     idx == len
/// - Oob:        otherwise
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SetIndex {
    Replace(usize),
    Append,
    Oob,
}

/// Classify index for set() according to core array semantics.
pub fn classify_set_index(len: usize, idx: i64) -> SetIndex {
    if idx < 0 {
        return SetIndex::Oob;
    }
    let u = idx as usize;
    if u < len {
        SetIndex::Replace(u)
    } else if u == len {
        SetIndex::Append
    } else {
        SetIndex::Oob
    }
}
