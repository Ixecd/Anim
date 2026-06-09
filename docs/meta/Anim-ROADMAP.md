# Anim ROADMAP

> 创建日期：2026-05-20
> 最后更新：2026-06-09
> 当前版本：v1.0（已打 tag——自举前唯一 tag。93 tests，Pass 0-5 完成，FSIR JSON + Postcard 双格式闭环）
> 原则：语言规范先于代码。模糊想法 → [FUTURE.md](Anim-FUTURE.md)

---

## 当前状态 (2026-06-09)

### ✅ Pass 0-5 — 已完成

| Pass | 名称 | 文件 | 代码行 | 测试 | 状态 |
|------|------|------|--------|------|------|
| 0a | LexParse (词法) | `lexer.rs` | 528 | 14 | ✅ |
| 0b | LexParse (语法) | `parser.rs` | 623 | 14 | ✅ |
| 1 | TypeCheck | `typeck.rs` | 403 | 17 | ✅ |
| 2 | StaticSafety | `rule.rs` | 150 | 3 | ✅ |
| 3 | UserSafety | `safety.rs` | 118 | 4 | ⚠️ STUB — check() 是 no-op |
| 4 | RuntimeGuard | `guard.rs` | 355 | 13 | ✅ — OiSmoothing 衰减曲线完整 |
| 5 | FSIR Gen | `fsir.rs` | 416 | 7 | ✅ — JSON + Postcard 双格式 |

**总代码: 3,903 行 Rust。93 个测试，0 fail。** 6 个依赖——serde、serde_json、postcard、chrono、sha2、hex。零异步运行时。

**PBM 地基已就绪** (`pbm.rs`, 485 行, 15 测试):
- SessionLabel (Normal/Abnormal/ColdStart) → PbmUpdateStrategy
- DataConfidence (High/Low/Contaminated) → step multipliers
- ColdStartGuard (前 10 次 Session 强制关闭预测器)
- DampingMatrix (情绪梯度>2.0×→冻结内脏+触觉, 内脏>1.5×→冻结情绪)
- 全部未接入管线——仅类型定义和单元测试

**示例文件** (`eg/`): 4 份 .anim + 1 份 registry.json (8 个核心原子)

---

### ❌ Pass 6-8 — 零代码

| Pass | 名称 | 描述 |
|------|------|------|
| 6 | Personalize | FSIR × PBM → PSIR — PBM 左乘, 个人基线织入 |
| 7 | DeviceMap | PSIR → DSIR — 设备感知交织, 算力分配 |
| 8 | CodeGen | DSIR → ESIR — 帧级指令生成, FPGA 对接 |

---

### 📋 FORGET 摘要

**P0 — 生产命门 (4/9 剩余):** 全部安全盲区——Pass 3 空桩修复、10Hz 安全看门狗、沙箱混合绕过、创伤绝对阈值——全部推迟到 ADR 009。

**P1 — 功能受限 (11/14 剩余):** Pass 6-8 零代码、PBM 冷启动系数+收敛未实现、宏系统零代码、设备热插拔/抖动/outline 零设计。

**P2 — 代码质量: 全部清零。**

---

## 下一步: v0.3 — Personalize + DeviceMap + CodeGen 骨架

### 目标

```
FSIR × PBM → PSIR → DSIR → ESIR 完整链路（不含 FPGA 固件对接）
```

### 核心交付

- **ADR 009** — 覆盖 P0 剩余 4 项安全盲区（10Hz 看门狗、沙箱混合、创伤绝对阈值、设备热插拔）
- **Pass 6 Personalize** — `personalize.rs`。pbm.rs 已有类型接入管线——ColdStartGuard 判定 SessionLabel → PbmUpdateStrategy 选 Full/SafetyOnly/Disabled → DampingMatrix 查冻结维度 → 生成 PSIR
- **PSIR 序列化** — JSON 调试 + Postcard 二进制
- **Pass 7 DeviceMap** — `device_map.rs`。PSIR → DSIR, 设备缺失降级 outline 模式
- **Pass 8 CodeGen** — `codegen.rs`。DSIR → ESIR 帧级指令骨架（不含 FPGA 协议对接）
- **测试** — FSIR JSON → Pass 6 → PSIR → Pass 7 → DSIR → Pass 8 → ESIR 闭环验证

### 验收

```
✓ FSIR → ESIR 完整链路跑通
✓ PBM 冷启动四维系数独立生效（内脏0.75/情绪0.40/触觉0.80/听觉0.85）
✓ ColdStartGuard + DampingMatrix 在管线中正向验证
✓ 设备缺失降级（缺 neck → outline 模式）
✓ cargo test 全绿
```

---

## 远期里程碑

```
v0.4   双流水线 + 离线预交织 — 后台离线 FSIR 缓存, Session 启动延迟 <100ms
v0.5   实时交织 + FPGA 对接 — ESIR → 硬件数据包, 闭环偏差修正, 紧急冲刷
v1.0   自举 — 用 v0.x 的 animi 编译 Anim 写的 animi, 不再依赖 Rust 工具链
v2.0   从 01 裸奔 — animi 运行在 Feelings 设备上, 不经过 OS
```

---

## 版本号规则

```
v0.x      Rust 寄居阶段，一切可变
v1.0      自举完成
v2.0      从 01 裸奔
```
