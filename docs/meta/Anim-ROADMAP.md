# Anim ROADMAP

> 创建日期：2026-05-20
> 最后更新：2026-06-17
> 当前版本：v1.0（已打 tag——自举前唯一 tag。157 tests，Pass 0-8 全链路骨架完成。24 模块。15 ADR。）
> 原则：语言规范先于代码。模糊想法 → [FUTURE.md](Anim-FUTURE.md)

---

## 当前状态 (2026-06-17)

### ✅ Pass 0-8 — 全部完成（含骨架）

| Pass | 名称 | 文件 | 状态 |
|------|------|------|:---:|
| 0a | LexParse (词法) | `lexer.rs` | ✅ |
| 0b | LexParse (语法) | `parser.rs` | ✅ |
| expand | MacroExpand | `macros.rs` | ✅ — macro_rules! 源码级展开（ADR 005） |
| 1 | TypeCheck | `typeck.rs` | ✅ — Registry 校验 + max_ratio + 建议算法 |
| 2 | StaticSafety | `rule.rs` | ✅ — 全局上限 / abrupt_stop / 沙箱禁主。已降为 `static_safety` hook |
| 3 | UserSafety | `safety.rs` | ✅ — 强度缩放 + low_anchor_cap + NeuroEnergyTracker 漏桶 + 非线性泄漏 + 不应期 + 跨维度 + 全局桶 |
| 4 | RuntimeGuard | `guard.rs` | ✅ — OiSmoothing 衰减曲线，已降为 `oi_smoothing` hook |
| 5 | FSIR Gen | `fsir.rs` | ✅ — JSON + Postcard 双格式，源码双哈希 |
| 6 | Personalize | `personalize.rs` | ✅ — FSIR × PBM → PSIR。sigmoidal 缩放 + DampingMatrix + ColdStartGuard + DefenceLevel |
| 7 | DeviceMap | `device_map.rs` | ✅ — PSIR × DeviceSet → DSIR 骨架（ear only v0.3） |
| 8 | CodeGen | `codegen.rs` | ✅ — DSIR → ESIR 帧序列（6 种 shape + Postcard 二进制，ear only v0.3） |

**总代码: 24 模块。157 个测试，0 fail。** clippy 零 warning。15 ADR。

**PBM 全链路接入** (`pbm.rs` + `personalize.rs` + `safety.rs`):
- SessionLabel (Normal/Abnormal/ColdStart) → PbmUpdateStrategy
- DataConfidence (High/Low/Contaminated) → step multipliers（从 config 读取）
- ColdStartGuard → from_config(&ColdStartConfig)
- DampingMatrix → gradient_threshold 从 config 读取
- DampingState → from_config(&DampingConfig)
- DefenceLevel D1/D2/D3 枚举就绪，判定逻辑空桩

**基础设施**:
- YAML 配置系统 — `configs/default.yaml`（67 参数 10 段）+ `src/config.rs` + `--config`
- Pipeline + Hook 架构 — `src/pipeline.rs`（7 hooks）— ADR 013
- 宏系统 — `src/macros.rs` — ADR 005
- oi 三层严重度 — oi!/oi_warn!/oi_note! + `--strict`/`--verbose` — ADR 014
- Postcard 二进制 ABI（FSIR / ESIR 双格式）
- Registry 外部化 + 哈希排序 + 科学计数法浮点字面量
- 源码 SHA-256 — SPL 锚定就绪
- 错误码自动生成 — build.rs → docs/error-codes.md

**示例文件** (`eg/`): 4 份 .anim + macro-calm.anim + 1 份 registry.json (8 个核心原子)

---

### 📋 FORGET 摘要

**P0 — 生产命门 (11/12):** 仅剩 DefenceLevel 判定逻辑空桩（等 Core PBM 真实生理数据）。

**P1 — 功能受限 (13/20):** Pass 7-8 需 v0.4 扩展、PBM 冷启动系数收敛待 v0.3、宏系统递归组合风险待评估、设备热插拔/outline/缓存失效/oi 可观测性待设计。

---

## 下一步: v0.4 — 多设备 + 自适应帧密度 + API 模式设计

### 目标

```
FSIR → ESIR 从 ear-only 骨架扩展到完整五设备协同。
Anim 不再只是 CLI 工具——增加 API 模式，直接接收 Core PBM 的实时数据注入。
```

### 核心交付

- **DSIR 多设备降级** — wrist/neck/temple 缺失时的 outline 降级策略（当前 ear-only 硬线）
- **ESIR 自适应帧密度** — rise 段 1ms、plateau 段 10ms、安全校验帧不受影响
- **Anim API 模式** — `animi --api <session-id>` — Pass 0-5 旁路，Core 直接喂 FSIR/PSIR。后半段管线（Pass 6-8）不变。
- **多设备帧率差异** — ear 1ms / wrist 8ms (BLE) / neck 2ms / temple 4ms

---

## 输入模式的阶段演进

### 阶段一（当前）— 人为输入 + 设备采集

```
创作者手写 .anim 源码。
Anim CLI 编译 → FSIR JSON。
设备后台采集 PBM — 心率、皮电、HRV — 为教练 AI 熟悉用户。
config 参数 = 设计估值 → 随着 PBM 数据累积逐步取代。
```

### 阶段二（v0.6+）— 自动输入

```
Core PBM 已知用户基线。教练 AI 了解用户的感受空白和生理特征。
不再需要 .anim 文件。
Core 生成 FSIR（或已校准后的 PSIR）直接喂给 Anim —
Anim 从 API 接收，不再是文件输入。
后半段管线（Pass 6-8）不变。
```

### 演进逻辑

```
早期 → 人工输入 .anim，Anim CLI 是入口，同时设备采集 PBM。
中期 → Core PBM 数据覆盖设计估值，config 参数从"猜"变为"用"。
成熟 → Core 自动生成感受请求，Anim 作为 API 服务运行，无需人工介入。
```

---

## 远期里程碑

```
v0.4   多设备 + 自适应帧密度 + 离线预交织 — 后台离线 FSIR 缓存, Session 启动延迟 <100ms
v0.5   实时交织 + FPGA 对接 — ESIR → 硬件数据包, 闭环偏差修正, 紧急冲刷
v0.6   API 模式 — Core 驱动 Anim。不再需要 .anim 文件。全自动感受生成。
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
