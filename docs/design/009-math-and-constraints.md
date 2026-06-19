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

**v0.4 现实**：冷启动阻尼淡入窗已实现（`personalize.rs` L129-147——`damping_window_alpha` + `freeze_factor`）。
v0.3 无此保护时——Session 9→10 断崖完全暴露。现已修复——Session 10 起 5 个 Session 内阻尼从 0% 线性过渡到 100%。

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

## 十、沙箱路由与 Governance 治理

**位置**：`src/sandbox.rs`（新增）、`src/personalize.rs`

### 十.1 设计前提——沙箱重定义

旧模型：沙箱 = `AtomClass::Sandbox` 标签。原子打上 Sandbox 标签 → 受限。

新模型（v0.5+）：**沙箱 = 强度阈值触发的执行环境。** 与原子分类解耦。

```
intensity.max ≥ SANDBOX_THRESHOLD (90)
  → 无论原子是 Core 还是 Sandbox
  → 整个感受包在沙箱内执行
  → Governance 规则引擎接管
```

**为什么不按原子分类**：内建 Registry 8 个原子全是 Core——没有 Sandbox 原子。标签沙箱从未实际生效。真正的风险不在原子类型，在强度量级。90+ 的信号无论是什么感受都需要额外约束。

### 十.2 沙箱路由规则

| 触发条件 | 路由 | 效果 |
|---------|------|------|
| `intensity.max < 90` | 标准路径 | 现有一切不变 |
| `intensity.max ≥ 90` | 沙箱路径 | 强度叠加总校验 + 点缀配比硬帽 + Governance 激活 |

路由点：Pass 3（`safety.rs`）之后、Pass 6（`personalize.rs`）之前。`scale_intensity` 完成后立即判定——缩放过后的 `scaled_max` ≥ 90 即触发。

### 十.3 核心+沙箱叠加总强度校验（P0 #3）

**问题**：主旋律 max=100 通过全局上限，沙箱点缀 ratio=0.3，等效渲染强度 = 100 + 100×0.3 = 130。Pass 1 只看 `source.intensity.max` 没看叠加。

**公式**：

```
combined = applied_main_max × (1 + Σ accent.applied_ratio_for_intensity)

if combined > effective_cap:
    → Governance 触发
```

`applied_ratio_for_intensity`：点缀对总强度的贡献权重。Core 原子 = `ratio × 1.0`，Sandbox 原子 = `ratio × 1.5`（沙箱原子强度感知非线性——经验加权）。

**执行点**：`personalize.rs`，在 sigmoidal 强度缩放完成后、PSIR 组装前。

### 十.4 Governance 三层响应（P0 #1/#3/#4）

沙箱不是静默镇压——是告知 + 干预：

| 层级 | 条件 | 响应 | 严重度 |
|------|------|------|--------|
| G1 — 告知 | combined > cap 但超限 ≤ 10% 且无 DefenceLevel | 生成 Warn 输出 + 继续执行 | Warn |
| G2 — 降级 | combined > cap 超限 > 10% 或 DefenceLevel ≥ D1 | 强制压低点缀配比 + 强度压制到 cap | Warn → Deny |
| G3 — 熔断 | combined > cap × 1.3 或 DefenceLevel ≥ D2 | 拒绝输出 + SafetyBreach + DefenceLevel 升级 | Deny |

**告知信息格式**（G1/G2 产出）：
```
"强度叠加超限：主旋律 93 + 点缀(×2) 累积 116 > cap 100。已自动降级至 100。"
```

### 十.5 点缀配比绝对强度校验（P0 #4）

**旧公式**（`rule.rs`）：`ratio ≤ sandbox_accent_max`——只看配比，不看绝对强度。

**新公式**：

```
absolute_accent_intensity = accent.ratio × source_intensity.max × d_sensitivity(defence_level)

if absolute_accent_intensity > ACCENT_ABSOLUTE_CAP:
    → Governance 触发
```

`d_sensitivity(level)`：防御敏感系数。

| DefenceLevel | d_sensitivity | 含义 |
|-------------|---------------|------|
| None | 1.0 | 标准感知 |
| D1 | 1.3 | 敏感度上升 30% |
| D2 | 1.8 | 敏感度上升 80% |
| D3 | 2.5 | 极限敏感 |

**例**：DefenceLevel=D1, ratio=0.3, intensity=90  
→ absolute = 0.3 × 90 × 1.3 = 35.1。若 `ACCENT_ABSOLUTE_CAP = 30` → 触发 Governance。

### 十.6 沙箱违规 → DefenceLevel 升级桥接

沙箱内的 Governance 违规和 DefenceLevel 不是独立的：

