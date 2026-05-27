# ADR 006: 错误处理——oi

> 状态：已定稿
> 日期：2026-05-26
> 对应规范：Anim-CROSSOVER.md

---

## 动机

Rust 用 `Result<T, E>` + `?` 传播错误。Go 用 `if err != nil`。C++ 用异常。Anim 不是编译器。Anim 的错误不是"程序崩了"——是"信号被挡了"。不需要堆栈。不需要 panic。不需要 unwrap。只需要一声短的信号——oi。这根丝不能和这根丝交叉。

---

## 零、oi 的现实语义——一声叫停

oi 不是编程语言发明的。oi 在现实里就是一声喝止。

```
你正要过马路——
    朋友喊 oi。
    有车。
    你没看到车。但你停住了。
    不是因为你懂了交规。是因为那一声 oi 直接剪断了你正要迈出去的运动程序。
    不需要解释。不需要理由。不需要"请处理以下异常"。
    就是——停。

你正要伸手碰热锅——
    你妈在背后喊 oi。
    你的手没碰到锅。收回来的速度比你听完解释的速度快得多。
    不是因为你理解了烧伤的病理机制。
    是因为那一声 oi 把你的运动皮层到手指的桥断了一帧。
```

**oi 不是错误。oi 是喝止。** 不是"程序崩了，请查阅堆栈"——是"这一帧这一根丝不准过"。挡了。没交叉。下一帧继续。

---

## 零之附、oi 不判断感受的好坏

Feelings 容纳所有感受。平静在。恐惧在。愤怒在。绝望在。暴力在。physical_danger 在。rage_eruption 在。grief_loss 在。没有这些——Feelings 是感受的精选集。精选集不是感受民主化。

```
oi 从不因为"这个感受不好"而拒绝：

    恐惧 → oi 不拒绝。恐惧在核心原子列表里。合法的感受。
    愤怒 → oi 不拒绝。rage_eruption 在 Pattern Registry 里。
    绝望 → oi 不拒绝。grief_loss 是所有用户都可以访问的感受类型。
    暴力 → oi 不拒绝。physical_danger 在沙盒原子里。

    oi 只拒绝一件事：安全规则被违反。

        强度 > cap → oi。不是恐惧不好。是强度超了。
        创伤 v3 × 归属 → oi。不是归属不好。是这个用户的创伤分型不能交叉这一根。
        未成年 × 亲密维度 → oi。不是亲密不好。是这个用户没到年龄。
        组合情绪对冲 → 不是愤怒不好。是愤怒 + 平静同一包无显式声明不安全。

    oi 挡的不是感受。
    oi 挡的是不安全的交叉。
    丝本身没有好坏。交叉的资格有边界。
```

---

## 决策

Anim 的错误处理叫 `oi`。不是 `Err`。不是 `Error`。是 `oi`。

### 为什么是 oi，不是 Err

```
Err / Error：
    严重。需要处理。需要堆栈。需要恢复流程。
    → 程序的语义。编译器的语义。

oi：
    轻。短。不 panic。不堆栈。不恢复流程——因为没有"流程"要恢复。
    丝被挡了。纺锤没出丝。交叉没发生。下一帧继续。
    → 织机的语义。交织器的语义。
```

### 语法

```rust
// Rust 侧——animi 的 Pass 管线
fn cross_over(warp: &Warp, weft: &Weft) -> Result<CrossingPoint, oi> {
    if !warp.can_cross(&weft) {
        oi!("warp '{}' cannot cross with weft '{}' at this point", warp.name, weft.name);
    }
    Ok(CrossingPoint::new(warp, weft))
}

// 调用方
let point = warp.cross_over(&weft, oi)?;
// oi? = 如果交叉被挡——oi 往上走。不堆栈。不 panic。上一层知道"这帧交叉没发生"。
```

### oi 的三个特征

