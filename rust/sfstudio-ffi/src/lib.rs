//! C ABI for calling SFStudio's shared Rust search and code generation from other languages.

use std::panic::{catch_unwind, AssertUnwindSafe};

const UNKNOWN_SYMBOL: isize = -6;
const INVALID_MODE: isize = -7;
const INVALID_FAVORITES_JSON: isize = -8;
const INVALID_QUERY: isize = -1;
const INVALID_UTF8: isize = -2;
const INVALID_OUTPUT: isize = -3;
const INTERNAL_ERROR: isize = -4;
const OUTPUT_TOO_LARGE: isize = -5;

unsafe fn read_utf8<'a>(input: *const u8, input_len: usize) -> Result<&'a str, isize> {
    if input_len == 0 {
        return Ok("");
    }
    if input.is_null() {
        return Err(INVALID_QUERY);
    }
    // SAFETY: The calling C ABI function requires its caller to provide `input_len` readable bytes.
    let bytes = unsafe { std::slice::from_raw_parts(input, input_len) };
    std::str::from_utf8(bytes).map_err(|_| INVALID_UTF8)
}

unsafe fn write_bytes(output: *mut u8, output_capacity: usize, bytes: &[u8]) -> isize {
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }
    let required = match isize::try_from(bytes.len()) {
        Ok(required) => required,
        Err(_) => return OUTPUT_TOO_LARGE,
    };
    if output_capacity == 0 || output_capacity < bytes.len() {
        return required;
    }
    // SAFETY: The calling C ABI function requires `output_capacity` writable bytes,
    // and this branch proves the copy fits.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len()) };
    required
}

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
    let query_text = match unsafe { read_utf8(query, query_len) } {
        Ok(text) => text,
        Err(code) => return code,
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
        serde_json::to_vec(&names).map_err(|_| INTERNAL_ERROR)
    })) {
        Ok(Ok(json)) => unsafe { write_bytes(output, output_capacity, &json) },
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

/// Generate the Kotlin Compose snippet for a catalog symbol.
///
/// `mode` is 0 for dualtone and 1 for monochrome. `tint_enabled` and
/// `background_enabled` use zero for false and any nonzero value for true.
/// The return value is the required/written UTF-8 byte count, or a negative
/// error: -1 invalid symbol pointer, -2 invalid UTF-8, -3 invalid output pointer,
/// -4 internal panic, -5 result too large, -6 unknown symbol, -7 invalid mode.
/// Use `output = NULL` and `output_capacity = 0` to query the required size.
///
/// # Safety
/// For nonzero `symbol_len`, `symbol` must point to that many readable bytes. If
/// `output_capacity` is nonzero, `output` must point to that many writable
/// bytes. The buffers may overlap because the function creates the full result
/// before writing it.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_compose_snippet(
    symbol_name: *const u8,
    symbol_len: usize,
    mode: i32,
    tint_argb: i32,
    tint_enabled: i32,
    background_argb: i32,
    background_enabled: i32,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let symbol_name = match unsafe { read_utf8(symbol_name, symbol_len) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }
    let mode = match mode {
        0 => sfstudio_core::codegen::SymbolMode::Dualtone,
        1 => sfstudio_core::codegen::SymbolMode::Monochrome,
        _ => return INVALID_MODE,
    };

    match catch_unwind(AssertUnwindSafe(|| {
        let symbol = sfstudio_core::catalog()
            .find(symbol_name)
            .ok_or(UNKNOWN_SYMBOL)?;
        Ok(sfstudio_core::codegen::compose_snippet(
            symbol,
            mode,
            tint_argb,
            tint_enabled != 0,
            (background_enabled != 0).then_some(background_argb),
        )
        .into_bytes())
    })) {
        Ok(Ok(snippet)) => unsafe { write_bytes(output, output_capacity, &snippet) },
        Ok(Err(code)) => code,
        Err(_) => INTERNAL_ERROR,
    }
}

