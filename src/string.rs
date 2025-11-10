/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub fn c_char_to_string<'a>(cchar: *const c_char) -> &'a str {
    let c_str = unsafe { CStr::from_ptr(cchar) };
    c_str.to_str().unwrap_or("")
}

pub fn string_to_c_char<T>(r_string: T) -> *mut c_char
where
    T: Into<String>,
{
    CString::new(r_string.into()).unwrap().into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_c_char_basic() {
        // Test basic ASCII string conversion
        let rust_str = "Hello, World!";
        let c_str_ptr = string_to_c_char(rust_str);

        // Verify the pointer is not null
        assert!(!c_str_ptr.is_null());

        // Clean up
        unsafe {
            let _ = CString::from_raw(c_str_ptr);
        }
    }

    #[test]
    fn test_string_to_c_char_empty() {
        // Test empty string
        let rust_str = "";
        let c_str_ptr = string_to_c_char(rust_str);

        assert!(!c_str_ptr.is_null());

        unsafe {
            let _ = CString::from_raw(c_str_ptr);
        }
    }

    #[test]
    fn test_string_to_c_char_unicode() {
        // Test Unicode string
        let rust_str = "Hello 世界 🦀";
        let c_str_ptr = string_to_c_char(rust_str);

        assert!(!c_str_ptr.is_null());

        unsafe {
            let _ = CString::from_raw(c_str_ptr);
        }
    }

    #[test]
    fn test_c_char_to_string_basic() {
        // Create a C string and convert it back to Rust
        let original = "Test String";
        let c_str = CString::new(original).unwrap();
        let c_str_ptr = c_str.as_ptr();

        let result = c_char_to_string(c_str_ptr);

        assert_eq!(result, original);
    }

    #[test]
    fn test_c_char_to_string_empty() {
        // Test empty string conversion
        let c_str = CString::new("").unwrap();
        let c_str_ptr = c_str.as_ptr();

        let result = c_char_to_string(c_str_ptr);

        assert_eq!(result, "");
    }

    #[test]
    fn test_c_char_to_string_unicode() {
        // Test Unicode conversion
        let original = "Hello 世界 🦀 Rust!";
        let c_str = CString::new(original).unwrap();
        let c_str_ptr = c_str.as_ptr();

        let result = c_char_to_string(c_str_ptr);

        assert_eq!(result, original);
    }

    #[test]
    fn test_c_char_to_string_special_chars() {
        // Test with various special characters
        let original = "Line1\nLine2\tTabbed\r\nWindows";
        let c_str = CString::new(original).unwrap();
        let c_str_ptr = c_str.as_ptr();

        let result = c_char_to_string(c_str_ptr);

        assert_eq!(result, original);
    }

    #[test]
    fn test_round_trip_conversion() {
        // Test converting Rust string -> C string -> Rust string
        let original = "Round trip test!";
        let c_str_ptr = string_to_c_char(original);

        let result = c_char_to_string(c_str_ptr);

        assert_eq!(result, original);

        // Clean up
        unsafe {
            let _ = CString::from_raw(c_str_ptr);
        }
    }

    #[test]
    fn test_round_trip_unicode() {
        // Test round trip with Unicode
        let original = "Rust 🦀 世界 测试";
        let c_str_ptr = string_to_c_char(original);

        let result = c_char_to_string(c_str_ptr);

        assert_eq!(result, original);

        unsafe {
            let _ = CString::from_raw(c_str_ptr);
        }
    }

    #[test]
    fn test_multiple_conversions() {
        // Test multiple conversions to ensure no memory leaks
        let strings = vec!["First", "Second", "Third", "Fourth", "Fifth"];

        for s in strings {
            let c_ptr = string_to_c_char(s);
            let result = c_char_to_string(c_ptr);
            assert_eq!(result, s);

            unsafe {
                let _ = CString::from_raw(c_ptr);
            }
        }
    }

    #[test]
    fn test_string_to_c_char_with_string_type() {
        // Test with owned String instead of &str
        let owned_string = String::from("Owned String");
        let c_str_ptr = string_to_c_char(owned_string.clone());

        let result = c_char_to_string(c_str_ptr);
        assert_eq!(result, owned_string);

        unsafe {
            let _ = CString::from_raw(c_str_ptr);
        }
    }

    #[test]
    fn test_c_char_to_string_invalid_utf8_returns_empty() {
        // Create a properly null-terminated byte array with invalid UTF-8 sequence
        // This tests the unwrap_or("") fallback behavior
        // Note: We need to use a static array to ensure it lives long enough
        static INVALID_UTF8: [u8; 4] = [0xFF, 0xFE, 0xFD, 0x00];

        let c_str_ptr = INVALID_UTF8.as_ptr() as *const c_char;
        let result = c_char_to_string(c_str_ptr);

        // Should return empty string on invalid UTF-8
        assert_eq!(result, "");
    }

    #[test]
    fn test_string_to_c_char_null_terminated() {
        // Verify that the C string is properly null-terminated
        let rust_str = "Test";
        let c_str_ptr = string_to_c_char(rust_str);

        unsafe {
            // Check that we can create a CStr from the pointer
            let c_str = CStr::from_ptr(c_str_ptr);
            assert_eq!(c_str.to_str().unwrap(), rust_str);

            // Clean up
            let _ = CString::from_raw(c_str_ptr);
        }
    }

    #[test]
    fn test_long_string_conversion() {
        // Test with a longer string to ensure no buffer issues
        let long_string = "a".repeat(1000);
        let c_str_ptr = string_to_c_char(long_string.as_str());

        let result = c_char_to_string(c_str_ptr);
        assert_eq!(result, long_string);

        unsafe {
            let _ = CString::from_raw(c_str_ptr);
        }
    }

    #[test]
    fn test_string_with_embedded_quotes() {
        // Test strings with various quote characters
        let original = r#"He said "Hello" and 'Goodbye'"#;
        let c_str_ptr = string_to_c_char(original);

        let result = c_char_to_string(c_str_ptr);
        assert_eq!(result, original);

        unsafe {
            let _ = CString::from_raw(c_str_ptr);
        }
    }
}
