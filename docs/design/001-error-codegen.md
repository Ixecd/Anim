# ADR 001: 错误码代码生成器

> 日期：2026-05-23
> 状态：定稿
> 性质：Anim 工程基础设施
> 参考：KubePivot `tools/codegen/codegen.go`

---

## 决策

错误码的**唯一定义源**是 Rust 源代码中的枚举变体 + 注释。文档（Markdown 表格）由构建工具自动生成，不手写。代码变 → 文档自动变。

## 为什么不是手写文档

```
手写
    改了一个错误码 → 要同步改 src/error.rs 和 docs/error-codes.md
    忘了 → 文档是错的
    任何人都能看出这是两件事

生成
    源 = src/error.rs 里的枚举 + 注释
    工具读源 → 输出 docs/error-codes.md
    不依赖人记得同步
```

和 KubePivot 的 `codegen -type=ErrorCode -doc` 同构。唯一的区别——KubePivot 的源是 Go 常量，Anim 的源是 Rust 枚举变体。

## 注释格式——严格正则

每个枚举变体必须附带一个遵循固定格式的文档注释。和 KubePivot 的注释正则完全同构：

```rust
// src/error.rs

#[derive(Debug)]
pub enum AnimiError {
    /// E001 - 500: Atom not found in pattern registry.
    AtomNotFound,

    /// E002 - 422: Intensity exceeds user cap.
    IntensityExceedsCap,

    /// E003 - 403: Minor access denied.
    MinorAccessDenied,
}
```

格式：

```
/// E{code} - {http_status}: {Description starting with capital}.
```

正则：

```
^E\d{3}\s*-\s*(\d{3})\s*:\s*([A-Z].*)\.\s*$
```

- `E001-E999`：Anim 内部错误码
- `HTTP 状态码`：映射到对外 API 响应
- `Description`：大写开头，英文句号结尾
- `///`：Rust 标准文档注释，同时被 rustdoc 和 codegen 工具消费

## 生成管线

```
[ src/error.rs ]                    源码——唯一定义源
       │
       ▼
[ cargo run --bin animi-codegen ]   构建工具
       │                             ├── parse: 读 error.rs AST
       │                             ├── extract: 提取 #[doc] 注释
       │                             ├── validate: 正则严格匹配
       │                             └── generate:
       │
       ├──► src/error_display.rs     Display impl（自动生成）
       │
       └──► docs/error-codes.md     错误码编目文档（自动生成）


make dev 流程

    1. cargo build
    2. cargo run --bin animi-codegen  ← 在 cargo build 之后、cargo test 之前
    3. cargo test --lib
    4. cargo clippy -- -D warnings
    5. cargo fmt --check
```

## 工具实现——Rust build script

不做 proc macro——proc macro 修改 TokenStream，这里只需要读源生成别的文件。用 build script（`build.rs`）配合 `syn` crate 解析源文件。

```rust
// tools/codegen/src/main.rs（或 build.rs）

use syn::{Item, ItemEnum, LitStr};
use regex::Regex;

struct ErrorEntry {
    code: String,       // E001
    http_status: u16,   // 500
    description: String,// Atom not found in pattern registry
    variant: String,    // AtomNotFound
}

impl ErrorEntry {
    fn from_variant(variant: &syn::Variant) -> Option<Self> {
        // 提取 #[doc = "..."] 属性
        let doc = variant.attrs.iter()
            .find(|a| a.path().is_ident("doc"))
            .and_then(|a| {
                if let syn::Meta::NameValue(nv) = &a.meta {
                    if let syn::Expr::Lit(lit) = &nv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            return Some(s.value());
                        }
                    }
                }
                None
            })?;

        // 正则匹配
        let re = Regex::new(
            r"^E(\d{3})\s*-\s*(\d{3})\s*:\s*(.+)\."
        ).unwrap();
        let caps = re.captures(&doc)?;

        Some(ErrorEntry {
            code: format!("E{}", &caps[1]),
            http_status: caps[2].parse().ok()?,
            description: caps[3].to_string(),
            variant: variant.ident.to_string(),
        })
    }
}
```

## 生成产物

### 产物一：Display impl

```rust
// src/error_display.rs（自动生成，勿手动编辑）

impl std::fmt::Display for AnimiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimiError::AtomNotFound =>
                write!(f, "E001 - 500: Atom not found in pattern registry."),
            AnimiError::IntensityExceedsCap =>
                write!(f, "E002 - 422: Intensity exceeds user cap."),
            AnimiError::MinorAccessDenied =>
                write!(f, "E003 - 403: Minor access denied."),
        }
    }
}
```

### 产物二：错误码 Markdown 编目

```markdown
# Anim 错误码

自动生成于 `animi-codegen`。勿手动编辑。

| Code | Identifier | HTTP | Description |
|------|-----------|------|-------------|
| E001 | AtomNotFound | 500 | Atom not found in pattern registry |
| E002 | IntensityExceedsCap | 422 | Intensity exceeds user cap |
| E003 | MinorAccessDenied | 403 | Minor access denied |
```

## 性能——Rust build script vs C #define

```
C #define
    #define ERR_ATOM_NOT_FOUND 1001
    编译时替换为整数。运行时 = 一次整数比较。零开销。

Rust enum
    enum AnimiError { AtomNotFound, ... }
    编译后 = 判别值（discriminant）。和 C #define 完全等价。
    编译器为每个变体分配从 0 开始的整数标签。
    模式匹配编译为跳转表——和 switch/case 一致的 O(1) 分发。

build script vs 运行时
    codegen 在编译期运行——不在设备上运行。
    生成的 Display impl = 编译期展开的 match 臂。
    不产生任何额外的二进制大小（编译器会内联短的 write! 调用）。
    和手写 Display 完全一样的性能。

结论
    Rust enum + codegen 的运行时性能 = C #define + 手写 switch。
    差异在编译期——多跑了一个 build script。
    build script 只跑一次（源码未变时 cargo 自动跳过）。
```

## 和 KubePivot codegen 的同构

| | KubePivot | Anim |
|---|---|---|
| 语言 | Go | Rust |
| 源码形式 | `const ErrX ErrorCode = iota` | `enum AnimiError { ... }` |
| 注释格式 | `// ErrX - 500: Description.` | `/// EXXX - 500: Description.` |
| 解析层 | `go/ast` + `go/types` | `syn` crate |
| 生成代码 | `String()` switch 方法 | `Display` impl |
| 生成文档 | Markdown 表格 | Markdown 表格 |
| 触发方式 | `codegen -type=ErrorCode -doc` | `build.rs`，自动在 `cargo build` 后触发 |

## 禁止事项

```
❌ 手写 Display impl for AnimiError —— 生成的不允许手动编辑
❌ 手写 docs/error-codes.md —— 全部由工具生成
❌ 注释格式不匹配还提交 —— CI 检查 → 失败 → 阻断 merge
❌ 错误码重复 —— 工具扫描全部变体，重复直接报错
```