```
1. oi 不携带堆栈。

    Rust 的 Result::Err 带堆栈跟踪——因为程序需要知道"从哪来的"。
    Anim 不需要。交叉被挡的原因只有一种——"这根经线没有和这根纬线交叉的资格"。
    不需要追溯调用链。不需要堆栈。只需要一声 oi。

2. oi 不 panic。不 unwind。

    Anim 的错误不是异常——是正常的交叉路径之一。
    交叉被挡 = 这帧 ESIR 不生成。下一帧继续。
    oi 不是红色的——是黄色的。"这帧不行。下一帧。"

3. oi 是"挡"——不是"错"。

    强度超过 cap → oi。不是程序错了——是安全层在挡。
    创伤原子被禁 → oi。不是用户错了——是交叉资格的断言生效了。
    组合情绪对冲 → oi。不是源码错了——是编织器说"这一针不安全"。
```

### oi 的生命周期

```
Pass 2 StaticSafety → 强度超过 cap → oi!("intensity {n} exceeds cap {c}")
    → 不生成后续 IR。oi 被返回给 animi 调用方。

Pass 3 UserStateSafety → 创伤 v3 不可交叉归属原子 → oi!("trauma_v3 cannot cross with belonging")
    → 同上。

Pass 4 RuntimeGuard → 预埋插桩。运行期交叉被挡。
    → oi 不返回给 animi——oi 是 ESIR 帧里的一个标记位。
    → 安全停止帧读到这个标记位 → 切回保底包。

Pass 8 CodeGen → 帧级偏差修正 → 心率超出硬上限
    → oi 在 FPGA 上是一个寄存器比较结果。
    → oi = true → 立即切换到紧急截断曲线帧。
```

### 和 Rust / Go 的对比

```
Rust    Result<T, E> + ?    错误是操作失败。堆栈可追溯。可恢复。
Go      if err != nil       错误是返回值。显式检查。调用方决定。
Anim    oi                  不是错。是挡。不堆栈。不 panic。不判断要不要恢复——
                            恢复是自动的——下一帧继续交叉。

Rust   Err(anyhow!("..."))           -> 程序可能崩了。请查堆栈。
Go     if err != nil                 -> 函数可能没完成。调用方继续判断。
Anim   oi!("warp cannot cross")      -> 这帧交叉没发生。下一帧。

现实   "oi! 有车!"                    -> 你正要迈出去。身后一声叫停。
                                        运动程序被剪断了一帧。
                                        不需要解释。不需要堆栈。
                                        那一脚没踩下去。就是停了。
```

---

## 实现

```rust
// oi 不是 Error trait——是 Anim 自己的轻量挡板
#[derive(Debug)]
pub struct Oi {
    message: String,
    // 没有堆栈。没有 source。没有 backtrace。
}

impl Oi {
    pub fn new(msg: impl Into<String>) -> Self {
        Oi { message: msg.into() }
    }
}

// oi! 宏——和 format! 一样轻
macro_rules! oi {
    ($($arg:tt)*) => {
        Oi::new(format!($($arg)*))
    };
}

// oi 可以被转换——如果需要给上层看
impl From<Oi> for AnimiError {
    fn from(oi: Oi) -> Self {
        AnimiError::CrossBlocked(oi)
    }
}

// 但 oi 不是 AnimiError 的子类。
// AnimiError = 真的错了（文件读不到、Pattern Registry 打不开）。
// oi = 挡了。正常。
```

---

## 禁止事项

```
❌ oi 携带堆栈——不。堆栈是程序的记忆。oi 不需要记忆。只需要提示。
❌ oi.panic()——不存在。oi 不可以被 panic。oi 不是异常——是交叉路径的分岔。
❌ oi = "出错了"——oi 不是错。oi 是"这根丝没资格交叉"。用词必须精确。
❌ oi 被吞掉——oi 被返回之后，调用方必须显式处理。不能 `let _ = oi;`。
    如果交叉被挡但调用方无视——Pass 2 拒绝生成后续 IR = 编译不过。
```

---

*oi。不是 Err。不是 Error。不是异常。不是栈上的红字。是朋友在你过马路时喊的那一声"有车"——你没看到车。但你停住了。不是因为理解了交规。是因为那一声 oi 剪断了你正要迈出去的那一帧运动程序。Anim 的 oi 同构：强度超 cap 的 weft 正在 cross over——oi 喊住了它。没交叉。没信号。你还在路上。没踩下去。下一帧新纬线。一帧不差。丝不断。*
