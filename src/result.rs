/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std;
use std::os::raw::{c_char, c_void};

/// Error codes that can be returned across the FFI boundary.
/// These codes provide a standardized way to communicate error types
/// between Rust and C/C++ code.
#[repr(C)]
#[derive(Debug)]
pub enum ErrorCode {
    /// Generic error for cases that don't fit other categories
    Other,
    /// Authentication or authorization failed
    AuthenticationError,
    /// Input validation failed (invalid format, out of range, etc.)
    ValidationError,
    /// Requested resource or item was not found
    NotFoundError,
    /// Operation not permitted due to insufficient permissions
    PermissionError,
    /// Operation timed out
    TimeoutError,
    /// Network-related error (connection failed, DNS error, etc.)
    NetworkError,
    /// Invalid argument passed to function
    InvalidArgumentError,
    /// I/O operation failed (file read/write, etc.)
    IoError,
}

/// An error struct containing an error code and a description string.
/// #Safety
///
/// Callers are responsible for managing the memory for the return value.
/// A destructor `free_extern_error` is provided for releasing the memory for this
/// pointer type.
#[repr(C)]
#[derive(Debug)]
pub struct ExternError {
    code: ErrorCode,
    message: *const c_char,
}

/// A C representation of Rust's [Result](std::result::Result).
/// A value of `Ok` results in `ok` containing a raw pointer as a `c_void`
/// and `err` containing a null pointer.
/// A value of `Err` results in `value` containing a null pointer and `err` containing an error struct.
///
/// #Safety
///
/// Callers are responsible for managing the memory for the return value.
/// A destructor `extern_result_destroy` is provided for releasing the memory for this
/// pointer type.
#[repr(C)]
#[derive(Debug)]
pub struct ExternResult {
    pub ok: *const c_void, // We could have used `*const T` instead, but that would have meant creating one `free` function per variant.
    pub err: *const ExternError,
}

impl ExternResult {
    pub fn ok<T>(result: T) -> *mut Self {
        Self::ok_ptr(Box::into_raw(Box::new(result)))
    }

    pub fn ok_ptr<T>(result: *mut T) -> *mut Self {
        Box::into_raw(Box::new(ExternResult {
            ok: result as *const _ as *const c_void,
            err: std::ptr::null_mut(),
        }))
    }

    pub fn ok_null() -> *mut Self {
        Box::into_raw(Box::new(ExternResult {
            ok: std::ptr::null_mut(),
            err: std::ptr::null_mut(),
        }))
    }

    pub fn ok_optional<T>(result: &Option<T>) -> *mut Self {
        match result {
            Some(t) => Self::ok(t),
            None => Self::ok_null(),
        }
    }

    pub fn err<S>(code: ErrorCode, msg: S) -> *mut Self
    where
        S: Into<String>,
    {
        Box::into_raw(Box::new(ExternResult {
            ok: std::ptr::null_mut(),
            err: Box::into_raw(Box::new(ExternError {
                code,
                message: crate::string::string_to_c_char(msg),
            })),
        }))
    }
}

impl<T, E> From<Result<T, E>> for ExternResult
where
    E: std::error::Error,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => ExternResult {
                ok: Box::into_raw(Box::new(value)) as *const _ as *const c_void,
                err: std::ptr::null(),
            },
            Err(e) => ExternResult {
                ok: std::ptr::null(),
                err: Box::into_raw(Box::new(ExternError {
                    code: ErrorCode::Other,
                    message: crate::string::string_to_c_char(e.to_string()),
                })),
            },
        }
    }
}

