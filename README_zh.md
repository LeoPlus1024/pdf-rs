# pdf-rs

[![Build Status](https://github.com/LeoPlus1024/pdf-rs/workflows/Rust/badge.svg)](https://github.com/your-username/pdf-rs/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

[//]: # ([![Crates.io]&#40;https://img.shields.io/crates/v/pdf-rs.svg&#41;]&#40;https://crates.io/crates/pdf-rs&#41;)

[中文](#chinese) | [English](README.md)

### 项目概述

`pdf-rs` 是一个用于解析 PDF 文件的 Rust 库。该项目旨在提供对 PDF 文档结构的解析功能，包括：

- PDF 版本识别
- 交叉引用表（xref）解析
- 对象解析（字典、数组、字符串等）
- 基本的 PDF 结构访问

### 关键特性

1. **PDF 版本支持**: 支持从 1.0 到 2.0 的 PDF 版本
2. **对象解析**: 解析 PDF 中的各种对象类型，包括字典、数组、字符串等
3. **交叉引用表解析**: 解析 PDF 的 xref 表以定位对象
4. **流式读取**: 使用 `Sequence` trait 实现高效的流式文件读取
5. **内存效率**: 设计为在解析过程中最小化内存使用
6. **错误处理**: 全面的错误处理和详细的错误信息
7. **类型安全**: 充分利用 Rust 的类型系统保证安全性

### 安装

将以下内容添加到您的 `Cargo.toml` 文件中：

```toml
[dependencies]
pdf-rs = "0.1"
```

### 使用示例

```rust
use std::path::PathBuf;
use pdf_rs::document::PDFDocument;

// 创建 PDF 文档解析器
let path = PathBuf::from("example.pdf");
let document = PDFDocument::open(path)?;

// 访问 PDF 版本
println!("PDF Version: {}", document.get_version());

// 获取交叉引用表
let xrefs = document.get_xref();
println!("XRef 条目数: {}", xrefs.len());
```

### API 文档

详细的 API 文档，请参考 [crate 文档](https://docs.rs/pdf-rs)。

### 模块结构

- `document`: 主要的 PDF 文档解析功能
- `objects`: PDF 对象表示（字典、数组、字符串等）
- `parser`: PDF 对象的核心解析逻辑
- `sequence`: 流式文件读取工具
- `tokenizer`: PDF 内容的标记化
- `error`: 错误类型和处理

### 设计亮点

- **模块化设计**: 各个功能分离到不同模块，便于维护和扩展
- **错误处理**: 完善的错误类型系统，提供详细的错误信息
- **内存效率**: 使用流式读取避免将整个文件加载到内存
- **类型安全**: 充分利用 Rust 的类型系统保证安全性
- **可扩展性**: 设计时考虑了未来的扩展性

### 当前状态

项目处于早期开发阶段，已实现基本的 PDF 解析功能，包括版本检测、xref 表解析和基本对象解析。

### 未来计划

- 完善 PDF 对象解析功能
- 添加加密 PDF 支持
- 实现更高级的 PDF 内容提取功能
- 提供更友好的 API 接口
- 添加全面的文档和示例
- 改进性能和内存使用

### 构建要求

- Rust 1.5+（推荐使用最新稳定版）

### 构建步骤

```bash
cargo build
```

### 运行测试

```bash
cargo test
```

### 贡献

欢迎贡献！请随时提交 Pull Request。

1. Fork 仓库
2. 创建您的功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交您的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request
6. 
### 许可证

该项目采用 Apache License 2.0 许可证 - 详情请见 [LICENSE](LICENSE) 文件。

### 致谢

- 感谢 Rust 社区提供了优秀的工具和库
- 受到其他语言中 PDF 解析库的启发