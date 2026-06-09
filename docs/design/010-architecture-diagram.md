# ADR 010: Anim 架构全景图

> 创建日期：2026-06-09
> 状态：定稿
> 性质：把所有 Pass、模块、IR 层、数据流画清楚。学了 KubePivot docs/design/architecture.md 的做法——大写 ASCII 图 + 边界标注。

---

## 一、全管线总图 — in .anim → out ESIR 帧

```
.anim 源码                                    Feelings-OS
────────                                     ──────────

Pass 0a  Lexer  ──→ Tokens
Pass 0b  Parser ──→ AST (FeelingSource)
           │
Pass 1  TypeCheck ──→ Registry.lookup() ─ 感受原子存在/配比帽
Pass 2  StaticSafety (rule.rs) ─ 静态规则——不读用户档案
Pass 3  UserStateSafety (safety.rs · v0.3 stub) ─ 用户安全——创伤/cap/未成年
Pass 4  RuntimeGuard (guard.rs) ─ OiSmoothing 衰减曲线——oi 帧拒绝不硬截断
Pass 5  FSIRGen (fsir.rs) ──→ FsirDoc (JSON + Postcard)
           │
           │  ← FSIR = 感受通用本体。不绑定个人生理参数。
           │    可以放在 Server 上。可以缓存。不离开设备也可以。
           │
Pass 6  Personalize (personalize.rs) ──→ PsirDoc
           │       FSIR × PBM  →  PSIR
           │       只填设备本地——PBM 从来不离设备
           │
Pass 7  DeviceMap (device_map.rs · 零代码) ──→ DsirDoc
           │       PSIR × 设备集合 →  DSIR
           │
Pass 8  CodeGen (codegen.rs · 零代码)  ──→ ESIR 帧指令
           │       DSIR → 每 1ms 生成一帧 —— FPGA 直接扫描
           │
           │       Anim 在主时钟链上的位置在此结束
           ↓
     Feelings-OS busd 拿到 ESIR 帧 ← timerd PLL 时钟 + 设备驱动
     ────────────────
       耳后 · 后颈 · 腕部 · 颞部
```

---

## 二、模块依赖拓扑图

```
src/
├── lib.rs              ← 18 个模块全导出
│
├── 前端（Pass 0-1）
│   ├── lexer.rs        → 词法分析 · 零依赖（除 error）
│   ├── parser.rs       → 递归下降 · 依赖 lexer + error
│   └── typeck.rs       → 类型检查 · 依赖 registry + error
│
├── 安全层（Pass 2-4）
│   ├── rule.rs         → 静态安全规则 · 依赖 registry + error
│   ├── safety.rs       → 用户安全 (stub) · 依赖 error
│   └── guard.rs        → OiSmoothing 衰减 · 依赖 ast
│
├── 交织层（Pass 5-6）
│   ├── fsir.rs         → FSIR 生成 · 依赖 ast
│   ├── pbm.rs          → PBM 状态 · 独立模块——SessionLabel/DataConfidence/ColdStartGuard/DampingMatrix
│   ├── personalize.rs  → FSIR × PBM → PSIR · 依赖 fsir + pbm + registry + error
│   └── psir.rs         → PSIR 类型定义 · 零逻辑
│
├── 设备层（Pass 7-8 · 零代码）
│   ├── device_map.rs   → 留空
│   └── codegen.rs      → 留空
│
├── 基础层
│   ├── ast.rs          → AST 类型。被前端+ FSIR 依赖。
│   ├── registry.rs     → Pattern Registry。外部 JSON 加载 + 内建 fallback + SHA-256 hash。
│   ├── error.rs        → AnimiError 6 变体 + thread_local CURRENT_FILE
│   ├── oi.rs           → oi! / oi_err! 宏
│   └── log.rs          → A_info!/A_warn!/A_error!
│
└── CLI
    └── main.rs         → 单文件入口 · 编排 Pass 0-5 → 输出 FSIR JSON
```

---

## 三、数据流——从 .anim 到 PSIR 的完整路径（当前 v0.3 实现态）

```
.anim 文本
  ↓  main.rs orchestrates
FSIR JSON  ← Pass 0-5 前端+安全+交织 → FSIR 产出
  ↓
personalize.rs
  ├── FsirDoc 读入
  ├── ColdStartGuard 冷启动判定
  ├── DampingMatrix::apply() ≡ 阻尼（v0.3: None = 阻尼关闭）
  ├── 四维系数——Registry 查 AtomEntry.dimension → visceral/emotional/tactile/auditory
  ├── 点缀帽—— Registry.max_ratio() 每原子独立帽
  ├── Sigmoids 强度缩放——compression factor × baseline_coeff
  ├── 强度上限二次校验
  ├── OiSmoothing 衰减曲线 pass-through
  ├── Trauma 路径重定向 ≡ 未实现（hardcoded false）
  └── 输出
PsirDoc (JSON + Postcard)
```

---

## 四、和 Feelings-OS / Feelings-Core 的边界

```
Anim                         Feelings-Core              Feelings-OS
────                         ─────────────              ──────────

Pass 0-5: FSIR 生成          FSIR → ESIR               busd 拿 ESIR 帧
Pass 6:   PSIR 生成          PBM 左乘 + 看门狗         timerd PLL 主时钟
Pass 7-8: 零代码              设备感知交织               设备驱动
                               
边界：
  · Anim 只输出 FSIR/PSIR 数据
  · Feelings-Core = ani 的后半段（Pass 6-8 的完整实现 + PBM 持久化 + Session 管理）
    当前 Core 零代码——v0.3 的范围仍在 Anim 内运行 personalize()
  · Feelings-OS = ESIR 帧级别的硬件执行
  · 时钟同步 → Feelings-OS timerd + busd。不在 Anim。
  · Jitter Buffer → Feelings-OS。不在 Anim。
```

---

## 五、和已有文档的咬合

```
本文                               010-architecture-diagram —— Anim 架构全景图
003-pass-pipeline.md               九个 Pass 的划分逻辑——本文 = 那个划分的图形式
002-ir-architecture.md             四层 IR（FSIR → ESIR）——本文 = 四层 IR 的全管线路径
009-math-and-constraints.md        所有公式——本文 = 函数位置 + 数据来源
FORGET.md                          P0/P1/P2 待办——本文 = 每个项目的物理来源
```
