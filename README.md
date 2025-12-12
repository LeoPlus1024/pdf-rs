# pdf-rs

[![Build Status](https://github.com/LeoPlus1024/pdf-rs/workflows/Rust/badge.svg)](https://github.com/your-username/pdf-rs/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

[//]: # ([![Crates.io]&#40;https://img.shields.io/crates/v/pdf-rs.svg&#41;]&#40;https://crates.io/crates/pdf-rs&#41;)

[English](#english) | [中文](README_zh.md)

## English

A PDF parsing library written in Rust.

### Overview

`pdf-rs` is a Rust library for parsing PDF files. The project aims to provide parsing functionality for PDF document structures, including:

- PDF version identification
- Cross-reference table (xref) parsing
- Object parsing (dictionaries, arrays, strings, etc.)
- Basic access to PDF structures

### Features

1. **PDF Version Support**: Supports PDF versions from 1.0 to 2.0
2. **Object Parsing**: Parses various object types in PDF, including dictionaries, arrays, strings, etc.
3. **Cross-reference Table Parsing**: Parses PDF's xref table to locate objects
4. **Stream Reading**: Uses `Sequence` trait for efficient streaming file reading
5. **Memory Efficiency**: Designed to minimize memory usage during parsing
6. **Error Handling**: Comprehensive error handling with detailed error messages
7. **Type Safety**: Fully utilizes Rust's type system for safety guarantees

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pdf-rs = "0.1"
```

### Usage Example

```rust
use std::path::PathBuf;
use pdf_rs::document::PDFDocument;

// Create PDF document parser
let path = PathBuf::from("example.pdf");
let document = PDFDocument::open(path)?;

// Access PDF version
println!("PDF Version: {}", document.get_version());

// Get cross-reference table
let xrefs = document.get_xref();
println!("XRef entries: {}", xrefs.len());
```

### API Documentation

For detailed API documentation, please refer to the [crate documentation](https://docs.rs/pdf-rs).

### Module Structure

- `document`: Main PDF document parsing functionality
- `objects`: PDF object representations (dictionaries, arrays, strings, etc.)
- `parser`: Core parsing logic for PDF objects
- `sequence`: Streaming file reading utilities
- `tokenizer`: Tokenization of PDF content
- `error`: Error types and handling

### Design Highlights

- **Modular Design**: Different functionalities separated into different modules for easy maintenance and extension
- **Error Handling**: Comprehensive error type system providing detailed error information
- **Memory Efficiency**: Streaming reading avoids loading entire files into memory
- **Type Safety**: Fully utilizes Rust's type system for safety guarantees
- **Extensibility**: Designed with extensibility in mind for future enhancements

### Current Status

The project is in early development stage. Basic PDF parsing functionality has been implemented, including version detection, xref table parsing, and basic object parsing.

### Future Plans

- Improve PDF object parsing functionality
- Add encrypted PDF support
- Implement advanced PDF content extraction features
- Provide more user-friendly API interfaces
- Add comprehensive documentation with examples
- Improve performance and memory usage

### Build Requirements

- Rust 1.5+ (latest stable version recommended)

### Build Steps

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request


### License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

### Acknowledgments

- Thanks to the Rust community for providing excellent tools and libraries
- Inspired by other PDF parsing libraries in different languages