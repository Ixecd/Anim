# Anim 错误码

> 由 `build.rs` 自动生成——源在 `src/error.rs`。不要手改此文件。

| 错误变体 | 说明 |
| -------- | ---- |
| `LexError` | 词法错误——不合法字符或词法结构（Pass 0a）。 |
| `ParseError` | 语法错误——不符合 Anim 语法的结构（Pass 0b）。 |
| `TypeCheckError` | 类型错误——未注册原子/Shape 或无效配比（Pass 1）。 |
| `StaticSafetyError` | 静态安全——对任何人的硬规则（Pass 2 / rule.rs）。 |
| `UserStateSafetyError` | 用户安全——对这个人的拒绝（Pass 3 / safety.rs）。 |
| `InternalError` | 交织器内部错误。 |
