//! C ABI for calling SFStudio's shared Rust catalog search from other languages.

use std::panic::{catch_unwind, AssertUnwindSafe};

const INVALID_QUERY: isize = -1;
const INVALID_UTF8: isize = -2;
const INVALID_OUTPUT: isize = -3;
const INTERNAL_ERROR: isize = -4;
const OUTPUT_TOO_LARGE: isize = -5;

/// Search the SF Symbols catalog and write a JSON array of Apple symbol names.
///
/// The return value is the number of JSON bytes required or written. Call once
/// with `output = NULL` and `output_capacity = 0` to size the buffer, then call
/// again with that many writable bytes. A negative return value indicates an
/// error: -1 invalid query pointer, -2 invalid UTF-8, -3 invalid output pointer,
/// -4 internal panic/serialization failure, -5 result length exceeds `isize`.
///
/// # Safety
/// For nonzero `query_len`, `query` must point to that many readable bytes. If
/// `output_capacity` is nonzero, `output` must point to that many writable
/// bytes. The buffers may overlap; the function finishes reading the query
/// before writing output.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_search_json(
    query: *const u8,
    query_len: usize,
    limit: usize,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let query_bytes: &[u8] = if query_len == 0 {
        &[]
    } else {
        if query.is_null() {
            return INVALID_QUERY;
        }
        // SAFETY: The caller guarantees `query` points to `query_len` readable bytes.
        unsafe { std::slice::from_raw_parts(query, query_len) }
    };
    let query_text = match std::str::from_utf8(query_bytes) {
        Ok(text) => text,
        Err(_) => return INVALID_UTF8,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }

    match catch_unwind(AssertUnwindSafe(|| {
        let names: Vec<_> = sfstudio_core::catalog()
            .search(query_text, limit)
            .iter()
            .map(|symbol| symbol.apple_name.as_str())
            .collect();
        let json = serde_json::to_vec(&names).map_err(|_| INTERNAL_ERROR)?;
        let required = isize::try_from(json.len()).map_err(|_| OUTPUT_TOO_LARGE)?;

        if output_capacity == 0 || output_capacity < json.len() {
            return Ok(required);
        }

        // SAFETY: The caller guarantees `output` points to `output_capacity`
        // writable bytes, and the size check above proves the copy fits.
        unsafe { std::ptr::copy_nonoverlapping(json.as_ptr(), output, json.len()) };
        Ok(required)
    })) {
        Ok(Ok(required)) => required,
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn search(query: &[u8], limit: usize) -> Result<String, isize> {
        // SAFETY: Both slices remain alive for the call and output capacity is exact.
        unsafe {
            let required =
                sfstudio_search_json(query.as_ptr(), query.len(), limit, std::ptr::null_mut(), 0);
            if required < 0 {
                return Err(required);
            }
            let mut output = vec![0; required as usize];
            let written = sfstudio_search_json(
                query.as_ptr(),
                query.len(),
                limit,
                output.as_mut_ptr(),
                output.len(),
            );
            if written != required {
                return Err(written);
            }
            Ok(String::from_utf8(output).expect("JSON is UTF-8"))
        }
    }

    #[test]
    fn returns_ranked_names_and_supports_sizing_calls() {
        assert_eq!(search(b"hosue", 1).unwrap(), r#"["house"]"#);
        assert_eq!(
            search(b"", 3).unwrap(),
            r#"["0.circle.fill","0.circle","0.square.fill"]"#
        );
        assert_eq!(search("🐈".as_bytes(), 5).unwrap(), "[]");
    }

    #[test]
    fn reports_bad_inputs_and_never_partially_writes_short_buffers() {
        // SAFETY: The non-null test buffers are valid for their declared lengths.
        unsafe {
            assert_eq!(
                sfstudio_search_json(std::ptr::null(), 1, 1, std::ptr::null_mut(), 0),
                INVALID_QUERY
            );
            assert_eq!(
                sfstudio_search_json([0xff].as_ptr(), 1, 1, std::ptr::null_mut(), 0),
                INVALID_UTF8
            );
            assert_eq!(
                sfstudio_search_json(b"a".as_ptr(), 1, 1, std::ptr::null_mut(), 1),
                INVALID_OUTPUT
            );
            let mut short = [0xa5; 1];
            let required = sfstudio_search_json(b"home".as_ptr(), 4, 1, short.as_mut_ptr(), 1);
            assert!(required > 1);
            assert_eq!(short, [0xa5]);
        }
    }
}
