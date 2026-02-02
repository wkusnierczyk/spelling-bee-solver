//! FFI bindings for Spelling Bee Solver.
//!
//! Provides a C-compatible interface for loading dictionaries and solving puzzles.
//! Dictionary is managed as an opaque pointer (Box/unbox pattern). No global state.
//!
//! # Memory Safety Contract
//!
//! - Pointers returned by `sbs_load_dictionary` must be freed with `sbs_free_dictionary`.
//! - Pointers returned by `sbs_solve` must be freed with `sbs_free_string`.
//! - The pointer from `sbs_version` is static and must NOT be freed.
//! - No pointer may be used after it has been freed (use-after-free).
//! - No pointer may be freed more than once (double-free), except null which is always safe.

use sbs::{Config, Dictionary, Solver};
use std::ffi::{c_char, CStr, CString};

/// Static version string.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum allowed length for JSON request strings (1 MiB).
/// Guards against excessive memory allocation from untrusted input.
const MAX_REQUEST_LEN: usize = 1024 * 1024;

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
/// Passing null is a no-op.
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
/// Input is limited to 1 MiB to prevent excessive memory allocation.
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
    let json_bytes = c_str.to_bytes();

    if json_bytes.len() > MAX_REQUEST_LEN {
        return to_json_error("request too large");
    }

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

