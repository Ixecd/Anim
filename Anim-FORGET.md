# FORGET.md — 待修复项（P0 + P1）

> 扫描日期：2026-05-26
> 范围：语言规范（`Feelings-LANGUAGE.md`）+ 设计文档 + 代码（零行）
> 原则：只列 P0（生产命门）和 P1（功能受限），P2 Ops / P3 Polish 不提

---

## P0 — 生产命门（交织器不存在就无法工作）— 0/5

### 交织管线

1. **八 Pass 骨架零代码** — `src/` 目录不存在。Pass 0-8 全部未实现。FIXME: v0.2 milestone 1。依赖：AST 类型定义 + parser + type checker + safety passes + IR 生成。

2. **四层 IR 零代码** — FSIR / PSIR / DSIR / ESIR 仅在规范中定义，无任何 Rust struct 实现。FIXME: v0.2 milestone 1（FSIR），v0.3 milestone 2（PSIR/DSIR/ESIR）。

### 类型系统

3. **双层感受原子体系零代码** — 核心原子（Pattern Registry 查询）和沙盒原子（unverified 标记）的区分逻辑未实现。

### 安全模型

4. **三层安全防线零代码** — StaticSafety / UserStateSafety / RuntimeGuard 均为规范描述，零实现。创伤分型交叉判定矩阵未编码。

### PBM

5. **个人基线矩阵零代码** — 四维差异化冷启动系数（内脏 0.75 / 情绪 0.40 / 触觉 0.80 / 听觉 0.85）仅在规范中，无任何实现。

---

## P1 — 功能受限（规模化前必做）— 0/6

### 交织管线

6. **双流水线调度器零代码** — 设计文档已完成（ADR 004），实现方案已定稿。代码零行。
    FIXME: v0.4 milestone 3。

7. **设备算力感知交织零代码** — 入门/标准/高端三档信号参数密度调整逻辑已在 ADR 003 Pass 7 中定稿。代码零行。

### 语法

8. **Anim 宏系统零代码** — 语法树级展开、三层保证（类型检查 + 强度生命周期 + 创伤作用域）已在 ADR 005 中定稿。`macro_rules!` 解析器 + 展开器 + 递归深度保护零代码。

9. **动态 ratio 变量绑定零实现** — `@bind(skin_conductance_trend, range(min, max))` 语法未实现到 parser/typeck。

10. **自定义 shape 曲线零实现** — `keyframes` 语法 + 插值算法未实现。

11. **声明式注解展开零代码** — 五条 `@` 注解展开为 ESIR 插桩的逻辑已在 ADR 005 中定稿。代码零行。

### 设备对接

12. **FPGA 固件接口协议零设计** — ESIR 帧格式 → 硬件数据包的通信协议未定义。

---

## 编辑记录

```
2026-05-20  v0.1 初始扫描
            - 语言规范 100%
            - 代码 0%
            - P0 5 项：八Pass / 四层IR / 原子体系 / 安全三层 / PBM
            - P1 6 项：双流水线调度 / 算力感知 / 动态ratio / 自定义shape /
                       注解展开 / FPGA协议
            - 全部 open，符合 v0.1 早期阶段预期

2026-05-26  v0.1.1 设计文档补全
            - 新增 ADR 002-005（IR架构 / Pass管线 / 双流水线 / 宏系统）
            - 新增 Anim-SAFETY.md、Anim-LOCALITY.md
            - P1 项更新：双流水线调度→设计已定稿代码零行；算力感知→设计已定稿；
              宏系统新增 P1；注解展开→设计已定稿代码零行
            - P0 不变：代码仍零行，v0.2 milestone 1 开始
```
