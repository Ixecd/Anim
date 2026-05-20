# MEMORY.md — Anim 项目入口索引

> 这是 AI 搭档打开项目后**第一时间读的文件**。
> 读完此文件 → 按文档地图索引到具体文档 → 建立完整项目心智模型。
> 最后更新：2026-05-20 | v0.1.0

---

## 一、项目身份（30 秒速览）

```
Anim   = 从 01 到感受的交织语言
animi  = Anim Interlinker（交织器）——不是编译器，是编织者
核心理念：多股独立流（感受语义、个人基线、设备约束、生理反馈、安全边界）
         织成一条连续的信号绳
技术栈：  Rust（v0.x 寄居）→ Anim 自举（v1.0）→ 从 01 裸奔（v2.0）
当前版本：v0.1.0 — 语言规范完成，交织器代码零行
上级项目：Feelings — 神经感受民主化平台
```

---

## 二、文档地图（按阅读顺序）

### 第一圈：核心理解（必读，按此顺序）

| 序号 | 文件 | 回答的问题 | 读完时长 |
|------|------|-----------|----------|
| 1 | `MEMORY.md` | 我在哪？这个项目是什么？文档怎么找？ | 你正在读 |
| 2 | [README.md](./Anim-README.md) | 项目对外介绍，定位和价值主张 | 3 min |
| 3 | [Anim-PHILOSOPHY.md](./Anim-PHILOSOPHY.md) | 怎么写代码？什么原则？ | 10 min |
| 4 | [HANDOFF.md](./Anim-HANDOFF.md) | 现在能做什么？开发约束？工作流？ | 5 min |
| 5 | [SNAPSHOT.md](./Anim-SNAPSHOT.md) | 精确到 commit 的代码结构和测试状态 | 5 min |

### 第二圈：工程管理（知道项目怎么管）

| 文件 | 回答的问题 |
|------|-----------|
| [ROADMAP.md](./Anim-ROADMAP.md) | 各个 milestone 分别交付什么？ |
| [FORGET.md](./Anim-FORGET.md) | 什么东西还没做？（P0 命门 / P1 受限） |
| [FUTURE.md](./Anim-FUTURE.md) | 什么种子现在不做、以后可能做？ |
| [DEPENDENCY_POLICY.md](./Anim-DEPENDENCY_POLICY.md) | 新 crate 能引入吗？三级判定怎么走？ |
| [MISTAKES.md](./Anim-MISTAKES.md) | 哪些坑踩过 ≥2 次？认知盲区在哪里？ |

### 第三圈：设计决策（理解为什么这么做）

| 文件 | 回答的问题 |
|------|-----------|
| 上级 `Feelings-LANGUAGE.md` | Anim 语言完整规范——类型系统、IR 管线、安全模型 |
| 上级 `docs/ten-bits.md` | 10 bit/s 瓶颈——Anim 的神经科学基础 |
| 上级 `Feelings-MATRIX.md` | 万物皆矩阵——Anim 的数学哲学基底 |
| 上级 `Feelings-FULLSTACK.md` | 二十二层全栈架构——Anim 在 Feelings 全栈的位置 |

### 第四圈：AI 搭档专属

| 文件 | 回答的问题 |
|------|-----------|
| [DEEPSEEK.md](./Anim-DEEPSEEK.md) | 上一个 DeepSeek 窗口留下了什么直觉？qc 是什么风格？ |

---

## 三、架构速览（一张图理解项目）

```
.anim 源码（感受结构声明）
    │
    ▼
┌──────────────────────────────────────────┐
│              animi（交织器）               │
│                                          │
│  Pass 0: LexParse      词法语法解析       │
│  Pass 1: TypeCheck      类型校验          │
│  Pass 2: StaticSafety   静态安全规则       │
│  Pass 3: UserStateSafety 用户状态安全      │
│  Pass 4: RuntimeGuard    运行期插桩生成    │
│  Pass 5: FSIRGen        感受结构 IR       │
│  Pass 6: Personalize    个人基线适配       │
│  Pass 7: DeviceMap      设备信号分配       │
│  Pass 8: CodeGen        帧级指令生成       │
│                                          │
│  双流水线：前台 FPGA 实时 + 后台离线预编译   │
│  四层 IR：FSIR → PSIR → DSIR → ESIR       │
└──────────────────────────────────────────┘
    │
    ▼
ESIR 帧级指令 → FPGA/RTOS → 设备 → 神经通路 → 感受
```

