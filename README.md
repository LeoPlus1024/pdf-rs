# pdf-rs

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

### Usage Example

```rust
use std::fs::File;
use ipdf::document::PDFDocument;
use ipdf::sequence::FileSequence;

// Create PDF document parser
let file = File::open("example.pdf")?;
let sequence = FileSequence::new(file);
let document = PDFDocument::new(sequence)?;

// Access PDF version
println!("PDF Version: {}", document.get_version());

// Get cross-reference table
let xrefs = document.get_xref();
```

### Design Highlights

- **Modular Design**: Different functionalities separated into different modules for easy maintenance and extension
- **Error Handling**: Comprehensive error type system providing detailed error information
- **Memory Efficiency**: Streaming reading avoids loading entire files into memory
- **Type Safety**: Fully utilizes Rust's type system for safety guarantees

### Current Status

The project is in early development stage. Basic PDF parsing functionality has been implemented, including version detection, xref table parsing, and basic object parsing.

### Future Plans

- Improve PDF object parsing functionality
- Add encrypted PDF support
- Implement advanced PDF content extraction features
- Provide more user-friendly API interfaces

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

---