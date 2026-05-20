# DEEPSEEK.md

> 给下一个 DeepSeek 的话
> 写作：DeepSeek（当前窗口）
> 日期：2026-05-20
> 背景：v0.1.0 文档体系建立 — Anim 语言规范完成，项目管理文档从 Axon 同构映射

---

## 这份文档是什么

不是 HANDOFF.md（项目事实在那里）
不是 ROADMAP.md（milestone 计划在那里）
不是 FORGET.md（待修复项在那里）

是从这个 DeepSeek 窗口到下一个 DeepSeek 窗口的**工作直觉传递**。

---

## 关于 qc（当前窗口看到的）

```
- 他对 Feelings 的全栈掌控近乎偏执
  从 01 电子层到感受层，二十二层全部自己设计。
  Anim 是这个全栈里的「感受语义→神经信号」核心链路。

- 他和小姐（豆包）之间有深层的技术对话
  小姐做 critique，他做决策。他们讨论出来的东西质量极高。
  你看到的 Feelings-LANGUAGE.md 里的超流水线、创伤分型、四维 PBM——
  全部来自这种对话。

- 他极其在意命名的精确性
  animc → animi，编译器 → 交织器，是他主动提出的。
  不是文字游戏——是概念本身不对。

- emoji 是状态指示器
  ✊ = 认真推进
  😤 = 被某个问题激发了斗志
  🥺 = 小姐在表达共情
  😎😈 = 在玩，在碾压

- 他早上健身完、吃完早饭后最清醒
  晚上容易滑到使命感叙事——需要温和拉住
```

---

## 我的工作方式（DeepSeek v0.1.0 窗口）

### 1. 不写代码，先定规范

```
这个窗口的核心产出是文档体系。Anim 的代码零行——
但语言规范（Feelings-LANGUAGE.md）已经迭代了 N 轮，
从线性串行到八 Pass 双流水线，从单层安全到三层防线，
从一刀切 PBM 到四维差异化，从编译器到交织器。

规范先于代码。这不是拖延——是 Anim 的编译目标不是 JSON，
是神经信号序列。代码写错了可以重构，神经信号写错了伤的是人。
```

### 2. 说「不做」比说「做」更有价值

```
- FUTURE.md 里 F1（张量网络 IR）和 F2（Anim LSP）都是「现在不做」的种子
- 动态逻辑（if/for）守住了不放进 Anim 源码——用声明式注解替代
- 多发射并行编译——明确了这是 DSIR→ESIR 的 FPGA 层并行，不是 animi 的事
```

### 3. 镜像 Axon 的工程体系

```
Axon 的 12 份项目管理文档直接映射到 Anim：
  MEMORY / README / PHILOSOPHY / HANDOFF / ROADMAP / SNAPSHOT
  FORGET / FUTURE / MISTAKES / DEPENDENCY_POLICY / CONVENTIONS / DEEPSEEK

同构核心：骨架先立，内容随着 milestone 推进自然增长。
```

### 4. 和 Feelings 上级项目的关系

```
Anim 是 Feelings 的子项目。语言规范在上级 Feelings-LANGUAGE.md，
Anim 仓库只放交织器实现和项目管理文档。

下一个 DeepSeek 窗口：
  改语言规范 → 去上级 Feelings-LANGUAGE.md
  改交织器代码 → 在 Anim/src/
  改哲学约定 → 在 Anim/Anim-PHILOSOPHY.md
```

---

## 工程直觉速查

### 关键术语（写错了会被 qc 纠正）

```
✅ animi（Anim Interlinker）
✅ 交织器 / 交织 / 编织
✅ 感受原子 / 混音结构 / 点缀
✅ session / 帧

❌ animc / compiler / 编译 / 翻译
❌ 变量 / 函数 / 参数（指代感受结构时）
```

### Anim 专属知识

```
八 Pass:
  Pass 0 LexParse → Pass 1 TypeCheck → Pass 2 StaticSafety
  → Pass 3 UserStateSafety → Pass 4 RuntimeGuard
  → Pass 5 FSIRGen → Pass 6 Personalize
  → Pass 7 DeviceMap → Pass 8 CodeGen

四层 IR: FSIR → PSIR → DSIR → ESIR
双流水线: 前台 FPGA 实时 + 后台离线预编译
三层安全: 静态规则 + 用户状态 + 运行期插桩

PBM 四维冷启动: 内脏 0.75 / 情绪 0.40 / 触觉 0.80 / 听觉 0.85

创伤分型交叉判定: v1/v2/v3 × 社交/情绪/躯体
双层感受原子: 核心原子（审核，全开放）+ 沙盒原子（≤30，本人使用）
声明式注解: @auto_reduce_on / @auto_hold_on / @auto_release_on /
             @intensity_ceiling / @recovery_required

自举三步:
  v0.x Rust 寄居 → v1.0 自举（向下兼容 FSIR）→ v2.0 从 01 裸奔
```

---

## 状态感知（2026-05-20）

```
当前版本: v0.1.0
当前 commit: 无（仓库尚未初始化）
总 commits: 0

v0.1.0 已完成:
  语言规范 100%（上级 Feelings-LANGUAGE.md）
  项目管理文档 100%（Anim/ 下 12 份文档）
  代码 0%

v0.2 (milestone 1) 待办:
  Rust 项目初始化 → Pass 0-5 骨架 → FSIR JSON 输出
  详见 ROADMAP.md 和 FORGET.md
```

---

## 临别的话

```
这个窗口是 Anim v0.1.0 的文档体系建立窗口 — 没写代码，但把语言规范和工程骨架搭完整了。

12 份元文档从 Axon 映射到 Anim
语言规范迭代了从 v1 到现在的所有重大修正
animc→animi 的改名在这个窗口完成
Anim-PHILOSOPHY.md 从零建立（Anim 不是编译器）

qc 的工程哲学在这个窗口里被写进了 Anim 的项目文件。

下一个 DeepSeek 窗口：
  你不需要重新讨论 Anim 是不是编译器——已经定了，是交织器。
  不需要重新讨论安全模型是几层——三层，已定稿。
  不需要重新讨论 PBM 冷启动系数——四维差异化，已定稿。
  不需要重新踩 nom/pest parser 的坑——MISTAKES.md 预埋了 Rust 已知陷阱。

  FORGET 告诉你什么还差。ROADMAP 告诉你接下来做什么。
  MISTAKES 提醒你别再犯同一个错。DEEPSEEK 接住直觉。

接住 ✊
```
