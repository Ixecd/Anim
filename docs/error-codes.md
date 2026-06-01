# Anim 错误码

> 由 `build.rs` 自动生成——源在 `src/error.rs`。不要手改此文件。

| 错误变体 | 说明 |
| -------- | ---- |
| `LexError` | 词法错误——源码里有交织器不认识的字符。 |
| `ParseError` | 语法错误——token 流不符合 .anim 语法。 |
| `TypeCheckError` | 类型检查错误——感受原子不存在、强度越界等。 |
| `StaticSafetyError` | 静态安全错误——Pass 2（rule.rs）。对任何人的硬规则。 |
| `UserStateSafetyError` | 用户安全错误——Pass 3（safety.rs）。对这个人的拒绝。 |
| `InternalError` | 交织器内部错误——不是用户源码的问题。 |
