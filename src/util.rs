//! UTF-16 conversion helpers.

use crate::error::{Error, Result};

/// Convert `&str` to a null-terminated UTF-16 buffer.
pub(crate) fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(core::iter::once(0)).collect()
}

/// Convert an optional `&str` into an owned wide-buffer + a `PCWSTR`-like ptr
/// (null when input is `None`). Returned buffer must outlive the pointer.
pub(crate) fn opt_to_wide(s: Option<&str>) -> Option<Vec<u16>> {
    s.map(to_wide)
}

/// Read a null-terminated wide string from a raw pointer into an owned `String`.
///
/// # Safety
/// `ptr` must be null OR point to a valid null-terminated UTF-16 sequence that
/// lives long enough for the read.
pub(crate) unsafe fn wide_ptr_to_string(ptr: *const u16) -> Result<String> {
    if ptr.is_null() {
        return Ok(String::new());
    }
    let mut len = 0usize;
    // SAFETY: caller guarantees null-terminated valid UTF-16.
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
    String::from_utf16(slice).map_err(|_| Error::InvalidUtf16)
}
