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

**对 011/012 的影响（v0.3 现实）**：
DampingMatrix 是 012 里 DeadBand 和方向切换保护的前置条件——
如果跨维度冻结规则是假的——梯度检测不生效——
那么从激活切安抚时——应该被阻尼冻结的维度仍在自由输出——
DeadBand 期间这些未冻结的信号仍在冲击受体——
方向切换保护的完整性当前不成立。
不是 DeadBand 设计错了——是它的前置阻尼还没实现。

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

### 7.1 冷启动阻尼淡入窗（v0.4）

**位置**：`src/personalize.rs` — `damping_window_alpha` + `freeze_factor`

冷启动结束后（Session 10 起），阻尼从 0% 线性过渡到 100%。避免 Session 9→10 首次触发阻尼时的断崖效应。

**参数**：

| 参数 | 默认值 | 含义 |
|------|--------|------|
| W (cold_start_window) | 5 | 淡入窗口长度（Session 数） |
| S_crit | 10 | 冷启动阈值（ColdStartGuard.threshold） |

**公式**：

```
α = min(1.0, (S - S_crit) / W)          // 过渡因子，Session 10→α=0, Session 15+→α=1
freeze_factor = 1.0 - 0.5 × α           // 阻尼因子，1.0→0.5
applied_ratio = (original.min(ratio_cap)) × freeze_factor
```

**Session 演算**：

```
S=10  α=0.0  freeze_factor=1.00   →  零压制（无断崖）
S=11  α=0.2  freeze_factor=0.90   →  轻试探
S=12  α=0.4  freeze_factor=0.80   →  缓进
S=13  α=0.6  freeze_factor=0.70   →  爬升
S=14  α=0.8  freeze_factor=0.60   →  接近全力
S=15+ α=1.0  freeze_factor=0.50   →  全额阻尼（过渡窗关闭）
```

**边界条件**：`cold_start_window = 0` → α 固定为 1.0 → 无窗口——首帧全额阻尼。`cold_start = true` → `is_frozen()` 永远返回 false——公式不触发。

**v0.3 现实**：冷启动阻尼淡入窗完全未实现——属于 v0.4 规划。
v0.3 无此保护——Session 9→10 首次触发阻尼时——断崖效应完全暴露——
用户在第 9 次 Session 感受不到任何阻尼——第 10 次突然全额压制——
感受断层——"为什么昨天还好好的——今天突然全变了"——
系统对此无解释——无过渡——无预警。
在阻尼淡入窗落地之前——Session 9→10 的断崖是冷启动期最脆弱的时刻。

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

**Damping 态调整**（v0.4 修正——先卡帽再斩断）：

```
非冻结：applied = original.min(ratio_cap)
冻结态：applied = (original.min(ratio_cap)) × freeze_factor   // freeze_factor ∈ [1.0, 0.5]
```

**修正说明**：旧版 `(original / 2.0).min(ratio_cap)` 先斩断再卡帽——原始值存在"溢出缓冲垫"时阻尼被对冲失效（0.8/2.0=0.4→min(0.3)=0.3，满额输出）。新版先卡帽再乘冻结因子——阻尼在 cap 内真正生效（0.8→min(0.3)→×0.5=0.15）。

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
| 低锚点置信度硬上限 | `low_anchor_cap(user_cap, R)` — R < 0.3 → `cap = min(20, user_cap)` | `safety.rs :: low_anchor_cap()` |
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
| 低锚点置信度用户 | strength cap ≤ 20（`anchor_confidence R < 0.3`） | 20 |
| 普通用户 | 按个人基线动态 cap | 按 R 分段 |

**当前（v0.4）**：`personalize.rs` 中 `trauma_rerouted` 硬编码为 `false`。`safety::check()` 是 no-op。`safety::low_anchor_cap()` 已落地——锚点置信度 R<0.3→cap 20 硬线（不是年龄——是锚在里还是在外）。

**对 011/012 的依赖链影响**：
011 的 NeuroEnergyTracker、012 的 P0 硬抢占/主动麻痹/带外快照覆写——
全部依赖 Trauma 路径能正确重定向这个前提。
011 写道——"Trauma v3: LeakRate→~0, CriticalThreshold→极低"。
012 写道——"主动麻痹是 Core 的沙箱指令——trauma v3 的神经电暴——Feelings 在信号层提前把门关了"。
这些协议的触发条件——全部建在 `trauma_rerouted = true` 之上。
当前这个前提 hardcoded false——等于 011 和 012 的 trauma 级防御全部挂在空中。
不是协议设计错了——是协议在等执行层追上。

---
---
## 十二、实现态诚实清单——纸面和代码之间的距离

```
下面每一项都不是设计缺陷。是"当前代码还没跟上来"的真实状态。
不假装已经实现了——不在文档里用将来时当现在时。

[ ] DampingMatrix          v0.3 apply() 是静态映射——不接收实时梯度。
                            012 的 DeadBand/方向切换依赖此为前提——当前前提空缺。
                            → v0.4 计划接入。

[ ] 冷启动阻尼淡入窗        v0.3 无此保护——Session 9→10 断崖完全暴露。
                            用户在 Session 10 突然遭遇全额阻尼——无过渡无预警。
                            → v0.4 计划接入。冷启动前 10 个 Session damping 完全关闭。

[ ] Trauma 路径重定向      hardcoded false——011/012 的全部 trauma v3 防御挂空。
                            Trauma v3 用户的 LeakRate→~0、主动麻痹、P0 抢占——
                            这些协议在纸面上完整——在代码里等这一行从 false 变成条件分支。
                            → v0.4 计划——从 PBM 档案读取 trauma_tier。

[ ] low_anchor_cap          safety.rs 有 low_anchor_cap(20)——
                            personalize.rs 当前未调用——低锚用户的 cap=20 硬线在 pass 层未闭合。
                            → 接入点明确——personalize() 强度上限校验前加一行。

[ ] PBM 宿主                pbm.rs 当前在 Anim 的 src/ 下——由 Anim 进程直接管理。
                            011/012 的"本地闭环""数据不离设备"——
                            全跑在 Anim 进程里——Anim 替 Core 扛着 PBM 的所有重活。
                            Core 零代码——不是架构矛盾——是 Core 还没出生。
```

不是羞愧地承认——是诚实地标注——让读代码的人知道哪些是设计——哪些是"还没实现"。

---

## 版本管理

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.4 | 2026-06-10 | §九点缀比例帽语义修正+freeze_factor。§七冷启动阻尼淡入窗。§十低锚点置信度硬上限 low_anchor_cap(R<0.3→cap 20)。§十一创伤表更新——"未成年"→锚点置信度。|
| v0.3 | 2026-06-09 | 完整重写：sigmoidal + 强度缩放 + OiSmoothing + SessionLabel × DataConfidence + 约束速查 + Trauma |
| v0.2 | 2026-06-09 | 从 009-pbm-math.md 拆分，覆盖 sigmoidal、冷启动、阻尼、点缀帽。迁移至 archived. |
| v0.1 | 2026-06-09 | 初始——仅 sigmoidal、冷启动系数、阻尼规则、点缀帽 |