/// Free a string previously returned by `sbs_solve`.
///
/// Passing null is a no-op. Do NOT pass the pointer from `sbs_version` to this function.
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

    /// Helper: create a temp dictionary file with the given words.
    fn make_dict_file(words: &[&str]) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        for w in words {
            writeln!(tmp, "{}", w).unwrap();
        }
        tmp.flush().unwrap();
        tmp
    }

    /// Helper: load a dictionary from a temp file, returning the raw pointer.
    fn load_dict(tmp: &tempfile::NamedTempFile) -> *mut Dictionary {
        let path = CString::new(tmp.path().to_str().unwrap()).unwrap();
        let ptr = unsafe { sbs_load_dictionary(path.as_ptr()) };
        assert!(!ptr.is_null(), "failed to load dictionary");
        ptr
    }

    /// Helper: call sbs_solve and return the parsed JSON value.
    /// Frees the returned C string.
    fn solve_json(dict: *const Dictionary, request: &str) -> serde_json::Value {
        let req = CString::new(request).unwrap();
        let result = unsafe { sbs_solve(dict, req.as_ptr()) };
        assert!(!result.is_null());
        let s = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        unsafe { sbs_free_string(result) };
        parsed
    }

    // --- sbs_version tests ---

    #[test]
    fn test_version_returns_valid_string() {
        let ptr = sbs_version();
        assert!(!ptr.is_null());
        let version = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap();
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_version_returns_stable_pointer() {
        let ptr1 = sbs_version();
        let ptr2 = sbs_version();
        assert_eq!(ptr1, ptr2, "OnceLock should return the same pointer");
    }

    // --- sbs_load_dictionary tests ---

    #[test]
    fn test_load_dictionary_null_returns_null() {
        let ptr = unsafe { sbs_load_dictionary(std::ptr::null()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn test_load_dictionary_nonexistent_path_returns_null() {
        let path = CString::new("/nonexistent/path.txt").unwrap();
        let ptr = unsafe { sbs_load_dictionary(path.as_ptr()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn test_load_dictionary_empty_file() {
        let tmp = make_dict_file(&[]);
        let dict = load_dict(&tmp);
        // Empty dictionary should load successfully — just has no words
        let parsed = solve_json(dict, r#"{"letters":"abc","present":"a"}"#);
        let words = parsed["words"].as_array().unwrap();
        assert!(words.is_empty());
        unsafe { sbs_free_dictionary(dict) };
    }

    #[test]
    fn test_load_and_free_dictionary() {
        let tmp = make_dict_file(&["hello", "world"]);
        let dict = load_dict(&tmp);
        unsafe { sbs_free_dictionary(dict) };
        // No crash = success
    }

    // --- sbs_free_dictionary tests ---

    #[test]
    fn test_free_dictionary_null_is_noop() {
        unsafe { sbs_free_dictionary(std::ptr::null_mut()) };
        // No crash = success
    }

    // --- sbs_free_string tests ---

    #[test]
    fn test_free_string_null_is_noop() {
        unsafe { sbs_free_string(std::ptr::null_mut()) };
        // No crash = success
    }

    // --- sbs_solve null/error tests ---

    #[test]
    fn test_solve_both_null() {
        let result = unsafe { sbs_solve(std::ptr::null(), std::ptr::null()) };
        assert!(!result.is_null());
        let s = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed["error"], "null pointer argument");
        unsafe { sbs_free_string(result) };
    }

    #[test]
    fn test_solve_null_dict() {
        let req = CString::new(r#"{"letters":"abc","present":"a"}"#).unwrap();
        let result = unsafe { sbs_solve(std::ptr::null(), req.as_ptr()) };
        assert!(!result.is_null());
        let s = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed["error"], "null pointer argument");
        unsafe { sbs_free_string(result) };
    }

    #[test]
    fn test_solve_null_request() {
        let tmp = make_dict_file(&["test"]);
        let dict = load_dict(&tmp);
        let result = unsafe { sbs_solve(dict, std::ptr::null()) };
        assert!(!result.is_null());
        let s = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed["error"], "null pointer argument");
        unsafe {
            sbs_free_string(result);
            sbs_free_dictionary(dict);
        }
    }

    // --- sbs_solve JSON parsing tests ---

    #[test]
    fn test_solve_invalid_json() {
        let tmp = make_dict_file(&["test"]);
        let dict = load_dict(&tmp);
        let parsed = solve_json(dict, "not json");
        assert!(parsed["error"].as_str().unwrap().contains("invalid JSON"));
        unsafe { sbs_free_dictionary(dict) };
    }

    #[test]
    fn test_solve_empty_json_object() {
        let tmp = make_dict_file(&["test"]);
        let dict = load_dict(&tmp);
        // Empty object — letters and present are None, solver returns error
        let parsed = solve_json(dict, "{}");
        assert!(parsed.get("error").is_some());
        unsafe { sbs_free_dictionary(dict) };
    }

    #[test]
    fn test_solve_extra_fields_ignored() {
        let tmp = make_dict_file(&["pale", "leap", "plea", "peal", "apple"]);
        let dict = load_dict(&tmp);
        let parsed = solve_json(
            dict,
            r#"{"letters":"aple","present":"a","unknown_field":"ignored"}"#,
        );
        assert!(
            parsed.get("words").is_some(),
            "extra fields should be ignored"
        );
        unsafe { sbs_free_dictionary(dict) };
    }

    // --- sbs_solve functional tests ---

    #[test]
    fn test_solve_missing_letters() {
        let tmp = make_dict_file(&["test"]);
        let dict = load_dict(&tmp);
        let parsed = solve_json(dict, r#"{"present":"a"}"#);
        assert!(parsed.get("error").is_some());
        unsafe { sbs_free_dictionary(dict) };
    }

    #[test]
    fn test_solve_missing_present() {
        let tmp = make_dict_file(&["test"]);
        let dict = load_dict(&tmp);
        let parsed = solve_json(dict, r#"{"letters":"abc"}"#);
        assert!(parsed.get("error").is_some());
        unsafe { sbs_free_dictionary(dict) };
    }

    #[test]
    fn test_solve_roundtrip() {
        let tmp = make_dict_file(&[
            "apple", "appeal", "peal", "pale", "leap", "plea", "ape", "ale",
        ]);
        let dict = load_dict(&tmp);

        let parsed = solve_json(dict, r#"{"letters":"aple","present":"a"}"#);
        let words = parsed["words"].as_array().unwrap();
        assert!(!words.is_empty());

        // All returned words should contain the required letter 'a'
        for w in words {
            let word = w.as_str().unwrap();
            assert!(
                word.contains('a'),
                "word '{}' missing required letter",
                word
            );
        }

        unsafe { sbs_free_dictionary(dict) };
    }

    #[test]
    fn test_solve_results_sorted() {
        let tmp = make_dict_file(&["zebra", "able", "fable", "bale", "label"]);
        let dict = load_dict(&tmp);

        let parsed = solve_json(dict, r#"{"letters":"abelfz","present":"a"}"#);
        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        let mut sorted = words.clone();
        sorted.sort();
        assert_eq!(words, sorted, "results must be sorted alphabetically");

        unsafe { sbs_free_dictionary(dict) };
    }

    #[test]
    fn test_solve_no_matches() {
        let tmp = make_dict_file(&["xyz", "zzz"]);
        let dict = load_dict(&tmp);

        let parsed = solve_json(dict, r#"{"letters":"abc","present":"a"}"#);
        let words = parsed["words"].as_array().unwrap();
        assert!(words.is_empty());

        unsafe { sbs_free_dictionary(dict) };
    }

    #[test]
    fn test_solve_respects_min_word_length() {
        let tmp = make_dict_file(&["ab", "abc", "abcd", "abcde"]);
        let dict = load_dict(&tmp);

        // Default min length is 4
        let parsed = solve_json(dict, r#"{"letters":"abcde","present":"a"}"#);
        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        for w in &words {
            assert!(w.len() >= 4, "word '{}' is shorter than min length 4", w);
        }

        unsafe { sbs_free_dictionary(dict) };
    }

    // --- Word length constraint tests ---

    #[test]
    fn test_solve_with_minimal_word_length() {
        let tmp = make_dict_file(&["ab", "abc", "abcd", "abcde"]);
        let dict = load_dict(&tmp);

        let parsed = solve_json(
            dict,
            r#"{"letters":"abcde","present":"a","minimal-word-length":5}"#,
        );
        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        assert!(words.contains(&"abcde"));
        assert!(!words.contains(&"abcd"));
        assert!(!words.contains(&"abc"));

        unsafe { sbs_free_dictionary(dict) };
    }

    #[test]
    fn test_solve_with_maximal_word_length() {
        let tmp = make_dict_file(&["ab", "abc", "abcd", "abcde"]);
        let dict = load_dict(&tmp);

        let parsed = solve_json(
            dict,
            r#"{"letters":"abcde","present":"a","minimal-word-length":2,"maximal-word-length":3}"#,
        );
        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        assert!(words.contains(&"ab"));
        assert!(words.contains(&"abc"));
        assert!(!words.contains(&"abcd"));

        unsafe { sbs_free_dictionary(dict) };
    }

    // --- Input size limit test ---

    #[test]
    fn test_solve_rejects_oversized_input() {
        let tmp = make_dict_file(&["test"]);
        let dict = load_dict(&tmp);

        // Create a JSON string larger than MAX_REQUEST_LEN
        let large = format!(
            r#"{{"letters":"abc","present":"a","output":"{}"}}"#,
            "x".repeat(MAX_REQUEST_LEN + 1)
        );
        let req = CString::new(large).unwrap();
        let result = unsafe { sbs_solve(dict, req.as_ptr()) };
        assert!(!result.is_null());
        let s = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed["error"], "request too large");

        unsafe {
            sbs_free_string(result);
            sbs_free_dictionary(dict);
        }
    }
}
