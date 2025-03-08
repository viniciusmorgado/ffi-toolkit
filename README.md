# ffi-toolkit

A Rust crate that provides shared patterns and utilities for creating Foreign Function Interfaces (FFI) from Rust to C and other languages.

## Overview

`ffi-toolkit` simplifies common challenges when exposing Rust functionality to other programming languages through FFI. It handles three crucial aspects:

1. **Memory Management** - Safely transfer ownership of Rust objects to and from C
2. **Error Handling** - Convert Rust's Result type to a C-compatible format
3. **String Conversion** - Transform between Rust strings and C strings

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
ffi-toolkit = "0.0.2"
```

## Usage Examples

### Memory Management

The `define_destructor` macro creates functions to properly free memory for Rust types exposed to C:

```rust
use ffi_toolkit::define_destructor;

// Define a custom type to expose via FFI
#[repr(C)]
pub struct MyData {
    value: i32,
}

// Create a destructor function for this type
define_destructor!(destroy_my_data, MyData);

// Expose a function to create this type
#[unsafe(no_mangle)]
pub extern "C" fn create_my_data(val: i32) -> *mut MyData {
    Box::into_raw(Box::new(MyData { value: val }))
}

// Now in C code:
// MyData* data = create_my_data(42);
// /* use data */
// destroy_my_data(data);
```

### Error Handling

The `ExternResult` type allows you to safely pass Rust's Result across FFI boundaries:

```rust
use ffi_toolkit::result::{ExternResult, ErrorCode};
use std::error::Error;

fn internal_operation() -> Result<i32, Box<dyn Error>> {
    // Some operation that might fail
    Ok(42)
}

#[unsafe(no_mangle)]
pub extern "C" fn perform_operation() -> *mut ExternResult {
    match internal_operation() {
        Ok(value) => ExternResult::ok(value),
        Err(e) => ExternResult::err(ErrorCode::Other, e.to_string())
    }
}

// In C:
// ExternResult* result = perform_operation();
// if (result->err != NULL) {
//     printf("Error: %s\n", result->err->message);
// } else {
//     int* value = (int*)result->ok;
//     printf("Result: %d\n", *value);
// }
// extern_result_destroy(result);
```

### String Conversion

Convert between Rust strings and C strings:

```rust
use ffi_toolkit::string::{string_to_c_char, c_char_to_string};
use std::os::raw::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn process_string(input: *const c_char) -> *mut c_char {
    // Convert C string to Rust string
    let rust_string = c_char_to_string(input);

    // Process the string
    let result = format!("Processed: {}", rust_string);

    // Convert back to C string
    string_to_c_char(result)
}

// In C:
// char* result = process_string("Hello, world!");
// printf("%s\n", result);
// destroy_c_char(result);
```

## API Reference

### Memory Module

- `define_destructor!(name, type)` - Creates a function to free memory for a specific type
- `define_destructor_with_lifetimes!(name, type)` - Creates a function to free memory for types with lifetimes
- `destroy(obj)` - Pre-defined destructor for `c_void` pointers
- `destroy_raw_uuid(obj)` - Pre-defined destructor for UUID byte arrays
- `destroy_c_char(s)` - Pre-defined destructor for C strings
- `assert_pointer_not_null!(expr)` - Macro to verify pointers are not null

### Result Module

- `ErrorCode` - Enum of possible error types
- `ExternError` - C-compatible error representation with code and message
- `ExternResult` - C-compatible result type with methods:
  - `ok(result)` - Create a success result
  - `ok_ptr(result)` - Create a success result from a pointer
  - `ok_null()` - Create a success result with a null value
  - `ok_optional(result)` - Create a result from an Option
  - `err(code, msg)` - Create an error result

### String Module

- `c_char_to_string(cchar)` - Convert a C string to a Rust string
- `string_to_c_char(r_string)` - Convert a Rust string to a C string

## Safety Notes

All FFI functions should be treated as unsafe. When using this library:

- Always free memory using the provided destructors
- Check for null pointers before dereferencing
- Handle errors properly on both sides of the FFI boundary
- Be aware of string encoding differences

## License

This project is licensed under the Mozilla Public License 2.0 - see the [LICENSE](LICENSE) file for details.
