# Anim 数学与约束规范

> 创建日期：2026-06-09
> 最后更新：2026-06-09
> 性质：所有公式与约束的单一定义点。代码从这里读，测试从这里验证，bench 从这里校准。

---

## 一、Sigmoidal 个人强度缩放

**位置**：`src/personalize.rs :: sigmoidal_scale()`

**目标**：ADR 003 §6.2——低强度接近线性，中强度响应减缓，高强度趋于饱和。
不是"整个 cap 做 sigmoid"——是用 sigmoid 作为 compression factor（压缩因子），倒过来用。

**公式**：

```
x = original / cap                     — 归一化到 [0, 1]

sigmoid(x) = 1 / (1 + e^{-k × (x - x0)})   — k=6, x0=0.5

compression(x) = 1 − α × sigmoid(x)     — α=0.5

applied = original × baseline_coeff × compression(x)  → 四舍五入 → min(applied, cap)
```

**数值表（cap=100, coeff=1.0）**：

| original | sigmoid | compression | 输出 | 描述 |
|----------|---------|-------------|------|------|
| 5 | 0.063 | 0.97 | ≈ 4.8 | 低强度≈线性 |
| 10 | 0.083 | 0.96 | ≈ 9.6 | |
| 20 | 0.142 | 0.93 | ≈ 18.6 | |
| 30 | 0.231 | 0.88 | ≈ 26.5 | 开始减缓 |
| 50 | 0.500 | 0.75 | ≈ 37.5 | 显著压缩 |
| 70 | 0.769 | 0.62 | ≈ 43.1 | |
| 90 | 0.917 | 0.54 | ≈ 48.7 | 接近饱和 |
| 100 | 0.953 | 0.52 | ≈ 52.4 | 饱和上限 |

**固定安全上限校验**：输出后若 `applied > user_cap` → 拒绝。

---

## 二、强度等比缩放

**位置**：`src/safety.rs :: scale_intensity()`

**目标**：当 user_cap < source max 时，等比缩放整个区间。

**公式**：

```
ratio = user_cap / source_max
scaled_min = round(source_min × ratio)
scaled_max = user_cap

例：source [15, 60], cap=45
    ratio = 45/60 = 0.75
    scaled_min = round(15 × 0.75) = 11
    scaled_max = 45
    输出 [11, 45]
```

**边界条件**：如果 `source_max ≤ user_cap` → 不缩放，原样返回。

---

## 三、PBM 冷启动四维系数

**位置**：`src/personalize.rs :: PbmColdStartCoefficients`

**来源**：Feelings-ROADMAP §1.2

| 维度 | 系数 | 生理基底 |
|------|------|---------|
| Visceral | 0.75 | 心率、呼吸、厌恶/恶心回路 |
| Emotional | 0.40 | 恐惧、悲伤、喜悦等弥散情绪 |
| Tactile | 0.80 | 压感、轻触、CT 纤维 |
| Auditory | 0.85 | 听觉感知 |

**系数含义**：系数 < 1.0 = 该维度的感受触发基线低于通用模板。新用户需要更强信号。

**适用范围**：冷启动前 10 次 Session。之后按 PBM 个人基线收敛动态调整。

**当前限制（v0.3）**：所有原子无条件使用 Emotional 系数（0.40）。
`AtomEntry` 需新增 `dimension: PbmDimension` 字段后再切换。（FORGET P1 #23）

---

## 四、DampingMatrix 跨维度冻结规则

**位置**：`src/pbm.rs :: DampingMatrix`

**冻结判定**：`should_freeze(trigger, target)` → `Some(trigger) = 冻结`, `None = 保持`

| 触发维度 | 目标维度 | 冻结？ | 说明 |
|----------|---------|--------|------|
| Emotional | Visceral | ✅ | 情绪剧烈波动 → 抑制内脏 |
| Emotional | Tactile | ✅ | 情绪剧烈波动 → 抑制触觉 |
| Emotional | Auditory | ❌ | 听觉免于情绪冻结 |
| Visceral | Emotional | ✅ | 心率骤升 → 抑制情绪 |
| Visceral | Tactile | ❌ | 内脏不封锁触觉 |
| Visceral | Auditory | ❌ | 内脏不封锁听觉 |
| Tactile | 任何 | ❌ | 瞬时震荡很少扩散 |
| Auditory | 任何 | ❌ | 瞬时震荡很少扩散 |
| 自身→自身 | — | ❌ | 自身不冻结自身 |

**各维度梯度阈值**（`gradient_threshold()`）：

| 维度 | 阈值（×step） | 含义 |
|------|--------------|------|
| Emotional | 2.0× | 情绪梯度超过 2 倍步长 → 触发阻尼 |
| Visceral | 1.5× | 内脏梯度超过 1.5 倍步长 → 触发阻尼 |
| Tactile | 3.0× | 触觉梯度超过 3 倍步长 (高容忍) → 触发阻尼 |
| Auditory | 2.0× | 听觉梯度超过 2 倍步长 → 触发阻尼 |

