# Anim 开发哲学

> 性质：强制约定，不是建议
> 范围：Anim 仓库所有 Rust 代码、文档、提交
> 更新：随项目演进持续修订
> 上级哲学：Feelings-LANGUAGE.md（语言规范）、Feelings-MATRIX.md（万物皆矩阵）

---

## 一、Anim 不是编译器

任何被称为「编译器」的代码，在 Anim 里就是写错了。

```
编译器做的事     把 A 翻译成 B。单向。一次性。
animi 做的事     把多股独立流织成一条连续的信号绳。
                每一层都在把一股新的东西织进去。
                FSIR 织入语义，PSIR 织入个人基线，
                DSIR 织入设备约束，ESIR 织入实时反馈。
```

代码里的命名、注释、文档——全部用「交织」「纺织」「编织」的语言，不用「编译」「翻译」「转换」。

---

## 二、安全在交织期，不在运行期

Anim 的安全模型是三层防线，全在交织期完成。

```
Pass 2: StaticSafety     源码级的安全规则——不依赖任何用户上下文
Pass 3: UserStateSafety  源码 × 用户上下文——创伤分型交叉判定
Pass 4: RuntimeGuard     编译期预埋，运行期激活——安全插桩
```

原则：能在 Pass 2 拒绝的，不留到 Pass 3。能在编译期插桩的，不等到运行期。运行期的神经信号已经进入人体了——安全校验必须在信号生成之前完成。

---

## 三、个人是偏移量，不是新源码

同一份 `.anim` 源码，不同的人，不同的交织结果。但个人差异不写在源码里——它写在 PBM（个人基线矩阵）里。

```
源码     = 通用感受结构（creator 写一次）
FSIR     = 源码的直接表达（纯感受语义）
PSIR     = FSIR × PBM（个人基线左乘）→ 适配到此人的信号参数
```

PBM 永不离设备。PBM 的四维差异化冷启动（内脏 0.75 / 情绪 0.40 / 触觉 0.80 / 听觉 0.85）是硬编码进 personalize.rs 的——不是配置文件。

---

## 四、感受是流，不是文件

感受包不是静态 JSON。它是每毫秒重新参数化的实时程序。

```
每一帧的 ESIR 参数依赖上一帧的生理反馈。
ESIRₜ → 注入 → 生理响应 → 偏差 → ESIRₜ₊₁ = ESIRₜ + 修正量。
```

代码里不做「加载感受包 → 播放」的模型。做「打开 session → 帧级交织 → 闭环修正」的模型。

---

## 五、零成本抽象——感受级别的

Anim v0.x 用 Rust 实现。Rust 的零成本抽象是内存级别的。Anim 的零成本抽象是感受级别的。

```
.anim 源码里写的 @auto_reduce_on(heart_rate > 120)
在交织期被展开为 ESIR 层的安全插桩代码。
不是运行时动态分支——是交织期代码生成。
运行时零开销——直接读寄存器值做线性插值。
```

---

## 六、空值处理

感受不存在 = None。信号失败 = Err。没有 null，没有 -1，没有空字符串当哨兵。

```rust
// ✅
fn get_feeling_atom(name: &str) -> Option<FeelingAtom>;
fn generate_esir_frame(psir: &Psir) -> Result<EsirFrame>;

// ❌
fn get_feeling_atom(name: &str) -> String;  // 空串表示没找到
```

---

## 七、错误处理

不用 String 当错误。不用裸 panic。`?` 传播优先。

```rust
// ✅ 用 thiserror 定义错误类型
#[derive(Error, Debug)]
enum AnimiError {
    #[error("feeling atom '{0}' not found in registry")]
    AtomNotFound(String),
    #[error("intensity {0} exceeds cap {1}")]
    IntensityExceedsCap(u8, u8),
}

// ❌
fn check() -> Result<(), String> { ... }
panic!("出错了");
```

---

## 八、禁止事项

```
❌ unsafe 代码（除非有性能基准证明必要 + 独立审查）
❌ unwrap() 在非测试/非断言代码中
❌ 硬编码生理阈值——阈值数据来自 Pattern Registry
❌ 全局可变状态（static mut）
❌ 把「编译器」写进代码、注释、文档——写「交织器」
❌ 把 animi 叫成 animc——animc 已退役
❌ println! / eprintln!（走 tracing）
❌ .clone() 满天飞（先想借用）
```

---

## 九、文档风格

```
技术文档    英文优先，核心概念保留中文别称（交织/编织/感受原子）
代码注释    英文
commit      type(scope): description（ASCII subject）
架构讨论    中英混合，和 Feelings 上层文档风格一致
```

---

*Anim 不是编译器。是编织者。每一行代码都在把抽象的感受织成真实的神经信号。*
