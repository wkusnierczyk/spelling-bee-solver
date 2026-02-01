//! FFI bindings for Spelling Bee Solver.
//!
//! Provides a C-compatible interface for loading dictionaries and solving puzzles.
//! Dictionary is managed as an opaque pointer (Box/unbox pattern). No global state.

use sbs::{Config, Dictionary, Solver};
use std::ffi::{c_char, CStr, CString};

/// Static version string.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Load a dictionary from the given file path.
///
/// Returns an opaque pointer to the Dictionary, or null on failure.
/// The caller must free it with `sbs_free_dictionary`.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn sbs_load_dictionary(path: *const c_char) -> *mut Dictionary {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match Dictionary::from_file(path_str) {
        Ok(dict) => Box::into_raw(Box::new(dict)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a Dictionary previously returned by `sbs_load_dictionary`.
///
/// # Safety
/// `ptr` must be a pointer returned by `sbs_load_dictionary`, or null.
/// Must not be called more than once for the same pointer.
#[no_mangle]
pub unsafe extern "C" fn sbs_free_dictionary(ptr: *mut Dictionary) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

/// Solve a puzzle given a dictionary and a JSON request string.
///
/// The request JSON should have the shape: `{"letters": "abc", "present": "a"}`.
/// Returns a JSON string: `{"words": [...]}` on success, or `{"error": "..."}` on failure.
/// The caller must free the returned string with `sbs_free_string`.
///
/// # Safety
/// - `dict` must be a valid pointer returned by `sbs_load_dictionary`.
/// - `request_json` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn sbs_solve(
    dict: *const Dictionary,
    request_json: *const c_char,
) -> *mut c_char {
    if dict.is_null() || request_json.is_null() {
        return to_json_error("null pointer argument");
    }

    let dict = unsafe { &*dict };
    let c_str = unsafe { CStr::from_ptr(request_json) };
    let json_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return to_json_error("invalid UTF-8 in request"),
    };

    let config: Config = match serde_json::from_str(json_str) {
        Ok(c) => c,
        Err(e) => return to_json_error(&format!("invalid JSON: {e}")),
    };

    let solver = Solver::new(config);
    match solver.solve(dict) {
        Ok(words) => {
            let mut sorted: Vec<String> = words.into_iter().collect();
            sorted.sort();
            let result = serde_json::json!({ "words": sorted });
            to_c_string(&result.to_string())
        }
        Err(e) => to_json_error(&e.to_string()),
    }
}

/// Free a string previously returned by `sbs_solve` or `sbs_version`.
///
/// # Safety
/// `s` must be a pointer returned by `sbs_solve`, or null.
/// Must not be called more than once for the same pointer.
#[no_mangle]
pub unsafe extern "C" fn sbs_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

/// Return the library version as a static string.
///
/// The returned pointer is valid for the lifetime of the library and must NOT be freed.
#[no_mangle]
pub extern "C" fn sbs_version() -> *const c_char {
    // Leak a CString once; it's a static value.
    static VERSION_CSTR: std::sync::OnceLock<CString> = std::sync::OnceLock::new();
    VERSION_CSTR
        .get_or_init(|| CString::new(VERSION).expect("version contains no nulls"))
        .as_ptr()
}

fn to_json_error(msg: &str) -> *mut c_char {
    let result = serde_json::json!({ "error": msg });
    to_c_string(&result.to_string())
}

fn to_c_string(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::io::Write;

    #[test]
    fn test_version() {
        let ptr = sbs_version();
        assert!(!ptr.is_null());
        let version = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap();
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_load_dictionary_null() {
        let ptr = unsafe { sbs_load_dictionary(std::ptr::null()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn test_load_dictionary_invalid_path() {
        let path = CString::new("/nonexistent/path.txt").unwrap();
        let ptr = unsafe { sbs_load_dictionary(path.as_ptr()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn test_solve_null_args() {
        let result = unsafe { sbs_solve(std::ptr::null(), std::ptr::null()) };
        assert!(!result.is_null());
        let s = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        assert!(s.contains("error"));
        unsafe { sbs_free_string(result) };
    }

    #[test]
    fn test_roundtrip() {
        // Create a temp dictionary file
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "apple").unwrap();
        writeln!(tmp, "appeal").unwrap();
        writeln!(tmp, "peal").unwrap();
        writeln!(tmp, "pale").unwrap();
        writeln!(tmp, "leap").unwrap();
        writeln!(tmp, "plea").unwrap();
        writeln!(tmp, "ape").unwrap();
        writeln!(tmp, "ale").unwrap();
        tmp.flush().unwrap();

        let path = CString::new(tmp.path().to_str().unwrap()).unwrap();
        let dict = unsafe { sbs_load_dictionary(path.as_ptr()) };
        assert!(!dict.is_null());

        let request = CString::new(r#"{"letters":"aple","present":"a"}"#).unwrap();
        let result = unsafe { sbs_solve(dict, request.as_ptr()) };
        assert!(!result.is_null());

        let s = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert!(parsed.get("words").is_some());
        let words = parsed["words"].as_array().unwrap();
        assert!(!words.is_empty());

        unsafe {
            sbs_free_string(result);
            sbs_free_dictionary(dict);
        }
    }

    #[test]
    fn test_solve_invalid_json() {
        // Create a minimal dictionary
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "test").unwrap();
        tmp.flush().unwrap();

        let path = CString::new(tmp.path().to_str().unwrap()).unwrap();
        let dict = unsafe { sbs_load_dictionary(path.as_ptr()) };
        assert!(!dict.is_null());

        let bad_json = CString::new("not json").unwrap();
        let result = unsafe { sbs_solve(dict, bad_json.as_ptr()) };
        assert!(!result.is_null());
        let s = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        assert!(s.contains("error"));

        unsafe {
            sbs_free_string(result);
            sbs_free_dictionary(dict);
        }
    }
}