**多维度同时触发**：冻结集合取并集。

**当前限制（v0.3）**：`apply()` 是静态映射表，不接收实时梯度输入。
v0.4 需传入各维度瞬时梯度 + 历史步长。（FORGET P1 #25）

---

## 五、SessionLabel → PbmUpdateStrategy 映射

**位置**：`src/pbm.rs :: SessionLabel`

| SessionLabel | PbmUpdateStrategy | 含义 |
|-------------|-------------------|------|
| Normal | Full | 基线 + 安全阈值全更新 |
| Abnormal | SafetyOnly | 仅更新安全阈值，基线冻结 |
| ColdStart | Full | 前 10 次 Session——全更新但预测器关闭 |

---

## 六、DataConfidence → 步长乘数

**位置**：`src/pbm.rs :: DataConfidence`

| DataConfidence | step_multiplier | 含义 |
|---------------|-----------------|------|
| High | ×1.0 | 全步长 |
| Low | ×0.15 | 因子三降级（ADR 007 §7.6）— 信号质量低 |
| Contaminated | ×0.0 | 冻结——不更新基线 |

---

## 七、ColdStartGuard 冷启动守护

**位置**：`src/pbm.rs :: ColdStartGuard`

| 参数 | 默认值 | 含义 |
|------|--------|------|
| threshold | 10 | 前 10 次 Session 判定为冷启动 |
| session_count | 0 | 递增计数器 |

**规则**：
- `is_cold_start()`: `session_count < threshold`
- `predictor_enabled()`: `!is_cold_start()`
- `complete_session()`: `session_count += 1`

---

## 八、OiSmoothing 衰减曲线

**位置**：`src/guard.rs :: inject()`

**目标**：oi 帧被拒绝时——不硬截断，平滑衰减到安全基线。

| 源码强度 | 衰减策略 | 序列 | 说明 |
|----------|---------|------|------|
| ≤ 20 | 硬截断 | [×1.0, ×0.0] | 低强度——直接关停安全 |
| > 20 | 五帧衰减 | [×1.0, ×0.6, ×0.3, ×0.1, ×0.0] | 高强度不能直接跳零——岛叶预期链断裂冲击太大 |

**物理意义**：五帧 = 4-8ms（FPGA 门级逻辑 → 微秒级响应，不是软件循环）。

---

## 九、点缀比例帽

**位置**：`src/personalize.rs :: default_accent_cap()`、`src/rule.rs`

| 原子类型 | 当前帽 | 来源 |
|---------|--------|------|
| Core | 0.30（通用默认）/ Registry.max_ratio() | `default_accent_cap()` |
| Sandbox | 0.30 | `rule.rs :: SANDBOX_ACCENT_MAX` |

**Damping 态调整**：

```
非冻结：applied = original.min(ratio_cap)
冻结态：applied = (original / 2.0).min(ratio_cap)
```

**后续扩展**：从 `Registry::max_ratio()` 读取每原子独立帽。

---

## 十、静态安全约束速查

**位置**：`src/rule.rs`、`src/safety.rs`

| 约束 | 常量/公式 | 位置 |
|------|----------|------|
| 全局强度上限 | `MAX_GLOBAL_INTENSITY = 100` | `rule.rs` |
| abrupt_stop 形状强度 | `≤ 20` | `rule.rs` |
| 沙箱原子作点缀 | `ratio ≤ 0.30` | `rule.rs` |
| 沙箱原子作主旋律 | 禁止 | `rule.rs` |
| 强度等比缩放 | `ratio = user_cap / source_max` | `safety.rs :: scale_intensity()` |
| 强度区间合法性 | `max ≥ min` | `ast.rs :: Intensity::validate()` |

---

## 十一、Trauma 路径重定向（v0.3 未实现）

**来源**：`docs/safety/trauma-protocol.md`，ADR 003 §6.1。

| 分级 | 主旋律 | 点缀 | 强度上限 |
|------|--------|------|---------|
| trauma v1 | 禁（安全类外全禁） | 允许，上限不变 | 不受影响 |
| trauma v2 | 禁 | 配比减半 | 同 v1 |
| trauma v3 | 禁 | 全禁 | 全禁 |
| 未成年人 | 强度 ≤ 20 | 亲密维度强制隔离 | 20 |

**当前（v0.3）**：`personalize.rs` 中 `trauma_rerouted` 硬编码为 `false`。`safety::check()` 是 no-op。

---

## 版本管理

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.3 | 2026-06-09 | 完整重写：sigmoidal + 强度缩放 + OiSmoothing + SessionLabel × DataConfidence + 约束速查 + Trauma |
| v0.2 | 2026-06-09 | 从 009-pbm-math.md 拆分，覆盖 sigmoidal、冷启动、阻尼、点缀帽。迁移至 archived. |
| v0.1 | 2026-06-09 | 初始——仅 sigmoidal、冷启动系数、阻尼规则、点缀帽 |
