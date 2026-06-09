# Anim 数学与约束规范

> 创建日期：2026-06-09
> 最后更新：2026-06-09
> 性质：所有公式与约束的单一定义点。代码从这里读，测试从这里验证，bench 从这里校准。
> 关联：`09-pbm-math.md` → 本文件。全部公式已在此合并，原文件保留为历史引用。

---

## 一、Sigmoidal 个人强度缩放

**位置**：`src/personalize.rs :: sigmoidal_scale()`

**目标**：ADR 003 §6.2——低强度接近线性，中强度响应减缓，高强度趋于饱和。
不是"整个 cap 做 sigmoid"——而是用 sigmoid 作为 **compression factor**（压缩因子），倒过来用。

**公式**：

```
x = original / cap                     — 归一化到 [0, 1]

sigmoid(x) = 1 / (1 + e^{-k × (x - x0)})   — k=6, x0=0.5
                                               σ(0)=0.047, σ(0.5)=0.5, σ(1)=0.953
                                               σ 单调递增，低→低，高→高

compression(x) = 1 − α × sigmoid(x)     — α=0.5
                                            low(x→0): compression ≈ 1.0 − 0.5×0.047 = 0.976 → 接近线性
                                            mid(x=0.5): compression = 1.0 − 0.5×0.5 = 0.75 → 开始压缩
                                            high(x→1): compression ≈ 1.0 − 0.5×0.953 = 0.524 → 饱和缩放

applied = original × baseline_coeff × compression(x)  → 四舍五入 → 整数取 min(applied, cap)
```

**数值表（cap=100, coeff=1.0）**：

| original | x | sigmoid | compression | 输出 | 描述 |
|-----------|---|---------|-------------|------|------|
| 5 | 0.05 | 0.063 | 0.97 | ≈ 4.8 | 低强度≈线性 |
| 10 | 0.10 | 0.083 | 0.96 | ≈ 9.6 | |
| 20 | 0.20 | 0.142 | 0.93 | ≈ 18.6 | |
| 30 | 0.30 | 0.231 | 0.88 | ≈ 26.5 | 开始减缓 |
| 50 | 0.50 | 0.500 | 0.75 | ≈ 37.5 | 显著压缩 |
| 70 | 0.70 | 0.769 | 0.62 | ≈ 43.1 | |
| 90 | 0.90 | 0.917 | 0.54 | ≈ 48.7 | 接近饱和 |
| 100 | 1.00 | 0.953 | 0.52 | ≈ 52.4 | 饱和上限 |

**固定安全上限校验**：输出后若 `applied > user_cap` → 拒绝（`UserStateSafetyError`）。

---

## 二、PBM 冷启动四维系数

**位置**：`src/personalize.rs :: PbmColdStartCoefficients`

**来源**：Feelings-ROADMAP §1.2

```
Visceral  （内脏）  ：  0.75   — 心率、呼吸、厌恶/恶心回路  低→需要更强信号才能触发。
Emotional （情绪）  ：  0.40   — 恐惧、悲伤、喜悦等弥散情绪
Tactile   （触觉）  ：  0.80   — 压感、轻触、CT 纤维
Auditory  （听觉）  ：  0.85   — 听觉感知
```

**系数含义**：系数 < 1.0 表示该维度的感受触发基线低于通用模板。
新用户在同等信号强度下需要更长的适应时间或更强信号。

**适用范围**：冷启动前 10 次 Session。之后按 PBM 个人基线收敛动态调整。

**当前限制（v0.3）**：所有原子无条件使用 `Emotional` 系数（0.40）。
`AtomEntry` 需新增 `dimension: PbmDimension` 字段后再切换。（FORGET P1 #23）

---

## 三、DampingMatrix 跨维度冻结规则

**位置**：`src/pbm.rs :: DampingMatrix`

**规则表**：

```
触发维度        目标维度        冻结？
──────────────────────────────────────────
Emotional      Visceral       → ✅ 冻结
Emotional      Tactile        → ✅ 冻结
Emotional      Auditory       → ❌ 保持活动
Visceral       Emotional      → ✅ 冻结
Visceral       Tactile        → ❌ 保持活动
Visceral       Auditory       → ❌ 保持活动
Tactile        (任何目标)      → ❌ 瞬时震荡很少扩散
Auditory       (任何目标)      → ❌ 瞬时震荡很少扩散
自身→自身      —              → ❌ 自身不冻结自身
```

**实现**：`should_freeze(trigger, target)` 返回 `Some(trigger)` 表示冻结，
`None` 表示保持活跃。`DampingMatrix` 是一个零大小结构体，所有规则是静态的。

**多维度同时触发**：冻结集合取并集（e.g. Emotional 触发 + Visceral 触发 → 冻结全部重叠维度）。

**当前限制（v0.3）**：`application(&gradients, &steps)` 是静态映射表，不接收实时梯度输入。
v0.4 需传入各维度瞬时梯度 + 历史步长用于动态判定。（FORGET P1 #25）

---

## 四、点缀比例帽

**位置**：`src/personalize.rs :: default_accent_cap()`

**当前（v0.3）**：全原子统一 0.30。

**后续扩展**：从 `Registry::max_ratio()` 读取每原子独立帽（见 `src/registry.rs`）。

**调整规则（Damping 态）**：

```
非冻结：applied = original.min(ratio_cap)
冻结态：applied = (original / 2.0).min(ratio_cap)
```

---

## 五、类型/安全约束速查表

**全局上限**：

```
MAX_GLOBAL_INTENSITY    = 100
MIN_VALID_INTENSITY     = 0
ABRUPT_STOP_MAX         = 20 （强度 ≤ 20 才允许 abrupt_stop）
```

**点缀配比范围**：

```
VALID_RATIO_MIN = 0.0
VALID_RATIO_MAX = 1.0  （外部 Registry 也受此约束）
```

**沙箱原子约束**（`rule.rs`）：

```
max_intensity ≤ 30
未验证（unverified_atom）→ 仅限创作者及授权小范围用户
```

**强度缩放**（`safety.rs :: scale_intensity()`）：

```
scaled_min = min × user_cap / 100
scaled_max = max × user_cap / 100
前提：scaled_max ≤ user_cap 已过 safety 校验
```

---

## 六、Trauma 路径重定向规则（v0.3 未实现）

**来源**：`docs/safety/trauma-protocol.md`，ADR 003 §6.1。

```
trauma v1:  禁主不禁点，强度上限不受影响
trauma v2:  禁主，点缀配比减半
trauma v3:  全禁——主旋律 & 点缀
未成年人：  强度 ≤ 20，亲密维度强制隔离 → 全部交给 safety::check()
```

**当前（v0.3）**：`personalize.rs` 中 `trauma_rerouted` 硬编码为 `false`。
`safety::check()` 是 no-op。这些需要 v0.3 的 trauma 判定逻辑实现。

---

## 版本管理

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.2 | 2026-06-09 | 从 `009-pbm-math.md` 拆分，覆盖 sigmoidal、冷启动、阻尼、点缀帽、约束速查、trauma |
| v0.1 | 2026-06-09 | 初始——仅 sigmoidal、冷启动系数、阻尼规则、点缀帽 |