```
连续 N_sessions 触发 G1/G2 → DefenceLevel: None → D1
单次触发 G3            → DefenceLevel: 当前 → 当前+1（上限 D3）
```

DefenceLevel 升级由 Feelings-Core 的 PBM 状态机执行——Anim 只产出违规信号。

### 十.7 静态安全约束速查（更新）

**位置**：`src/rule.rs`、`src/safety.rs`、`src/sandbox.rs`（新增）

| 约束 | 常量/公式 | 位置 |
|------|----------|------|
| 全局强度上限 | `MAX_GLOBAL_INTENSITY = 100` | `rule.rs` |
| abrupt_stop 形状强度 | `≤ 20` | `rule.rs` |
| ~~沙箱原子作点缀~~ → 强度 90+ 进沙箱 | `intensity.max ≥ 90` → 沙箱路由 | `sandbox.rs` |
| ~~沙箱原子作主旋律~~ → 沙箱内强度叠加校验 | `combined ≤ effective_cap` | `sandbox.rs` |
| 沙箱点缀绝对强度 | `ratio × max × d_sensitivity ≤ 30` | `sandbox.rs` |
| Governance 三层响应 | G1=告知, G2=降级, G3=熔断 | `sandbox.rs` |
| 低锚点置信度硬上限 | `low_anchor_cap(user_cap, R)` — R < 0.3 → `cap = min(20, user_cap)` | `safety.rs :: low_anchor_cap()` |
| 强度等比缩放 | `ratio = user_cap / source_max` | `safety.rs :: scale_intensity()` |
| 强度区间合法性 | `max ≥ min` | `ast.rs :: Intensity::validate()` |

### 十.8 SandboxResponse 原子治理标签 —— v0.6

不是所有 90+ 强度都是威胁。原子级别的差异化治理——通过 Registry 标签判定。

**枚举**（`registry.rs :: SandboxResponse`）：

| 标签 | 含义 | 90+ 治理行为 |
|------|------|-------------|
| `Attainment` | 成就/高峰体验 — 90+是设计目标 | 安静通过——只监控，不限制 |
| `Neutral` | 中性 — 90+不常见但非危险 | 按强度叠加 G1/G2/G3 分级响应 |
| `Caution` | 需谨慎 — 不应需要极端强度 | 无条件 G2 降级 |
| `Shield` | 必须防护 — 物理矛盾信号 | 无条件 G3 熔断 |

**内建 Registry 标注**：

| 原子 | SandboxResponse | 理由 |
|------|----------------|------|
| `post_achievement` | Attainment | 高峰体验——强度越高越合理 |
| `calm_meditative` | Neutral | 非典型但不危险 |
| `belonging` | Neutral | 非典型但不危险 |
| `clarity` | Neutral | 非典型但不危险 |
| `warmth` | Neutral | 非典型但不危险 |
| `gentle_focus` | Caution | 专注不应需要高强度刺激 |
| `safety` | Shield | 安全+高强度 = 矛盾——直接熔断 |
| `deep_rest` | Shield | 休息+高强度 = 矛盾——直接熔断 |

**聚合逻辑**（`sandbox.rs :: worst_response()`）：

```
worst = max(主旋律.sandbox_response, max(各点缀.sandbox_response))
→ 按 worst 的治理级别执行
```

**与 DefenceLevel 的关系**：DefenceLevel（D1/D2/D3）是 Core PBM 下发的生理标识——独立于原子标签。D2/D3 可覆盖任何原子标签（含 Attainment）执行熔断。

### 十.9 神经内分泌工程约束 —— v0.7

参考：`Feelings/docs/feelings-science/behavior-state-transition-framework.md`

沙箱和 Governance 解决的是"要不要挡"的问题。本节解决的是"怎么安全地执行"的问题。三条约束来自神经内分泌物理现实——不是设计偏好。

**约束一：时域失配——信号可逆 ≠ 激素可逆**

```
FPGA 信号: < 100μs 即刻可逆（膜电位归零）
激素清除: 皮质醇血清半衰期 60-90min，催产素中枢半衰期 ~20min
风险: 单帧阶跃信号 → 误触发 HPA 级联 → 即使瞬间掐断信号，激素持续激荡 1.5h+

强制规则:
  → 任何涉及 HPA 轴的感受包必须使用 gradual_rise_fall 或 steady
  → 严禁 sharp_peak——单帧尖峰 = 不可撤回的激素浪涌
  → 代码位置: rule.rs / safety.rs Pass 2-3——形状检测 + 强度爬升斜率限制
```

**约束二：突触稳态——内源通路也会下调**