**animi 不是编译器——是交织器。** 每层织入一股新东西：语义、基线、设备、反馈、安全。

---

## 四、代码结构速查（v0.1.0 预计）

```
Anim/
├── Cargo.toml              Rust 项目配置
├── Makefile                 构建脚本
│
├── src/
│   ├── main.rs              animi 入口
│   ├── lib.rs               pub mod lexer/parser/typeck/...
│   ├── lexer.rs             Pass 0: 词法分析
│   ├── parser.rs            Pass 0: 语法分析 → AST
│   ├── ast.rs               AST 类型定义
│   ├── typeck.rs            Pass 1: 类型检查
│   ├── static_safety.rs     Pass 2: 静态安全规则
│   ├── user_safety.rs       Pass 3: 用户状态安全
│   ├── runtime_guard.rs     Pass 4: 运行期插桩生成
│   ├── fsir.rs              Pass 5: FSIR 生成
│   ├── personalize.rs       Pass 6: PSIR 生成（PBM 偏移）
│   ├── device_map.rs        Pass 7: DSIR 生成
│   ├── codegen.rs           Pass 8: ESIR 生成
│   └── pbm.rs               个人基线矩阵
│
├── docs/
│   └── design/              设计文档（ADR）
│
├── tests/                   集成测试
├── benches/                 性能基准
│
├── MEMORY.md                项目入口索引
├── Anim-PHILOSOPHY.md       开发哲学
├── CONVENTIONS.md           代码规范
├── MISTAKES.md              错误日志
├── FORGET.md                P0+P1 待修复
├── DEPENDENCY_POLICY.md     依赖管理
├── FUTURE.md                架构种子库
├── ROADMAP.md               milestone 计划
├── HANDOFF.md               接手指南
├── DEEPSEEK.md              DeepSeek 窗口直觉传递
├── SNAPSHOT.md              代码快照
└── README.md
```

---

## 五、关键约定（踩坑前必看）

### commit 规范

```
git commit -F commits/<file>.txt   ← 不是 -m
格式: type(scope): description    ← 见 commits/README.md
subject 严格 ASCII
```

### Rust 常见坑

```
tokio::Mutex guard 不能跨 .await  → enum StateMachine + mem::replace
bail! 需要 use anyhow::bail;      → 别忘导入
tokio::spawn Handle 必须 await    → 否则 panic 静默吞
测试 mock 用 trait 不用全局 var    → 见 MISTAKES.md
```

### 依赖引入

```
新增 crate → 查 DEPENDENCY_POLICY.md 三级判定
L0 禁止 / L1 受限需论证 / L2 CLI wrapper 直接通过
```

### 错误记录

```
犯两次 → MISTAKES.md 入册（根因+解法+计数）
MISTAKES.md 本身就是这条规则的产品
```

---

## 六、当前状态（2026-05-20）

```
版本:    v0.1.0
提交:    0 commits（仓库尚未初始化）
分支:    无
Sprint:  语言规范定稿 → 编译器骨架

完成度:
  语言规范   ████████████████████ 100%（上级 Feelings-LANGUAGE.md）
  交织器代码 ░░░░░░░░░░░░░░░░░░░░   0%
  测试       ░░░░░░░░░░░░░░░░░░░░   0%

P0 待修复:  见 FORGET.md
```

---

## 七、外部依赖

| 系统 | 关系 | 文档 |
|------|------|------|
| Feelings | 上级项目，定义感受架构与全栈 | `../` |
| Feelings-LANGUAGE.md | Anim 语言规范 | `../Feelings-LANGUAGE.md` |
| Feelings-MATRIX.md | 万物皆矩阵——数学哲学基底 | `../Feelings-MATRIX.md` |
| docs/ten-bits.md | 10 bit/s 意识瓶颈 | `../docs/ten-bits.md` |
| Feelings-FULLSTACK.md | 二十二层全栈架构 | `../Feelings-FULLSTACK.md` |

---

## 八、读完后你应该能回答

- [ ] Anim 是什么？animi 是什么？和编译器有什么区别？
- [ ] 八 Pass 分别做什么？四层 IR 是哪四层？
- [ ] 双流水线架构是什么？前台做什么、后台做什么？
- [ ] Anim 的自举三步是什么？
- [ ] 下一个文件该读哪个？（按文档地图顺序）