/// Toggle a favorite in a JSON array while preserving its order.
///
/// The return value is the required/written UTF-8 byte count, or a negative
/// error: -1 invalid input pointer, -2 invalid UTF-8, -3 invalid output pointer,
/// -4 internal panic/serialization failure, -5 result too large, -8 invalid
/// favorites JSON. Use `output = NULL` and `output_capacity = 0` to query size.
///
/// # Safety
/// Each nonzero input length must describe readable bytes. If `output_capacity`
/// is nonzero, `output` must point to that many writable bytes. The buffers may
/// overlap because the function owns its inputs' result before writing output.
#[no_mangle]
pub unsafe extern "C" fn sfstudio_favorites_toggle_json(
    favorites_json: *const u8,
    favorites_json_len: usize,
    symbol_name: *const u8,
    symbol_name_len: usize,
    output: *mut u8,
    output_capacity: usize,
) -> isize {
    let favorites_json = match unsafe { read_utf8(favorites_json, favorites_json_len) } {
        Ok(json) => json,
        Err(code) => return code,
    };
    let symbol_name = match unsafe { read_utf8(symbol_name, symbol_name_len) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    if output_capacity > 0 && output.is_null() {
        return INVALID_OUTPUT;
    }

    match catch_unwind(AssertUnwindSafe(|| {
        let mut favorites = sfstudio_core::favorites::Favorites::from_json(favorites_json)
            .map_err(|_| INVALID_FAVORITES_JSON)?;
        favorites.toggle(symbol_name);
        favorites
            .to_json()
            .map(String::into_bytes)
            .map_err(|_| INTERNAL_ERROR)
    })) {
        Ok(Ok(json)) => unsafe { write_bytes(output, output_capacity, &json) },
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

    fn compose_snippet(
        mode: i32,
        tint_enabled: bool,
        background: Option<i32>,
    ) -> Result<String, isize> {
        let name = b"heart.fill";
        // SAFETY: Input and output slices remain alive and satisfy their lengths.
        unsafe {
            let background_argb = background.unwrap_or_default();
            let background_enabled = i32::from(background.is_some());
            let required = sfstudio_compose_snippet(
                name.as_ptr(),
                name.len(),
                mode,
                0x80e0_245e_u32 as i32,
                i32::from(tint_enabled),
                background_argb,
                background_enabled,
                std::ptr::null_mut(),
                0,
            );
            if required < 0 {
                return Err(required);
            }
            let mut output = vec![0; required as usize];
            let written = sfstudio_compose_snippet(
                name.as_ptr(),
                name.len(),
                mode,
                0x80e0_245e_u32 as i32,
                i32::from(tint_enabled),
                background_argb,
                background_enabled,
                output.as_mut_ptr(),
                output.len(),
            );
            if written != required {
                return Err(written);
            }
            Ok(String::from_utf8(output).expect("snippet is UTF-8"))
        }
    }

    #[test]
    fn returns_the_existing_plain_and_background_snippets() {
        assert_eq!(
            compose_snippet(1, true, Some(0xffdc_e9d3_u32 as i32)).unwrap(),
            include_str!(
                "../../../app/src/jvmTest/resources/compose-snippet-monochrome-background.kt.txt"
            )
            .replace("\r\n", "\n")
        );
        assert_eq!(
            compose_snippet(0, false, None).unwrap(),
            include_str!(
                "../../../app/src/jvmTest/resources/compose-snippet-dualtone-plain.kt.txt"
            )
            .replace("\r\n", "\n")
        );
    }

    #[test]
    fn reports_unknown_symbols_and_invalid_modes() {
        // SAFETY: The input is a valid static byte slice and output is a size query.
        unsafe {
            assert_eq!(
                sfstudio_compose_snippet(
                    b"missing".as_ptr(),
                    7,
                    0,
                    0,
                    0,
                    0,
                    0,
                    std::ptr::null_mut(),
                    0
                ),
                UNKNOWN_SYMBOL
            );
            assert_eq!(
                sfstudio_compose_snippet(
                    b"heart.fill".as_ptr(),
                    10,
                    2,
                    0,
                    0,
                    0,
                    0,
                    std::ptr::null_mut(),
                    0
                ),
                INVALID_MODE
            );
        }
    }

    #[test]
    fn toggles_favorites_without_changing_order_or_json_contract() {
        let input = br#"["heart.fill","star"]"#;
        let name = b"gear";
        // SAFETY: Inputs and output satisfy their declared lengths.
        unsafe {
            let required = sfstudio_favorites_toggle_json(
                input.as_ptr(),
                input.len(),
                name.as_ptr(),
                name.len(),
                std::ptr::null_mut(),
                0,
            );
            assert!(required > 0);
            let mut output = vec![0; required as usize];
            assert_eq!(
                sfstudio_favorites_toggle_json(
                    input.as_ptr(),
                    input.len(),
                    name.as_ptr(),
                    name.len(),
                    output.as_mut_ptr(),
                    output.len(),
                ),
                required
            );
            assert_eq!(
                String::from_utf8(output).unwrap(),
                "[\n  \"heart.fill\",\n  \"star\",\n  \"gear\"\n]"
            );
        }
    }

    #[test]
    fn rejects_malformed_favorites_json() {
        // SAFETY: Both inputs are valid static byte slices; output is a size query.
        unsafe {
            assert_eq!(
                sfstudio_favorites_toggle_json(
                    b"{broken".as_ptr(),
                    7,
                    b"gear".as_ptr(),
                    4,
                    std::ptr::null_mut(),
                    0,
                ),
                INVALID_FAVORITES_JSON
            );
        }
    }
}