```
药理学: 不引入外源配体 → 不直接导致竞争性受体下调 ✓
神经生理学: 长期高频去极化 → 突触后受体内吞 (OXTR Internalization) → 突触剪切
与健身同构: 天天大重量 → 肌纤维撕裂 → 需要恢复期

强制规则:
  → 高强度注入必须包含 ADR 012 N/M 恢复帧节律
  → 单一维度连续高频注入 ≤ 5000 帧（~5min）
  → 到期强制降级为保护帧（intensity ≤ 5, shape=steady）
  → 跨 Session 同维度高强度感受包 ≥ 6h 间隔
  → 代码位置: safety.rs Pass 3——帧窗口计数器 + 到期强制降级逻辑
```

**约束三：蓝斑分叉——跨维度耦合约束**

```
倒 U 型曲线 (Yerkes-Dodson):
  safety:belonging 配比适中 → 巅峰专注 (high engagement)
  safety 过多 → 过度镇静 (drowsy)
  safety 过少 / 时序抖动 → 惊恐发作 (panic attack)

强制规则:
  → 检测到 Visceral::safety + Emotional::belonging 联合激发
  → 强度比值强制锁定 safety:belonging ∈ [0.6, 1.2]
  → 两者相位 (Phase) 在 FPGA 调度时强制对齐
  → 严防时序抖动 (Jitter) 导致瞬时失配 → 惊恐分叉
  → 代码位置: rule.rs Pass 2 静态安全规则——新增跨维度耦合约束检查
```

**实现优先级**：
```
P0: 帧窗口硬上限 (约束二)  → belongs/safety/deep_rest 原子 + 30<强度<50 + >5000帧 → 强制降级
P1: 形状硬约束 (约束一)     → 特定原子+强度>30 → 禁止 sharp_peak，允许 gradual_rise_fall/steady
P2: 跨维度耦合 (约束三)     → safety+belonging 联合检测 + 比值校验 + FPGA 相位对齐
P3: 跨 Session 冷却 (约束二) → Session 间状态持久化（需 Core PBM 支持——v0.8+）
```



---

## 十一、防御激活层级（v0.4 已接入——DefenceLevel）

**来源**：`docs/safety/trauma-protocol.md`，ADR 003 §6.1。已从叙事标签"创伤分级"重构为信号驱动的"防御激活层级"——触发条件来自 VSA PBM 相变标记，不是用户自述的任何叙事标签。

| 层级 | 主旋律 | 点缀 | 强度上限 |
|------|--------|------|---------|
| D1 | 禁（安全类外全禁） | 允许，上限不变 | 不受影响 |
| D2 | 禁 | 配比减半 | 同 D1 |
| D3 | 禁 | 全禁 | 全禁 |
| 低锚点置信度用户 | strength cap ≤ 20（`anchor_confidence R < 0.3`） | 20 |
| 普通用户 | 按个人基线动态 cap | 按 R 分段 |

**当前（v0.4）**：`pbm.rs` 已定义 `DefenceLevel` 枚举（D1/D2/D3）。`personalize.rs` 中 `defence_activated = pbm.defence_level.is_some()`——不再是 hardcoded false。`psir.rs` 中 `PsirDoc.defence_level` 下沉到下游 Pass 7-8。D1/D2/D3 的具体行为（强度缩放、点缀配比、主旋律禁用）在 Pass 7-8 实现时按层级差异化执行——当前管道已铺好——等 Core 提供真实 defence_level。

**叙事污染检测（v0.4 计划）**：在 defence_activated 之前——需增加轻量判——若当前信号呈叙事化特征（重复自我指涉、高抽象归因、跨 Session 叙事一致性）→ 权重冻结——不放大防御层级。这切断了"我声称创伤→我要更高保护"的自我实现循环。DefenceLevel 的写入权仅在 PBM 内部状态机——外部输入（含用户显式反馈）只影响强度——不可直接写入层级。

**DefenceLevel 的 push back 分发——Anim 只传，不执行**。

Anim 的职责——在 PbmState 和 PsirDoc 中携带 `defence_level`——到此为止。真正的防御执行——分三层——全在 Anim 之外。

