# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ffi-toolkit` is a Rust crate that provides shared patterns and utilities for creating Foreign Function Interfaces (FFI) from Rust to C and other languages. It handles three crucial aspects of FFI:

1. **Memory Management** - Safely transfer ownership of Rust objects to and from C
2. **Error Handling** - Convert Rust's Result type to a C-compatible format
3. **String Conversion** - Transform between Rust strings and C strings

## Development Commands

### Building
```bash
cargo build              # Debug build
cargo build --release    # Release build
```

### Testing
```bash
cargo test                    # Run all tests
cargo test <test_name>        # Run specific test by name
cargo test --no-fail-fast     # Run all tests even if some fail
```

### Other Commands
```bash
cargo check              # Fast check without building
cargo clippy             # Run linter
cargo doc --open         # Generate and open documentation
```

## Architecture

### Module Structure

The crate is organized into three core modules (in `src/`):

- **`memory.rs`**: Memory management utilities and macros
  - `define_destructor!` macro: Creates FFI-safe destructor functions for types
  - `define_destructor_with_lifetimes!` macro: Destructor macro for types with lifetimes
  - Pre-defined destructors: `destroy()`, `destroy_raw_uuid()`, `destroy_c_char()`
  - `assert_pointer_not_null!` macro: Pointer validation helper

- **`result.rs`**: Error handling across FFI boundary
  - `ErrorCode` enum: Standardized error codes (Other, AuthenticationError)
  - `ExternError` struct: C-compatible error representation with code and message
  - `ExternResult` struct: C-compatible Result type with `ok` and `err` pointers
  - Methods: `ok()`, `ok_ptr()`, `ok_null()`, `ok_optional()`, `err()`
  - Implements `From<Result<T, E>>` for automatic conversion

- **`string.rs`**: String conversion utilities
  - `c_char_to_string()`: Converts C string to Rust `&str`
  - `string_to_c_char()`: Converts Rust String/&str to C `*mut c_char`

### Key Design Patterns

**Memory Ownership Transfer**:
- Functions return `*mut T` (raw pointers) created via `Box::into_raw()`
- C side owns the memory after the call
- Must call corresponding destructor to avoid leaks
- Use `define_destructor!` macro to create type-safe destructors

**Error Handling Pattern**:
- Rust functions return `*mut ExternResult`
- Success: `ok` field contains data pointer, `err` is null
- Failure: `ok` is null, `err` contains `ExternError` with code and message
- Both `ok` value and `ExternResult` must be freed separately by caller

**String Conversion Safety**:
- `string_to_c_char()` allocates via `CString::new().into_raw()`
- Caller must free returned C string with `destroy_c_char()`
- `c_char_to_string()` returns empty string on invalid UTF-8 (via `unwrap_or("")`)

### Safety Considerations

- All FFI functions use `extern "C"` ABI and `#[unsafe(no_mangle)]` or `#[no_mangle]`
- Functions marked with `#[repr(C)]` for stable memory layout
- Test modules extensively verify memory safety, including:
  - Proper cleanup in all paths
  - Round-trip conversions
  - Edge cases (empty strings, null values, Unicode)
  - Multiple allocations/deallocations

## Testing Philosophy

The codebase has comprehensive unit tests for all modules:
- Each public function has multiple test cases
- Tests verify both success and edge cases
- Memory cleanup is tested in each test (no leaks)
- Round-trip conversions tested for string module
- Complex types and Unicode strings are tested
