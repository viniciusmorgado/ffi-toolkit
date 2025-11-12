# ffi-toolkit Roadmap

This roadmap outlines planned improvements and extensions to make `ffi-toolkit` a comprehensive solution for Rust FFI with both C and C++ codebases.

## Phase 1: Core Improvements

### Memory Management
- [ ] Support for custom allocators (allow C/C++ to provide their own allocation functions)
- [ ] Smart pointer adapters (convert between Rust's `Box`, `Rc`, `Arc` and C++ `std::shared_ptr`, `std::unique_ptr`)
- [ ] Memory pool utilities for high-performance FFI with many small allocations
- [ ] Debug utilities to track memory leaks across FFI boundaries

### Error Handling
- [x] Expanded error type system (more specific error codes)
- [ ] Error context preservation (stack traces where possible)
- [ ] Exception bridging for C++ (convert between Rust's `Result` and C++ exceptions)
- [ ] Error callbacks (allow registering C functions to be called on error)

### String Handling
- [ ] Support for non-UTF8 encodings (handling Windows' UTF-16, etc.)
- [ ] String view types (zero-copy string references when full ownership transfer is unnecessary)
- [ ] String pool utilities for frequently reused strings
- [ ] Automatic encoding detection and conversion

## Phase 2: Collection and Complex Types

### Collections
- [ ] Vector/array utilities (safely sharing arrays between languages)
- [ ] Map/dictionary conversion (between Rust's collections and C++ STL)
- [ ] Iterator adapters (exposing Rust iterators to C/C++)
- [ ] Slice utilities (zero-copy access to Rust slices from C/C++)

### Complex Data Types
- [ ] Struct field reflection helpers (for easier mapping between Rust and C/C++ structs)
- [ ] Enum conversion utilities (between Rust enums and C/C++ enums)
- [ ] Tagged union helpers (safe pattern for C unions and Rust enums)
- [ ] Option type utilities (idiomatic representation of Rust's Option in C/C++)

### Time and DateTime
- [ ] DateTime conversion (between Rust's chrono and C/C++ time types)
- [ ] Duration conversion utilities
- [ ] Timezone-aware conversions

## Phase 3: Advanced FFI Patterns

### Asynchronous FFI
- [ ] Callback registration system (for event-based APIs)
- [ ] Promise/Future bridging (between Rust futures and C++ promises)
- [ ] Cancelation token support
- [ ] Thread pool management across FFI boundary

### Resource Management
- [ ] RAII patterns for C (mimicking C++ RAII in C code)
- [ ] Resource handle tracking and validation
- [ ] File descriptor and handle management
- [ ] Cross-language reference counting

### Versioning and ABI Stability
- [ ] Version checking macros and functions
- [ ] ABI compatibility layers
- [ ] Feature detection across FFI boundaries
- [ ] Graceful degradation patterns

## Phase 4: Language-Specific Optimizations

### C++ Specific Features
- [ ] Template metadata extraction and binding
- [ ] STL container adapters (vector, map, unordered_map, etc.)
- [ ] Operator overloading support
- [ ] C++ namespaces integration

### Platform-Specific Optimizations
- [ ] Windows COM interface support
- [ ] POSIX-specific utilities
- [ ] Objective-C interoperability for macOS/iOS
- [ ] Android JNI helpers

## Phase 5: Tooling and Documentation

### Tooling
- [ ] C/C++ header generation from Rust code
- [ ] FFI boundary testing utilities
- [ ] Performance benchmarking tools
- [ ] API compatibility checkers

### Documentation and Examples
- [ ] Comprehensive API documentation with examples
- [ ] Language-specific integration guides (C, C++, Objective-C)
- [ ] Performance best practices
- [ ] Common pitfalls and solutions guide

## Phase 6: Community and Ecosystem

### Community Building
- [ ] Contribution guidelines
- [ ] Example projects and templates
- [ ] Integration with popular C and C++ libraries

### Ecosystem Integration
- [ ] Web Assembly (WASM) support
- [ ] Integration with cbindgen and other FFI tools
- [ ] Support for other languages beyond C/C++ (Python, Swift, etc.)
- [ ] Interoperability with other FFI solutions
