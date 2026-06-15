# Anim 错误码

> 由 `build.rs` 自动生成——源在 `src/error.rs`。不要手改此文件。

| 错误变体 | 说明 |
| -------- | ---- |
| `StaticSafetyError` | 静态安全——对任何人的硬规则（Pass 2 / rule.rs）。 |
| `UserStateSafetyError` | 用户安全——对这个人的拒绝（Pass 3 / safety.rs）。 |
| `SafetyBreach` | 时域能量累积熔断——ADR 011 Neuro-Leaky Bucket。 |
| `InternalError` | 交织器内部错误。 |
