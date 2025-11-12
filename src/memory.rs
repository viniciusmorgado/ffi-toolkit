/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::os::raw::{c_char, c_void};

/// Creates a function with a given `$name` that releases the memory for a type `$t`.
#[macro_export]
macro_rules! define_destructor (
    ($name:ident, $t:ty) => (
        #[unsafe(no_mangle)]
        extern "C" fn $name(obj: *mut $t) {
            let _ = unsafe{ Box::from_raw(obj) };
        }
    )
);

/// Creates a function with a given `$name` that releases the memory
/// for a type `$t` with lifetimes <'a, 'c>.
/// TODO: Move to using `macro_rules` lifetime specifier when it lands in stable
/// This will enable us to specialise `define_destructor` and use repetitions
/// to allow more generic lifetime handling instead of having two functions.
/// https://github.com/rust-lang/rust/issues/34303
/// https://github.com/mozilla/mentat/issues/702
#[macro_export]
macro_rules! define_destructor_with_lifetimes (
    ($name:ident, $t:ty) => (
        #[no_mangle]
        pub extern "C" fn $name<'a, 'c>(obj: *mut $t) {
            let _ = unsafe{ Box::from_raw(obj) };
        }
    )
);

define_destructor!(destroy, c_void);

#[unsafe(no_mangle)]
pub extern "C" fn destroy_raw_uuid(obj: *mut [u8; 16]) {
    let _ = unsafe { Box::from_raw(obj) };
}

#[unsafe(no_mangle)]
pub extern "C" fn destroy_c_char(s: *mut c_char) {
    let _ = unsafe { CString::from_raw(s) };
}

#[macro_export]
macro_rules! assert_pointer_not_null {
    ($($e:expr),+ $(,)*) => ($(
        assert!(!$e.is_null(), concat!("Unexpected null pointer: ", stringify!($e)));
    )+);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    // Test structure for destructor macro testing
    #[derive(Debug, PartialEq)]
    struct TestStruct {
        value: i32,
        name: String,
    }

    // Define a custom destructor for our test struct
    define_destructor!(destroy_test_struct, TestStruct);

    #[test]
    fn test_destroy_test_struct_valid_pointer() {
        // Create a boxed value and convert to raw pointer
        let test_obj = Box::new(TestStruct {
            value: 42,
            name: String::from("test"),
        });
        let raw_ptr = Box::into_raw(test_obj);

        // This should not panic - the destructor should properly clean up
        destroy_test_struct(raw_ptr);
    }

    #[test]
    fn test_destroy_c_void_valid_pointer() {
        // Test the generic c_void destructor
        let value = Box::new(123u32);
        let raw_ptr = Box::into_raw(value) as *mut c_void;

        // This should not panic
        destroy(raw_ptr);
    }

    #[test]
    fn test_destroy_raw_uuid_valid_pointer() {
        // Create a UUID-like array on the heap
        let uuid = Box::new([1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let raw_ptr = Box::into_raw(uuid);

        // This should properly clean up the memory
        destroy_raw_uuid(raw_ptr);
    }

    #[test]
    fn test_destroy_c_char_valid_cstring() {
        // Create a CString and convert to raw pointer
        let c_string = CString::new("Hello, FFI!").expect("CString creation failed");
        let raw_ptr = c_string.into_raw();

        // This should properly clean up the CString
        destroy_c_char(raw_ptr);
    }

    #[test]
    fn test_destroy_c_char_empty_string() {
        // Test with empty string
        let c_string = CString::new("").expect("CString creation failed");
        let raw_ptr = c_string.into_raw();

        destroy_c_char(raw_ptr);
    }

    #[test]
    fn test_destroy_c_char_unicode_string() {
        // Test with Unicode content
        let c_string = CString::new("Hello 世界 🦀").expect("CString creation failed");
        let raw_ptr = c_string.into_raw();

        destroy_c_char(raw_ptr);
    }

    #[test]
    fn test_assert_pointer_not_null_valid() {
        let value = Box::new(42);
        let ptr = Box::into_raw(value);

        // Should not panic
        assert_pointer_not_null!(ptr);

        // Clean up
        unsafe {
            let _ = Box::from_raw(ptr);
        };
    }

    #[test]
    fn test_assert_pointer_not_null_multiple() {
        let v1 = Box::new(1);
        let v2 = Box::new(2);
        let ptr1 = Box::into_raw(v1);
        let ptr2 = Box::into_raw(v2);

        // Should not panic with multiple pointers
        assert_pointer_not_null!(ptr1, ptr2);

        // Clean up
        unsafe {
            let _ = Box::from_raw(ptr1);
            let _ = Box::from_raw(ptr2);
        }
    }

    #[test]
    #[should_panic(expected = "Unexpected null pointer")]
    fn test_assert_pointer_not_null_panics_on_null() {
        let null_ptr: *mut i32 = ptr::null_mut();
        assert_pointer_not_null!(null_ptr);
    }

    // Test to verify macro-generated function has correct signature
    #[test]
    fn test_destructor_macro_generates_extern_c_function() {
        // This test verifies that the destructor can be called like a C function
        // The fact that it compiles proves the signature is correct
        let test_fn: extern "C" fn(*mut TestStruct) = destroy_test_struct;

        let obj = Box::new(TestStruct {
            value: 100,
            name: String::from("macro test"),
        });
        let ptr = Box::into_raw(obj);

        test_fn(ptr);
    }

    // Test memory safety: ensure we can create and destroy multiple objects
    #[test]
    fn test_multiple_allocations_and_destructions() {
        for i in 0..100 {
            let obj = Box::new(TestStruct {
                value: i,
                name: format!("Object {}", i),
            });
            let ptr = Box::into_raw(obj);
            destroy_test_struct(ptr);
        }
    }

    // Test with different primitive types
    #[test]
    fn test_destroy_various_types() {
        // Test with u64
        let val_u64 = Box::new(u64::MAX);
        let ptr_u64 = Box::into_raw(val_u64) as *mut c_void;
        destroy(ptr_u64);

        // Test with f64
        let val_f64 = Box::new(3.14159f64);
        let ptr_f64 = Box::into_raw(val_f64) as *mut c_void;
        destroy(ptr_f64);

        // Test with a larger struct
        let val_large = Box::new([0u8; 1024]);
        let ptr_large = Box::into_raw(val_large) as *mut c_void;
        destroy(ptr_large);
    }
}
