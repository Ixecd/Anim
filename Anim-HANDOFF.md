# HANDOFF — Anim v0.1.0

> 编写日期：2026-05-20
> Last release: v0.1.0（语言规范阶段）
> Total commits: 0
> Co-Authored-By: DeepSeek

---

## 一、项目定位

**Anim** 是 Feelings 生态的交织语言。animi（Anim Interlinker）是把 .anim 源码织成神经信号序列的工具。

**核心理念**：不是编译器——是编织者。多股独立流（感受语义、个人基线、设备约束、生理反馈、安全边界）织成一条连续的信号绳。

**技术选型**：
- **Rust** — v0.x 寄居宿主。零成本抽象，无 GC，和安全文化不打架
- **八 Pass 双流水线** — 前台 FPGA 实时 + 后台离线预编译
- **四层 IR** — FSIR → PSIR → DSIR → ESIR
- **上级项目** — Feelings 定义语言规范（`Feelings-LANGUAGE.md`），Anim 实现交织器

---

## 二、现在能做什么

### 语言规范

- `Feelings-LANGUAGE.md` 已完成——Anim 语言完整定义
- 类型系统、八 Pass、四层 IR、双流水线、安全模型全部定稿
- 双层感受原子体系、声明式动态注解已规范

### 代码

- **零行。** 仓库尚未初始化。

---

## 三、开发约束

### 设计先行

新 Pass / 新 IR 层：先补 `Feelings-LANGUAGE.md` 规范 → 再写 `docs/design/<feature>.md` → 拍板 → 实施。

### Rust 约定

- `trait` 注入 mock（不用全局 var）
- 热路径禁止 unwrap/expect（走 `?` + `AppError`）
- Rule of Three — 3+ 处重复 → trait，1-2 处保留 concrete

### 依赖审查

新增 crate → 对照 DEPENDENCY_POLICY.md 三级判定。

### 错误记录

重复 ≥ 2 次的错误 → MISTAKES.md。首次犯 → 观察。再犯 → 入册（根因+解法+重复计数）。

---

## 四、当前工作流

```bash
# 尚未初始化仓库。预计流程：
cargo new anim
make dev          # cargo build + cargo test --lib + cargo clippy + cargo fmt --check
make test         # cargo test
make lint         # cargo clippy -- -D warnings + cargo fmt --check
```

---

## 五、文档地图

```
Anim/
  Anim-MEMORY.md (AI 搭档入口索引 — 第一时间读这个)
  Anim-HANDOFF.md / Anim-ROADMAP.md / Anim-SNAPSHOT.md / Anim-FORGET.md
  Anim-FUTURE.md / Anim-DEPENDENCY_POLICY.md / Anim-MISTAKES.md
  Anim-PHILOSOPHY.md / Anim-README.md

上级项目（Feelings）：
  ../Feelings-LANGUAGE.md  — Anim 语言规范
  ../Feelings-MATRIX.md    — 万物皆矩阵
  ../Feelings-FULLSTACK.md — 二十二层全栈架构
  ../docs/ten-bits.md      — 10 bit/s 意识瓶颈
```

---

## 六、联系

```
作者:    qc (Ixecd)
许可:    MIT
上层:    Feelings 项目 — github.com/Ixecd/Feelings
```
