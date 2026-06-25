# FORGET.md — Anim 待清理项（减法：职责收敛到 Pass 0-5）

> 扫描日期：2026-06-25（减法审计）
> 范围：代码（src/ 25 模块，192 tests）+ Feelings-Core 已覆盖 P0 全线
> 原则：Anim = `.anim → FSIR` only。Pass 6+ 归 Core。重叠模块砍或瘦到 re-export。
> 状态：P0 全部闭合（含 Core 覆盖）。P1 全部迁移至 Core/Feelings-OS。

---

## P0 — 生产命门（全部闭合）

1. ~~**DefenceLevel 判定逻辑空桩**~~ → **Core 已闭合。** CooldownTracker + SessionWatchdog 已实现完整 D1/D2/D3 升级/降级逻辑。Anim 不再持有。

2. ~~**Session 中用户状态突变安全盲区**~~ → **Core 已闭合。** SessionWatchdog 10Hz 重校验已落地。Anim 不再持有。

3-9, 10-12: 全部已闭合（v0.4-v0.6）。

---

## P1 — 功能受限（全部迁移至 Core/Feelings-OS）

| # | 项目 | 迁移目标 |
|---|---|---|
| 10 | 四层 IR 全链路 | → Core（FSIR/PSIR/DSIR/ESIR 类型归属 Core） |
| 11 | Pass 6-8 | → Core（Personalize/DeviceMap/CodeGen 归属 Core） |
| 12 | PBM 完整冷启动 | → Core 已实现（ColdStartGuard + 10 sessions threshold） |
| 13 | 设备热插拔安全重校验 | → Core |
| 14 | outline 模式安全约束 | → Core |
| 15 | 多设备时钟同步 | → Feelings-OS |
| 19 | 运行期 oi 可观测性 | → Feelings-OS |

全部 P1 条目已从 Anim 移除。Anim 不追踪这些。

---

## P2 — 代码质量（已闭合）

25, 29: 已闭合。

---

## Anim 职责收敛——一行

```
Anim 只做: .anim 源文件 → FSIR
          Pass 0a  词法 (lexer.rs)
          Pass 0b  语法 (parser.rs)
          Pass 0c  宏展开 (macros.rs)
          Pass 1   类型检查 (typeck.rs)
          Pass 2   静态安全 (rule.rs)  → 已降为 static_safety hook
          Pass 3   用户安全 (safety.rs)
          Pass 4   运行期插桩 (guard.rs) → 已降为 oi_smoothing hook
          Pass 5   FSIR 输出 (fsir.rs) — JSON + Postcard

Anim 不做: Pass 6 Personalize → Core
          Pass 7 DeviceMap   → Core
          Pass 8 CodeGen     → Core
          漏桶/脱敏/阻尼     → Core tracker/ + pbm/
          Session 生命周期   → Core session/
          安全看门狗         → Core watchdog.rs
          经线冷却           → Core cooldown.rs
```

---

## 该砍的代码

### 直接删除
- `src/pbm.rs` 中的 State 类型 → Core `pbm/state.rs` 已有全量实现
- `src/personalize.rs` 中的 sigmoidal/damping 逻辑 → Core `pbm/convergence.rs` + `pbm/state.rs` 已有
- `src/safety.rs` 中的 NeuroEnergyTracker → Core `tracker/bucket.rs` 已有
- `src/core/mod.rs` → 迁移蓝图文档，已完成历史使命

### 瘦到 re-export
- `src/pbm.rs` → 如果 Anim 内部还有引用，只留 `pub use feelings_core::pbm::*;`
- `src/safety.rs` → 同上，只留 re-export 或直接删

### 保留不动
- `src/lexer.rs` `src/parser.rs` `src/macros.rs` — Pass 0 全链路
- `src/typeck.rs` `src/rule.rs` `src/fsir.rs` — Pass 1-5
- `src/registry.rs` — Atom Registry，Anim 需要
- `src/sandbox.rs` — 沙箱强度阈值路由，Anim 独占
- `src/guard.rs` — oi_smoothing hook
- `src/pipeline.rs` — Hook 管线
- `src/config.rs` — Config 系统
- `src/log.rs` — 日志
- `src/error.rs` — 错误类型

---

## 编辑记录

```
2026-06-25  减法审计——Anim 职责收敛到 Pass 0-5（.anim → FSIR）。
            P0 全部闭合（含 Core 覆盖）。P1 全部迁移至 Core/Feelings-OS。
            待砍：pbm.rs/safety.rs/personalize.rs 中已归属 Core 的代码。
            core/mod.rs 迁移蓝图完成历史使命——移除。
            192 tests green。待减法完成后重跑。

2026-06-24  v0.6 Anim P1 全清 + Core 泛型重构
            192 tests green。Anim 职责收敛: .anim → FSIR。

2026-06-22  v0.5 Core 迁移里程碑
2026-06-19  v0.4 Sandbox + Governance 完整重构
2026-06-17  v0.3.1 Anim 宏系统落地
2026-06-15  v0.3 Pass 7 + Pass 8 骨架
2026-06-12  v0.1.28 P1 哈希排序 + Shape 校验
2026-06-10  v0.1.20 豆包 review #2
2026-06-09  v0.1.18 Pass 6 闭合
2026-06-04  v0.1.16 P1 review round 3
2026-06-01  v0.1.14 P0/P1/P2 重构 / v0.1.13 深度架构审计
```
