# Anim 数学公式手册（ADR 009 前置）

> 创建日期：2026-06-09
> 版本：v0.1
> 性质：所有公式的单一定义点——代码从这里读，测试从这里验证，bench 从这里校准

---

## 一、Sigmoidal 个人强度缩放

**位置**：`src/personalize.rs :: sigmoidal_scale()`

**目标**：ADR 003 §6.2——低强度接近线性，中强度响应减缓，高强度趋于饱和。

**公式（v0.3 — 已修正）**：

```
x = original / cap                     — 归一化到 [0, 1]

sigmoid(x) = 1 / (1 + e^{-k × (x - x0)})   — k=6, x0=0.5
                                               σ(0)=0.047, σ(0.5)=0.5, σ(1)=0.953
                                               σ 单调递增，低→低，高→高

compression(x) = ─ 1 − α × sigmoid(x)  — α=0.5
                                            low(x→0): compression ≈ 1.0 − 0.5×0.047 = 0.976 → 接近线性
                                            mid(x=0.5): compression = 1.0 − 0.5×0.5 = 0.75 → 开始压缩
                                            high(x→1): compression ≈ 1.0 − 0.5×0.953 = 0.524 → 饱和缩放

applied = original × baseline_coeff × compression(x)  → 四舍五入 → 整数取 min(applied, cap)
```

**数值表（cap=100, coeff=1.0）**：

| original | x | sigmoid | compression | 个人比例因子后的输出 | 描述 |
|-----------|---|---------|-------------|-------------------|------|
| 5 | 0.05 | 0.063 | 0.97 | ≈ 4.8 | 低强度≈线性 |
| 10 | 0.10 | 0.083 | 0.96 | ≈ 9.6 | |
| 20 | 0.20 | 0.142 | 0.93 | ≈ 18.6 | |
| 30 | 0.30 | 0.231 | 0.88 | ≈ 26.5 | 开始减缓 |
| 50 | 0.50 | 0.500 | 0.75 | ≈ 37.5 | 显著压缩 |
| 70 | 0.70 | 0.769 | 0.62 | ≈ 43.1 | |
| 90 | 0.90 | 0.917 | 0.54 | ≈ 48.7 | 接近饱和 |
| 100 | 1.00 | 0.953 | 0.52 | ≈ 52.4 | 饱和上限 |

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

**当前限制（v0.3）**：所有原子无条件使用 `Emotional` 系数（0.40）。
`AtomEntry` 需新增 `dimension: PbmDimension` 字段后再切换。（FORGET P1 #23）

---

## 三、DampingMatrix 跨维度冻结规则

**位置**：`src/pbm.rs :: DampingMatrix`

**规则**：

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

**实现细节**：`should_freeze(trigger, target)` 返回 `Some(trigger)` 表示冻结。

**当前限制（v0.3）**：`application(&gradients, &steps)` 是静态映射表，不接收实时梯度输入。
v0.4 需传入各维度瞬时梯度+历史步长用于动态判定。

---

## 四、点缀比例帽

**位置**：`src/personalize.rs :: default_accent_cap()`

**当前（v0.3）**：全原子统一 0.30。后续从 `Registry::max_ratio()` 读取每原子独立帽。

**调整规则**（Damping 态）：
- 非冻结：`applied = original.min(ratio_cap)`
- 冻结态：`applied = (original / 2.0).min(ratio_cap)`

---

## 五、版本管理

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1 | 2026-06-09 | 初始——sigmoidal、冷启动系数、阻尼规则、点缀帽 |
