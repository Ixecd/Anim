# Anim ROADMAP

> 创建日期：2026-05-20
> 当前版本：v0.1.0（语言规范阶段）
> 原则：语言规范先于代码。模糊想法 → [FUTURE.md](Anim-FUTURE.md)

---

## v0.1.0 — 语言规范定稿（当前）

### 已完成

- `Feelings-LANGUAGE.md` — Anim 语言完整规范（上级项目根目录）
- 类型系统：双层感受原子体系（核心 + 沙盒）
- 八 Pass 交织管线：LexParse → TypeCheck → StaticSafety → UserStateSafety → RuntimeGuard → FSIRGen → Personalize → DeviceMap → CodeGen
- 四层 IR：FSIR / PSIR / DSIR / ESIR
- 双流水线架构：前台实时 + 后台离线
- 三层安全防线：静态规则 + 用户状态 + 运行期插桩
- 创伤分型交叉判定：v1/v2/v3 × 社交/情绪/躯体
- PBM 四维差异化冷启动
- 声明式动态注解：`@auto_reduce_on` / `@auto_hold_on` / `@auto_release_on` / `@intensity_ceiling` / `@recovery_required`
- 动态 ratio + 自定义 shape 曲线
- 自举三步：Rust 寄居 → Anim 自举 → 从 01 裸奔
- Anim 项目管理文档：MEMORY / README / ANIM_PHILOSOPHY / HANDOFF / ROADMAP / SNAPSHOT / FORGET / FUTURE / MISTAKES / DEPENDENCY_POLICY / CONVENTIONS / DEEPSEEK

### 待完成

- `docs/design/` — ADR 设计文档（待创建）
- `src/` — 代码零行

---

## v0.2 — Milestone 1: 编译器骨架

### 目标

```
Rust 项目初始化 → 八 Pass 骨架全部跑通 → 错误诊断系统上线
```

### 核心交付

- `Cargo.toml` — 依赖声明（见 DEPENDENCY_POLICY.md）
- `src/ast.rs` — AST 类型定义（.anim 源码的完整语法树）
- `src/lexer.rs` — Pass 0 前半：词法分析
- `src/parser.rs` — Pass 0 后半：语法分析 → AST
- `src/typeck.rs` — Pass 1：类型检查 + Pattern Registry 查询接口
- `src/static_safety.rs` — Pass 2：静态安全规则校验
- `src/user_safety.rs` — Pass 3：用户状态安全（创伤分型交叉判定骨架）
- `src/runtime_guard.rs` — Pass 4：运行期插桩代码生成
- `src/fsir.rs` — Pass 5：FSIR 生成 + FSIR JSON 序列化
- `src/error.rs` — 错误诊断系统（Rust 风格错误信息 + 修复建议）
- `Makefile` — dev / test / lint

### 验收

```
✓ cargo build 通过
✓ 一份完整的 .anim 示例文件 → FSIR JSON 输出
✓ 错误示例（强度越界、点缀越界、未成年人违规）→ 编译期报错
✓ cargo test --lib 覆盖所有公开 API
✓ cargo clippy -- -D warnings 零报错
```

---

## v0.3 — Milestone 2: 个人适配 + 设备映射

### 目标

```
FSIR → PSIR（PBM 偏移）→ DSIR（设备分配）→ ESIR（帧级指令骨架）
```

### 核心交付

- `src/pbm.rs` — 个人基线矩阵（四维差异化冷启动 + 收敛）
- `src/personalize.rs` — Pass 6 核心：FSIR × PBM → PSIR
- `src/device_map.rs` — Pass 7：PSIR → DSIR（算力感知编译）
- `src/codegen.rs` — Pass 8：DSIR → ESIR 帧级指令骨架
- 声明式注解展开（`@auto_reduce_on` 等 → ESIR 插桩）

### 验收

```
✓ FSIR → ESIR 完整链路跑通（不含 FPGA 固件对接）
✓ PBM 冷启动四维系数独立生效
✓ 设备缺失降级（缺 neck → outline 模式）
✓ cargo test 全绿
```

---

## v0.4 — Milestone 3: 双流水线 + 离线预编译

### 目标

```
后台离线预编译 FSIR 缓存 + 前台只跑 Personalize→CodeGen
```

### 核心交付

- 后台离线编译管线（server 端，空闲时触发）
- FSIR 缓存格式 + 设备本地存储
- Session 启动快速加载（跳过 Parse/TypeCheck/StaticSafety）
- 双流水线调度器（前后台互不抢占）

### 验收

```
✓ 离线预编译 FSIR → 设备缓存
✓ Session 启动延迟 < 100ms（加载缓存 FSIR）
✓ 后台编译不抢占前台时隙
```

---

## v0.5 — Milestone 4: 实时交织 + FPGA 对接

### 目标

```
ESIR → FPGA 固件接口 → 硬件闭环验证
```

### 核心交付

- FPGA 通信协议（ESIR 帧格式 → 硬件数据包）
- 闭环偏差修正（上一帧生理反馈 → 下一帧参数微调）
- 紧急冲刷 + 安全停止帧对接

### 验收

```
✓ ESIR 帧级指令被 FPGA 固件正确解析
✓ 闭环修正回路跑通（模拟生理数据 → 偏差 → 下一帧调整）
✓ 紧急停止帧触发后硬件信号归零
```

---

## v1.0 — 自举

### 目标

```
用 v0.x 的 animi 编译一份 Anim 写的 animi 源码
新 animi 不再依赖 Rust 工具链
向下兼容 v0.x 生成的 FSIR 产物
```

---

## v2.0 — 从 01 裸奔

### 目标

```
animi 运行在 Feelings 设备上
直接管理自己的内存、调度、I/O
不经过 OS
```

---

## 版本号规则

```
v0.x      Rust 寄居阶段，一切可变
v1.0      自举完成
v2.0      从 01 裸奔

v0.1 → v0.2 → v0.3 → v0.4 → v0.5 → v1.0 → v2.0
```