define_destructor!(extern_result_destroy, ExternResult);

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    // Helper error type for testing
    #[derive(Debug)]
    struct TestError {
        message: String,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for TestError {}

    #[test]
    fn test_extern_result_ok_basic() {
        let value = 42i32;
        let result_ptr = ExternResult::ok(value);

        unsafe {
            assert!(!result_ptr.is_null());
            let result = &*result_ptr;
            assert!(!result.ok.is_null());
            assert!(result.err.is_null());

            // Clean up - free inner value first, then the result
            let _ = Box::from_raw(result.ok as *mut i32);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_extern_result_ok_ptr() {
        let value = Box::new(100u64);
        let value_ptr = Box::into_raw(value);
        let result_ptr = ExternResult::ok_ptr(value_ptr);

        unsafe {
            assert!(!result_ptr.is_null());
            let result = &*result_ptr;
            assert!(!result.ok.is_null());
            assert!(result.err.is_null());
            assert_eq!(result.ok as *const u64, value_ptr as *const u64);

            // Clean up - free inner value first, then the result
            let _ = Box::from_raw(value_ptr);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_extern_result_ok_null() {
        let result_ptr = ExternResult::ok_null();

        unsafe {
            assert!(!result_ptr.is_null());
            let result = &*result_ptr;
            assert!(result.ok.is_null());
            assert!(result.err.is_null());

            // Clean up
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_extern_result_ok_optional_some() {
        let value = Some(42i32);
        let result_ptr = ExternResult::ok_optional(&value);

        unsafe {
            assert!(!result_ptr.is_null());
            let result = &*result_ptr;
            assert!(!result.ok.is_null());
            assert!(result.err.is_null());

            // Clean up - need to free the value inside
            let _ = Box::from_raw(result.ok as *mut i32);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_extern_result_ok_optional_none() {
        let value: Option<i32> = None;
        let result_ptr = ExternResult::ok_optional(&value);

        unsafe {
            assert!(!result_ptr.is_null());
            let result = &*result_ptr;
            assert!(result.ok.is_null());
            assert!(result.err.is_null());

            // Clean up
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_extern_result_err() {
        let result_ptr = ExternResult::err(ErrorCode::AuthenticationError, "Auth failed");

        unsafe {
            assert!(!result_ptr.is_null());
            let result = &*result_ptr;
            assert!(result.ok.is_null());
            assert!(!result.err.is_null());

            let error = &*result.err;
            assert!(!error.message.is_null());

            // Verify error message
            let c_str = std::ffi::CStr::from_ptr(error.message);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, "Auth failed");

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_extern_result_err_with_string() {
        let error_msg = String::from("Error message");
        let result_ptr = ExternResult::err(ErrorCode::Other, error_msg.clone());

        unsafe {
            assert!(!result_ptr.is_null());
            let result = &*result_ptr;
            assert!(result.ok.is_null());
            assert!(!result.err.is_null());

            let error = &*result.err;
            let c_str = std::ffi::CStr::from_ptr(error.message);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, error_msg);

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_from_result_ok() {
        let rust_result: Result<i32, TestError> = Ok(123);
        let extern_result = ExternResult::from(rust_result);

        assert!(!extern_result.ok.is_null());
        assert!(extern_result.err.is_null());

        unsafe {
            let value = *(extern_result.ok as *const i32);
            assert_eq!(value, 123);

            // Clean up
            let _ = Box::from_raw(extern_result.ok as *mut i32);
        }
    }

    #[test]
    fn test_from_result_err() {
        let rust_result: Result<i32, TestError> = Err(TestError {
            message: String::from("Test error"),
        });
        let extern_result = ExternResult::from(rust_result);

        assert!(extern_result.ok.is_null());
        assert!(!extern_result.err.is_null());

        unsafe {
            let error = &*extern_result.err;
            let c_str = std::ffi::CStr::from_ptr(error.message);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, "Test error");

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(extern_result.err as *mut ExternError);
        }
    }

    #[test]
    fn test_error_code_variants() {
        // Test both error code variants
        let auth_err = ExternResult::err(ErrorCode::AuthenticationError, "Auth error");
        let other_err = ExternResult::err(ErrorCode::Other, "Other error");

        unsafe {
            let auth_error = &*(&*auth_err).err;
            let other_error = &*(&*other_err).err;

            // Verify we can distinguish error codes
            match auth_error.code {
                ErrorCode::AuthenticationError => {}
                _ => panic!("Expected AuthenticationError"),
            }

            match other_error.code {
                ErrorCode::Other => {}
                _ => panic!("Expected Other error"),
            }

            // Clean up
            let _ = CString::from_raw(auth_error.message as *mut _);
            let _ = Box::from_raw((&*auth_err).err as *mut ExternError);
            let _ = Box::from_raw(auth_err);

            let _ = CString::from_raw(other_error.message as *mut _);
            let _ = Box::from_raw((&*other_err).err as *mut ExternError);
            let _ = Box::from_raw(other_err);
        }
    }

    #[test]
    fn test_extern_result_destroy() {
        // Test that the destructor doesn't crash with a valid pointer
        let result_ptr = ExternResult::ok(42i32);

        unsafe {
            // First free the inner value
            let result = &*result_ptr;
            let _ = Box::from_raw(result.ok as *mut i32);
        }

        // Now destroy the ExternResult itself
        extern_result_destroy(result_ptr);
    }

    #[test]
    fn test_multiple_extern_results() {
        // Create multiple results to ensure no memory conflicts
        let results: Vec<*mut ExternResult> = (0..10).map(|i| ExternResult::ok(i)).collect();

        unsafe {
            for result_ptr in results {
                let result = &*result_ptr;
                assert!(!result.ok.is_null());
                assert!(result.err.is_null());

                // Clean up
                let _ = Box::from_raw(result.ok as *mut i32);
                let _ = Box::from_raw(result_ptr);
            }
        }
    }

    #[test]
    fn test_extern_result_with_complex_type() {
        #[derive(Debug, PartialEq)]
        struct ComplexType {
            id: u64,
            name: String,
            values: Vec<i32>,
        }

        let complex = ComplexType {
            id: 123,
            name: String::from("Test"),
            values: vec![1, 2, 3, 4, 5],
        };

        let result_ptr = ExternResult::ok(complex);

        unsafe {
            let result = &*result_ptr;
            assert!(!result.ok.is_null());

            let value = &*(result.ok as *const ComplexType);
            assert_eq!(value.id, 123);
            assert_eq!(value.name, "Test");
            assert_eq!(value.values, vec![1, 2, 3, 4, 5]);

            // Clean up
            let _ = Box::from_raw(result.ok as *mut ComplexType);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_extern_error_unicode_message() {
        let result_ptr = ExternResult::err(ErrorCode::Other, "Error: 错误 🚨");

        unsafe {
            let result = &*result_ptr;
            let error = &*result.err;
            let c_str = std::ffi::CStr::from_ptr(error.message);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, "Error: 错误 🚨");

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_all_error_code_variants() {
        // Test all error code variants to ensure they can be created and used
        let test_cases = vec![
            (ErrorCode::Other, "Generic error"),
            (ErrorCode::AuthenticationError, "Auth failed"),
            (ErrorCode::ValidationError, "Invalid input"),
            (ErrorCode::NotFoundError, "Resource not found"),
            (ErrorCode::PermissionError, "Access denied"),
            (ErrorCode::TimeoutError, "Request timed out"),
            (ErrorCode::NetworkError, "Network unavailable"),
            (ErrorCode::InvalidArgumentError, "Bad argument"),
            (ErrorCode::IoError, "File read failed"),
        ];

        for (code, message) in test_cases {
            let result_ptr = ExternResult::err(code, message);

            unsafe {
                assert!(!result_ptr.is_null());
                let result = &*result_ptr;
                assert!(result.ok.is_null());
                assert!(!result.err.is_null());

                let error = &*result.err;
                let c_str = std::ffi::CStr::from_ptr(error.message);
                let msg_str = c_str.to_str().unwrap();
                assert_eq!(msg_str, message);

                // Clean up
                let _ = CString::from_raw(error.message as *mut _);
                let _ = Box::from_raw(result.err as *mut ExternError);
                let _ = Box::from_raw(result_ptr);
            }
        }
    }

    #[test]
    fn test_validation_error() {
        let result_ptr = ExternResult::err(
            ErrorCode::ValidationError,
            "Email format is invalid"
        );

        unsafe {
            let result = &*result_ptr;
            let error = &*result.err;

            match error.code {
                ErrorCode::ValidationError => {}
                _ => panic!("Expected ValidationError"),
            }

            let c_str = std::ffi::CStr::from_ptr(error.message);
            assert_eq!(c_str.to_str().unwrap(), "Email format is invalid");

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_not_found_error() {
        let result_ptr = ExternResult::err(
            ErrorCode::NotFoundError,
            "User with ID 123 not found"
        );

        unsafe {
            let result = &*result_ptr;
            let error = &*result.err;

            match error.code {
                ErrorCode::NotFoundError => {}
                _ => panic!("Expected NotFoundError"),
            }

            let c_str = std::ffi::CStr::from_ptr(error.message);
            assert_eq!(c_str.to_str().unwrap(), "User with ID 123 not found");

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_permission_error() {
        let result_ptr = ExternResult::err(
            ErrorCode::PermissionError,
            "Insufficient permissions to access resource"
        );

        unsafe {
            let result = &*result_ptr;
            let error = &*result.err;

            match error.code {
                ErrorCode::PermissionError => {}
                _ => panic!("Expected PermissionError"),
            }

            // Clean up
            let _ = CString::from_raw((&*result.err).message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_timeout_error() {
        let result_ptr = ExternResult::err(
            ErrorCode::TimeoutError,
            "Operation exceeded 30 second timeout"
        );

        unsafe {
            let result = &*result_ptr;
            let error = &*result.err;

            match error.code {
                ErrorCode::TimeoutError => {}
                _ => panic!("Expected TimeoutError"),
            }

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_network_error() {
        let result_ptr = ExternResult::err(
            ErrorCode::NetworkError,
            "Failed to connect to server: connection refused"
        );

        unsafe {
            let result = &*result_ptr;
            let error = &*result.err;

            match error.code {
                ErrorCode::NetworkError => {}
                _ => panic!("Expected NetworkError"),
            }

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_invalid_argument_error() {
        let result_ptr = ExternResult::err(
            ErrorCode::InvalidArgumentError,
            "Argument 'count' must be positive"
        );

        unsafe {
            let result = &*result_ptr;
            let error = &*result.err;

            match error.code {
                ErrorCode::InvalidArgumentError => {}
                _ => panic!("Expected InvalidArgumentError"),
            }

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_io_error() {
        let result_ptr = ExternResult::err(
            ErrorCode::IoError,
            "Failed to read file: permission denied"
        );

        unsafe {
            let result = &*result_ptr;
            let error = &*result.err;

            match error.code {
                ErrorCode::IoError => {}
                _ => panic!("Expected IoError"),
            }

            // Clean up
            let _ = CString::from_raw(error.message as *mut _);
            let _ = Box::from_raw(result.err as *mut ExternError);
            let _ = Box::from_raw(result_ptr);
        }
    }

    #[test]
    fn test_error_code_distinction() {
        // Ensure we can distinguish between different error codes
        let validation_err = ExternResult::err(ErrorCode::ValidationError, "Validation");
        let network_err = ExternResult::err(ErrorCode::NetworkError, "Network");
        let io_err = ExternResult::err(ErrorCode::IoError, "IO");

        unsafe {
            let v_error = &*(&*validation_err).err;
            let n_error = &*(&*network_err).err;
            let i_error = &*(&*io_err).err;

            // Verify each error has the correct code
            match v_error.code {
                ErrorCode::ValidationError => {}
                _ => panic!("Expected ValidationError"),
            }

            match n_error.code {
                ErrorCode::NetworkError => {}
                _ => panic!("Expected NetworkError"),
            }

            match i_error.code {
                ErrorCode::IoError => {}
                _ => panic!("Expected IoError"),
            }

            // Clean up
            let _ = CString::from_raw(v_error.message as *mut _);
            let _ = Box::from_raw((&*validation_err).err as *mut ExternError);
            let _ = Box::from_raw(validation_err);

            let _ = CString::from_raw(n_error.message as *mut _);
            let _ = Box::from_raw((&*network_err).err as *mut ExternError);
            let _ = Box::from_raw(network_err);

            let _ = CString::from_raw(i_error.message as *mut _);
            let _ = Box::from_raw((&*io_err).err as *mut ExternError);
            let _ = Box::from_raw(io_err);
        }
    }
}
