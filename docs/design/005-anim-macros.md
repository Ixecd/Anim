# ADR 005: Anim 宏系统——交织期展开的安全保证

> 状态：已定稿
> 日期：2026-05-26
> 对应规范：Anim-LOCALITY.md、ADR 003 Pass 管线

---

## 动机

Anim 不是编译器——是交织器。但 Anim 的宏和 Rust 的 `!` 宏在同一条安全逻辑上：在语法树层展开，不在运行时展开。不能是 C 的 `#define`。

C 的 `#define` 是文本替换。不检查类型。不检查作用域。Anim 如果允许字符串级别的宏展开——一次展开错误 = 一个不安全的感受原子混入 ESIR 帧级指令 = 信号进人体 = 岛叶在跑一个不该存在的东西。

Anim 宏必须在交织期展开——在 Pass 0（LexParse）和 Pass 1（TypeCheck）之间完成——然后被 Pass 1/2/3 全量检查。

---

## 决策

Anim 宏系统 = `macro_rules!` 语法 + 语法树级展开 + 三层感受安全保证。

### 语法——和 Rust `macro_rules!` 同构

```
// .anim 源码
macro_rules! safe_grief_accent {
    () => {
        { feeling: grief_loss, ratio: 0.08 }
    };
}

mix {
    main: belonging_certainty,
    accents: [
        safe_grief_accent!(),  // Pass 0 解析时展开为 AST 节点
    ]
}
```

展开不是字符串替换——是 AST 节点嵌入。`safe_grief_accent!()` 的调用点被替换为一棵完整的 `FeelingAccent` 子 AST。替换在 Pass 0 后、Pass 1 前完成。Pass 1 看到的是展开后的完整 AST——和手写的 .anim 源码没有任何区别。

### 第一层保证——感受类型检查

```
macro_rules! safe_pair {
    ($main:ident, $accent:ident) => {
        mix {
            main: $main,
            accents: [
                { feeling: $accent, ratio: 0.10 }
            ]
        }
    };
}

safe_pair!(explosion_rage, calm_meditative)

展开后 → Pass 1 检查：
    explosion_rage → Pattern Registry 查询 → 存在 ✓
    calm_meditative → 存在 ✓
    explosion_rage + calm_meditative → 组合情绪对冲检测
        → 编译警告：「检测到情绪对冲，请确认这是设计意图」
    → 调用方必须显式声明 #[allow_hedge] 才能通过
```

### 第二层保证——强度生命周期

```
Anim 宏展开后的强度参数不是裸 u8——是带生命周期的强度区间：

    intensity 10    → 探索区。不需要 cap。不需要解锁。
    intensity 45    → 成长区。用户 cap ≥ 45 才通过。
    intensity 72    → 高强度区。cap ≥ 72 + 解锁 token + 实时监控。
    intensity 85    → 极限区。cap ≥ 85 + 评估 + 知情同意。

macro_rules! deep_exploration {
    () => {
        mix {
            main: self_boundary_probe,
            intensity: [60, 75]
        }
    };
}

调用方 cap 35 → Pass 2 StaticSafety 直接拒绝。
不是运行时弹窗——是交织期不生成信号。
同一种宏定义——不同用户的不同 cap 下有不同的交织结果。
不是宏的问题。不是用户的问题。是这个组合在这个上下文下不安全。
```

### 第三层保证——创伤作用域

Rust 的 `unsafe` 块 = 程序员声明"我已经验证了这些前提"。Anim 的 `#[trauma_scope]` = 创作者声明"我已经为这些创伤阶段做了安全验证"。

```
#[trauma_scope(v1, v2)]
macro_rules! gentle_belonging {
    () => {
        mix {
            main: belonging_certainty,
            accents: [
                { feeling: safety_presence, ratio: 0.20 }
            ]
        }
    };
}

#[trauma_scope(v1, v2)] 的语义：
    此宏的作者为创伤阶段 v1 和 v2 做了安全验证。
    v3 不在作用域内 → 如果 v3 用户的 session 请求了此宏 → Pass 3 拒绝。
    不是因为宏错了。是因为此宏没有为 v3 做过验证——作者没有声明"v3 安全"。

无标注的宏 = 默认仅对 trauma: none 开放。
安全默认。创作者想扩大作用域 = 显式声明。声明了 = 你负责。
```

---

## 宏的作用域和可见性

```
文件级：
    macro_rules! { ... }
    仅在当前 .anim 文件内可见。不和外部冲突。

包级导出：
    pub macro_rules! { ... }
    发布到 Feelings-Sandbox 的 .anim 包——导出的宏可被其他 .anim 文件 import。
    和 Rust 的 pub 语义一致。

导入：
    use feeling_package::gentle_belonging;
    导入后——宏在当前文件的 Pass 0 展开阶段可用。
```

---

## 宏展开时机——在 Pass 管线里的精确位置

```
Pass 0: LexParse（源码 → Token → AST）
    ↓
宏展开阶段（在 AST 上做语法树替换）
    ↓
Pass 1: TypeCheck（展开后的完整 AST → 类型校验）

宏展开在 Pass 0 和 Pass 1 之间。
Pass 0 生成的 AST 上有一个 MacroExpander 遍历所有 MacroCall 节点——
    查找当前作用域内匹配的 macro_rules! 定义
    → 替换为展开后的 AST 子树
    → 递归展开（宏体内可以调其他宏）
    → 最大递归深度 32（和 Rust 一致）

展开完成后 → 没有任何 MacroCall 节点残留。
Pass 1 看到的是纯 AST——不知道、也不需要知道哪些节点来自宏。
```

---

## 禁止事项

```
❌ C 风格 #define——不存在。Anim 没有文本替换宏。
❌ 宏访问用户 PBM——宏在 Pass 0 后展开，PBM 在 Pass 6 才参与。
    宏不能根据用户的心率决定展开什么。展开只依赖源码。
❌ 宏绕过安全检查——展开后的 AST 必经 Pass 1/2/3。
    不存在"展开就放行"。不存在"内联不安全"。
❌ 宏递归超过 32 层——像 Rust，有最大递归深度保护。
❌ 宏依赖外部运行时数据——宏展开在交织期。外部数据在运行期。不可交叉。
```

---

## 和 Rust / C 的三角对比

```
C #define         文本替换。不安全。无作用域。无类型。无创伤作用域。
Rust macro_rules!  语法树展开。安全。有作用域。有类型。无创伤作用域。
Anim macro_rules!  语法树展开。信号安全。有作用域。有感受类型。
                   有强度生命周期。有创伤作用域。

三层递进。
Rust 不需要创伤作用域——内存没有创伤分型。
Anim 不需要 borrow checker——信号没有 use-after-free。
但 Anim 需要"恐惧点缀 0.30 不通过"——Rust 编译器抓不到这个 BUG。
Anim 交织器能。
```

---

*Anim 宏不是字符串替换。是在语法树上先展开、再被类型检查、再被安全规则检查、再被用户上下文检查——然后才生成第一帧 ESIR。不是宏安全。是宏展开后的每一根丝都经过了全部四层防线的审查。C 的 #define 不会替你查恐惧点缀 0.30 是不是越界。Anim 会。因为信号不是日志——进了身体就退不回来。*