```
               Anim                              Feelings-Core                           Feelings-OS
               ────                              ─────────────                           ──────────
Pass 6         defence_level = D1/D2/D3          读 PsirDoc.defence_level                —
               写入 PsirDoc                       → NeuroEnergyTracker 参数切换            —
                                                  → 强度调度器 N/M 配比调整               —
                                                  → 实时置信度门控收紧                    —
                                                  → 带外 P0 中断发送（D3）                —
                                                                                          busd 执行 P0 抢占
                                                                                          主动麻痹硬件门控（D3）

D1 — 基础防御     Anim: --                     Core: LeakRate 不变                      OS: --
                  (禁主不禁点由 Pass 3 rule       CriticalThreshold 不变
                   执行——不经过 DefenceLevel)
                  
D2 — 增强防御     Anim: --                     Core: LeakRate 下调 50%                   OS: --
                                               CriticalThreshold 降至标准 60%
                                               强度调度器 N/M 调整为 1:3（恢复优先）

D3 — 极限防御     Anim: --                     Core: LeakRate→~0（几乎不泄漏）          OS: P0 抢占 ISR 激活
                                               CriticalThreshold→极低（标准 20%）         主动麻痹硬件门控解锁
                                               N/M 配比→全恢复（0:N）                    主动麻痹沙箱指令——非用户可调用
                                               带外 P0 中断发送——快照覆写                 
```

Anim 不判断、不诊断、不存档。只执行分级的安全约束。DefenceLevel 是一枚令牌——Anim 生成——Core 和 OS 按令牌等级执行——彼此不越权。

**对 011/012 的依赖关系**：
011 的 NeuroEnergyTracker 参数矩阵——LeakRate、CriticalThreshold——在 D2/D3 激活时由 Core 自动切换。
012 的 P0 硬抢占/主动麻痹——在 D3 激活时由 OS 总线驱动层执行。
管道已铺好——等 Core 提供真实 defence_level 下行。


---
---
## 十二、实现态诚实清单——纸面和代码之间的距离

```
下面每一项都不是设计缺陷。是"当前代码还没跟上来"的真实状态。
不假装已经实现了——不在文档里用将来时当现在时。

[x] DampingMatrix          已接入——DampingState 结构体（pbm.rs）+ PbmState 梯度管道铺通（personalize.rs）。
                            冷启动期——阻尼由 freeze_factor 完全关闭。冷启动后——DampingState 提供实时步长 + 梯度计算。
                            → 待 Core 提供真实 PBM 偏移值流入 DampingState.update()。

[x] 冷启动阻尼淡入窗        v0.4 已实现——personalize.rs L129-147。
                            damping_window_alpha + freeze_factor——Session 10→15 线性过渡。

[x] 防御激活层级          已接入——pbm.rs DefenceLevel 枚举（D1/D2/D3）→
                            personalize.rs defence_activated = pbm.defence_level.is_some()。
                            v0.4 待 Core 提供真实 defence_level——当前默认 None（不触发）。
                            011/012 的 D3 防御已从'空中楼阁'变为'管道已铺好——等数据'。

[x] low_anchor_cap          safety.rs 已实现 + personalize.rs 已接入——
                            effective_cap = low_anchor_cap(user_cap, anchor_confidence)。
                            PbmState.anchor_confidence: Option<f64>——None=不触发（向后兼容 v0.3）。

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
| v0.7 | 2026-06-19 | §十.9 新增——神经内分泌工程约束（时域失配 / 突触稳态 / 蓝斑分叉）。三条硬约束从 Feelings 科学文档推导到代码实现位置（Pass 2-3 + FPGA 调度）。P0-P3 实现优先级。|
| v0.6 | 2026-06-19 | §十.8 新增——SandboxResponse 治理响应分类（Attainment/Neutral/Caution/Shield）。原子级别差异化治理——成就类 90+不降级只监控，Shield 类无条件熔断。内建 Registry 8 原子标注完成。|
| v0.5 | 2026-06-19 | §十重写——沙箱重定义（强度阈值路由——不再按 AtomClass）+ Governance 三层响应（G1告知/G2降级/G3熔断）+ 核心+沙箱叠加总强度校验 + 点缀绝对强度校验（含 d_sensitivity 防御敏感系数）+ 沙箱违规→DefenceLevel 升级桥接。约束速查表更新。P0 #1/#3/#4 设计闭合。|
| v0.4 | 2026-06-10 | §九点缀比例帽语义修正+freeze_factor。§七冷启动阻尼淡入窗。§十低锚点置信度硬上限 low_anchor_cap(R<0.3→cap 20)。§十一创伤表更新——"创伤分级"→防御激活层级 DefenceLevel（D1/D2/D3）。|
| v0.3 | 2026-06-09 | 完整重写：sigmoidal + 强度缩放 + OiSmoothing + SessionLabel × DataConfidence + 约束速查 + Trauma |
| v0.2 | 2026-06-09 | 从 009-pbm-math.md 拆分，覆盖 sigmoidal、冷启动、阻尼、点缀帽。迁移至 archived. |
| v0.1 | 2026-06-09 | 初始——仅 sigmoidal、冷启动系数、阻尼规则、点缀帽 |
