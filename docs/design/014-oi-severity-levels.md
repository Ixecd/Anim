# ADR 014: oi 三层严重度 — oi! / oi? / oi~

> 状态：定稿
> 日期：2026-06-15
> 性质：oi 宏从单一"编译期拒绝"扩展为三层语义——和 Feelings 犟种协议同构
> 对应：ADR 006 错误处理系统升级

---

## 动机

当前 `oi!` 宏只有一种行为：`return Err(AnimiError::...)`——编译期硬拒绝。所有不合规的输入都被挡在门外。

但 Feelings 的哲学从来不是"系统替你锁车门"。GOVERNANCE-FEELINGS 里写着："犟种协议存在，但协议之后系统继续全力保护。" 有人站在悬崖边，你喊了一声 oi——他听见了，看了一眼，然后自己踩了油门。这不是系统失败——这是用户行使了选择权。

`oi` 需要表达的，不是一刀切的"禁止"，而是三个梯度：

- **严厉** — "这帧交叉被挡了。你的等控器物理上装不下这个。" 编译期拒绝。无协商余地。
- **警告** — "你确定？这里很深。我不建议，但门没锁。"
- **调侃** — "你这不是在冥想，是在装睡。" 语义调皮的提醒，不阻止任何事。

---

## 决策

`oi` 从单一宏扩展为三层宏 + `Severity` 枚举驱动。

### Severity 枚举

```
Severity:
  Deny   = 编译期硬拒绝——return Err。和当前 oi! 行为一致。
  Warn   = 打印到 stderr——return Ok 继续。--strict 下升级为 Deny。
  Note   = 仅 --verbose 模式打印——return Ok 继续。调试/语义调皮专用。
```

### 宏签名

```
oi!(variant, field=val, ...)          // 默认 Deny
oi!(variant, severity=Warn, field=..) // 显式 Warn
oi?(variant, field=val, ...)          // 语法糖——默认 Warn
oi~(variant, field=val, ...)          // 语法糖——默认 Note
```

`oi!` 不传 severity 时默认 `Deny`——向后兼容，所有现有调用点行为不变。

### AnimiError 升级

每个错误变体新增 `severity: Severity` 字段，由宏自动注入。`Display` 按 severity 切换前缀：

```
[oi]  词法错误 (test.anim:1:5): unexpected @        ← Deny
[oi?] 安全规则 (test.anim): abrupt_stop 建议不超过 20 ← Warn
[oi~] 类型提示 (test.anim): calm_zero — 强度为零的冥想包？← Note
```

### 规则降级

| 当前规则 | 旧 severity | 新 severity | 原因 |
|---------|-----------|-----------|------|
| 全局强度 > 100 | Deny | Deny | 生理硬线——等控器物理极限 |
| 沙箱做主旋律 | Deny | Deny | 安全硬线——未经审核的原子 |
| `abrupt_stop` > 20 | Deny | **Warn** | 设计建议——不是物理硬线 |
| 沙箱点缀 > 0.3 | Deny | **Warn** | 设计建议——配比上限是经验值，非生理极限 |

### CLI 参数

- `--strict` — 把全部 `Warn` 升级为 `Deny`。CI / 生产环境默认开启。
- `--verbose` — 显示 `Note` 级别输出。

`main.rs` 的 `die()` 函数改为 `die_with_severity()`——按 severity 分发：

```
Deny → A.error + process::exit(1)
Warn → A.warn + 吞掉 (return Ok(())), 除非 --strict
Note → A.debug + 吞掉, 除非 --verbose
```

---

## 和已有文档的咬合

```
本文                                    ADR 014——oi 三层严重度
docs/design/006-error-handling-oi.md     ADR 006——oi 错误处理系统（本文升级）
docs/design/013-pipeline-hooks.md        ADR 013——Hook 可通过 oi? 提供非致命反馈
src/oi.rs                                oi! / oi? / oi~ 宏实现
src/error.rs                             AnimiError —— 新增 severity 字段
src/rule.rs                              static_safety hook —— 两条规则降级
GOVERNANCE-FEELINGS.md                   犟种协议——"系统不替你活，系统在你没准备好之前替你守着"
```

---

*oi 不是关门。oi 是有人站在悬崖边，你在后面喊了一声。他听见了。然后他自己决定要不要回头。*
