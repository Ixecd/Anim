# FORGET.md — 待修复项（P0 + P1 + P2）

> 扫描日期：2026-06-19
> 范围：代码（src/ 25 模块，187 测试）+ 设计文档（15 ADR）
> 原则：P0 = 生产命门。P1 = 功能受限。P2 = 代码质量/可维护性。
> 命名：animi 是交织器（interlinker），不是编译器。
> 版本：v1.0 已打 tag——自举前唯一 tag。自举完成前不再打任何 tag。

---

## P0 — 生产命门（7/12）

1. **三层安全防线不全 — DefenceLevel 判定逻辑空桩** — `pbm.rs` 已定义 `DefenceLevel` 枚举（D1/D2/D3），但判定触发条件空白。等控器的 VSA 相变标记长什么样？PBM 内部状态机凭什么从 None 升到 D1？只能来自等控器自己的生理信号——不能是用户自述的叙事标签（叙事污染检测）。**部分设计闭合：** ADR 009 §十.4 Governance G1/G2/G3 已定义 DefenceLevel 参与治理的逻辑 + §十.6 沙箱违规→DefenceLevel 升级桥接。→ **归属 Core。** 真正触发条件需 Core PBM 数据。

2. **Session 中用户状态突变安全盲区** — PSIR 只在启动时校验一次。运行中 cap 变了但旧参数继续输出。需 10Hz 轻量安全看门狗。→ **归属 Core。**

3. ~~**混合原子包强度叠加绕过沙箱**~~ ✅ — 沙箱已从原子分类模型重构为强度阈值路由（`intensity ≥ 90 → 自动进沙箱`）。ADR 009 §十.3 定义叠加总强度校验公式 `combined = main × (1 + Σ accent_ratio × weight) ≤ cap`。Governance 三层响应（G1/G2/G3）+ SandboxResponse 原子标签。代码：`src/sandbox.rs`。**设计闭合，代码已落。**

4. ~~**DefenceLevel 激活用户的点缀配比绝对强度**~~ ✅ — ADR 009 §十.5 定义公式 `absolute = ratio × max × d_sensitivity(level) ≤ ACCENT_ABSOLUTE_CAP (30)`。d_sensitivity: None=1.0, D1=1.3, D2=1.8, D3=2.5。`src/sandbox.rs :: check_accent_absolute()`。**设计闭合，代码已落。**

10. ~~**时间窗口累积能量安全盲区**~~ ✅ — `NeuroEnergyTracker` 四维独立漏桶已实现。非线性泄漏（防 PWM）、时钟挂起防护、绝对不应期保护窗、跨维度耦合 σ、全局总耦合能耗漏桶全部落地。`src/safety.rs` + ADR 011。

11. ~~**同一强度持续过久——神经通路适应性脱敏**~~ ✅ — 信号变异度检测（500 帧 δ<2 → `degraded=true`）已接入 `personalize()`。`RhythmTemplate` 类型定义完成（N 帧高强度 + M 帧恢复——Core 零代码，不下发）。`shape=steady` 例外桩就绪。`src/personalize.rs` + ADR 012。

12. ~~**user_cap 静态单一 vs 动态个人基线**~~ ✅ — `user_cap` 通过 `--cap` + `--config` 外部注入。`low_anchor_cap` 从 `config.caps.low_anchor_boundary` / `config.caps.low_anchor_cap` 取值。YAML config 系统完成。ADR 012。

5. ~~**oi 帧缺失无平滑处理**~~ ✅ — `OiSmoothing` 衰减曲线，强度≤hard_cut_boundary 硬截断，>boundary 走 `config.oi_smoothing.decay_sequence`。写入 `FsirDoc.smoothing`。`src/guard.rs` + `src/fsir.rs`。

### 架构

6. ~~**异常终止 Session 数据污染**~~ ✅ — `SessionLabel` + `DataConfidence` + `PbmUpdateStrategy`。`src/pbm.rs`。

7. ~~**新用户预测器冷启动灾难**~~ ✅ — `ColdStartGuard`，前 `config.cold_start.sessions_threshold` 次 Session 强制关闭预测器。`src/pbm.rs`。

8. ~~**跨维度阻尼矩阵无具体参数**~~ ✅ — `DampingMatrix` 参数表已实现。梯度阈值从 `config.damping` 读取。`src/pbm.rs`。

9. ~~**FSIR 跨语言 ABI 零设计**~~ ✅ — Postcard 二进制格式。`to_binary`/`from_binary`。ADR 008。`src/fsir.rs`。

---

## P1 — 功能受限（13/20）

### 交织管线

10. **四层 IR 全链路骨架完成** — FSIR/PSIR/DSIR/ESIR 全部有类型定义 + 序列化。PSIR 有完整 personalize。DSIR 有 DeviceMap 骨架（ear only v0.3）。ESIR 有 CodeGen 骨架（6 种 shape + Postcard 二进制）。完整 IR 分层落地。→ **归属 Core。**

11. **Pass 6-8（Personalize/DeviceMap/CodeGen）** — Pass 6 已完成。Pass 7-8 骨架已完成（v0.3——`.anim → .esir` 端到端跑通）。→ **归属 Core。**

12. **PBM 地基就绪，完整冷启动未实现** — `pbm.rs` 已有全部地基类型。四维差异化冷启动系数 + sigmoidal 收敛因子三实时置信度待 v0.3。→ **归属 Core。**

### 设备

13. **设备热插拔安全重校验缺失** — 设备断开重连后 DSIR 不重做安全校验。→ **归属 Core。**

14. **outline 模式安全约束缺失** — 缺设备时降级行为未定义。→ **归属 Core。**

15. **多设备时钟同步与无线抖动零设计** — 边界归 Feelings-OS `timerd` + `busd`。Anim 侧只定义 ESIR 帧时序约束（1ms 帧周期），不实现 Jitter Buffer。→ **归属 Feelings-OS。**

### 语法

16. ~~**Anim 宏系统零代码**~~ ✅ — `src/macros.rs`。`macro_rules!` 源码级展开 + 参数替换 + 最大递归深度 32。Pass 0 后、Pass 1 前展开。5 测试全绿。ADR 005。

17. ~~**宏递归组合风险绕过**~~ ✅ — 宏展开后 `validate_expanded()` 校验：拒绝超大源码 (>1MB)。拒绝递归组合爆炸。`src/macros.rs`。

~~21. **科学计数法浮点字面量不支持**~~ ✅ — `src/lexer.rs`。

~~22. **Registry 哈希依赖原子顺序**~~ ✅ — 先排序再 SHA-256。`src/registry.rs`。

~~23. **Pass 6 基线偏移硬编码情绪维度**~~ ✅ — `AtomEntry.dimension: PbmDimension`。`src/pbm.rs`。

~~24. **Shape 校验不对称**~~ ✅ — `Registry::shape_names()` 实例方法。`src/registry.rs` + `src/typeck.rs`。

~~25. **DampingMatrix 在 Pass 6 中被架空**~~ ✅ — `damping_gradients: Option`。`src/personalize.rs`。

~~26. **damping_gradients=None 信号缺失降级策略缺失**~~ ✅ — `PbmState.previous_frozen` + Damping Hold。`src/personalize.rs`。

~~27. **点缀冻结维度判定硬编码 Tactile**~~ ✅ — 每原子按自身维度判定。`src/personalize.rs`。

~~28. **主旋律阻尼维度硬编码 Emotional**~~ ✅ — `atom_dimension(&fsir.mix.main)`。`src/personalize.rs`。

~~29. **冷启动阻尼断崖**~~ ✅ — `cold_start_window` + `freeze_factor` α 线性过渡。`src/personalize.rs`。

### 缓存

18. ~~**后台预编译 FSIR 缓存失效策略缺失**~~ ✅ — `FsirDoc::is_cache_valid()`：校验 registry_hash + safety_rules_version 双条件。任一不匹配 → 缓存失效。`SAFETY_RULES_VERSION` 常量。`src/fsir.rs`。`SAFETY_RULES_VERSION` 常量。`src/fsir.rs`。

### 可观测性

19. **运行期 oi 可观测性为零** — → **归属 Feelings-OS。**

### 代码功能

~~20. **错误信息无文件名**~~ ✅ — thread-local `CURRENT_FILE`。

---

## P2 — 代码质量 / 可维护性（2/2）

25. **log.rs 全局日志级别使用 Relaxed 内存顺序** — 单线程 CLI 不触发。需升级为 `Release`/`Acquire`。→ **归属 Anim。**(已闭合—[StepState;4] 固定数组)

29. ~~**PbmDimension HashMap 可换固定数组**~~ ✅ — 4 枚举值 SipHasher 开销。当前 1ms/帧不构成瓶颈。→ **归属 Anim。**(已闭合—[StepState;4] 固定数组)

~~25. **无日志系统**~~ ✅ — `A_info!/A_warn!/A_error!` 宏。

~~26. **错误路径测试覆盖不足**~~ ✅ — 空源/纯空白/纯注释/空文件/缺少 main/缺少 mix。

~~27. **无版本兼容性检查**~~ — 不需要。通过 Registry 哈希 + FSIR JSON 保证兼容。

---

## 已完成（v1.1 → v1.2 途中）

### Pass 管线
- ✅ Pass 0a 词法（lexer.rs）— 关键字/标识符/数字/标点/注释/行号 + 科学计数法
- ✅ Pass 0b 语法（parser.rs）— 递归下降，顺序无关，尾逗号
- ✅ Pass 1 类型检查（typeck.rs）— Registry 8 原子+5 shape + max_ratio + 建议算法
- ✅ Pass 2 静态安全（rule.rs）— 全局上限 / abrupt_stop / 沙箱禁主。已降为 `static_safety` hook
- ✅ Pass 3 用户安全（safety.rs）— scale_intensity + low_anchor_cap。Config 驱动
- ✅ Pass 4 运行期插桩（guard.rs）— OiSmoothing 衰减曲线。已降为 `oi_smoothing` hook
- ✅ Pass 5 FSIR（fsir.rs）— JSON + Postcard 双格式，双哈希
- ✅ Pass 6 Personalize（personalize.rs）— FSIR × PBM → PSIR。sigmoidal 缩放 + DampingMatrix + ColdStartGuard + DefenceLevel
- ✅ FSIR → ESIR 全链路 — Pass 7 DeviceMap + Pass 8 CodeGen（v0.3 骨架）
- ✅ 四层 IR 全部落地 — DSIR（DeviceSet/DeviceAssignment/DsirDoc）+ ESIR（EsirFrame/EsirDoc）

### 安全体系（ADR 011/012）
- ✅ `NeuroEnergyTracker` — 四维独立漏桶。非线性泄漏 + 不应期 + 跨维度耦合 σ + 全局桶
- ✅ `UserSafetyProfile` — 六种 Profile 参数矩阵（Standard/LowAnchor/D1/D2/D3）
- ✅ 脱敏检测 — 信号变异度（500 帧 δ<2 → degraded）+ steady 例外桩
- ✅ `RhythmTemplate` — 节律模板类型定义（Core 零代码，不下发）
- ✅ `AnimiError::SafetyBreach` — 时域能量熔断专用错误变体
- ✅ `PsirDoc.degraded` — 脱敏降级标志

### 基础设施
- ✅ YAML 配置系统 — `configs/default.yaml`（67 参数 10 段）+ `src/config.rs` + `--config`
- ✅ ADR 013 Pipeline + Hook 架构 — `src/pipeline.rs`（7 hooks）+ PipelineStage 6 阶段
- ✅ Postcard 二进制 ABI（ADR 008）
- ✅ PBM 地基 — SessionLabel / DataConfidence / ColdStartGuard / DampingMatrix / DampingState
- ✅ Registry 外部化 + 哈希排序 + 科学计数法浮点 + Shape 校验对称
- ✅ oi 帧平滑过渡 — OiSmoothing + decay_sequence 从 config 读取
- ✅ 源码 SHA-256 — SPL 锚定就绪
- ✅ 错误码自动生成 — build.rs → docs/error-codes.md
- ✅ 157 单元测试全绿，clippy 零 warning
- ✅ ADR 014 oi 三层严重度 — oi!/oi_warn!/oi_note! + Severity Deny/Warn/Note + --strict/--verbose
- ✅ Anim 宏系统 — `src/macros.rs` macro_rules! 源码级展开（ADR 005）
- ✅ 全部硬编码数字已迁移到 configs/default.yaml（67 参数 10 段）

---

## 设计决策

### D01: 包名与感受原子解耦——三层结构

```
feeling <基本感受包名> {
    mix {
        main: <基本感受的某种细分变体>
        accents: [<点缀1> <配比>, <点缀2> <配比>, ...]
    }
    shape: <时间形状>
    intensity: [<min>, <max>]
}
```

创作者先选基本感受——再选具体变体——再加点缀和形状。Anim 不替创作者决定——只保证类型正确+安全。

### D02: Pipeline + Hook 架构（ADR 013）

管线核心只做数据流转——每个安全/检测/校准机制 = 独立的 `PipelineHook`，在 `PipelineStage` 注入点注册。Hook 从 `config.hooks` YAML 段构建，Session 启动时一次性初始化，运行期不变。全部静态组合，零虚函数开销。

---

## 编辑记录
```

2026-06-24  v0.6 Anim P1 全清 + Core 泛型重构 + LANGUAGE.md 对齐
            - Anim: P1 #17 宏递归闭合 / P1 #18 FSIR缓存闭合 / P2 #25 log内存有序闭合
              192 tests green, 零 clippy 警告。Anim 职责收敛: .anim → FSIR。
            - Core: FeelingTarget trait + NeuroEnergyTracker<D,S> + Session<D,S> 全线泛型化。
              维度数编译期展开。D3 主动麻痹锚点留位。PersonalityAnchor 舱位已开。
              CoreConfig 全面参数化+validate_dimensions。CLI --species 默认 human。
              20 tests green。P0 0/7→泛型地基就位。
            - Feelings: LANGUAGE.md trauma→defence 全量清除。沙箱→多类型 Governance。
              Registry 原子名全量对齐当前 8 原子。教练 AI 混合部署 (本地FPGA+云端深度推理)。

2026-06-22  v0.5 Core 迁移里程碑 — Feelings-Core Rust 项目初始化
            - Feelings-Core: 5 模块 (pbm/personalize/tracker/session/dsir) + Cargo.toml
            - Anim: src/core/mod.rs — Core 职责蓝图 + 迁移路线文档
            - FORGET/HANDOFF/009/012 PROTOCOL 闭源 → MIT 开源措辞全局修正
            - 下一步: Anim pbm.rs/safety.rs/personalize.rs 代码迁入 Core

2026-06-19  v0.4 Sandbox + Governance 完整重构 — 告别阻断，拥抱引导
            - ADR 009 §十 完全重写：沙箱 = 强度阈值路由（≥90）+ GovernanceAction 引导感受
            - SandboxResponse 原子标签：Attainment/Neutral/Caution/Shield，8 原子内建标注
            - GovernanceAction（PassThrough/Steer/Redirect/Anchor）替代 Error 返回值
            - ADR 009 §十.9 神经内分泌工程约束：时域失配 / 突触稳态 / 蓝斑分叉
            - src/sandbox.rs：187 测试，check_combined/check_accent_absolute + worst_response/strictest
            - P0 #3/#4 设计闭合+代码落地。P0 #1 部分设计闭合。P0 计数 11/12 → 9/12。
            - 25 模块，187 测试，clippy 零 warning。

2026-06-17  v0.3.1 Anim 宏系统落地 — macro_rules! 源码级展开
            - src/macros.rs — extract_macros + expand_macros + 5 测试
            - main.rs: 宏展开在 Pass 0 后、Pass 1 前
            - eg/macro-calm.anim — 端到端示例
            - P1 #16 闭合。157 tests，24 模块。

2026-06-15  v0.3  Pass 7 + Pass 8 骨架 — FSIR → ESIR 全链路闭合
            - src/dsir.rs + src/device_map.rs — Pass 7: PSIR × DeviceSet → DSIR
            - src/esir.rs + src/codegen.rs — Pass 8: 6 种 shape → ESIR 帧序列 + Postcard 二进制
            - FsirDoc::to_psir_stub() — 离线 CLI 模式 PSIR 桩
            - main.rs: .anim → .json → .dsir.json → .esir 端到端
            - ADR 015: 设计文档定稿
            - 152 tests，23 模块，15 ADR
            - P1: 4/20 → 12/20 (#10/#11 闭合)

2026-06-15  v0.1.33 ADR 014 oi 三层严重度 + FORGET 刷新 + 全部硬编码回填 config
            - oi!/oi_warn!/oi_note! + Severity Deny/Warn/Note + --strict/--verbose
            - rule.rs: abrupt_stop + sandbox_accent 从 oi! 降为 oi_warn!
            - 140 tests，14 ADR（新增 014）

2026-06-15  v0.1.32 硬编码数字全量回填 config + ADR 014 oi 三层严重度
            - NeuroEnergyTracker::from_config(&LeakyBucketConfig)
            - ColdStartGuard::from_config(&ColdStartConfig)
            - DampingState/DampingMatrix 全部从 DampingConfig 读取梯度阈值+初始步长+EMA
            - DataConfidence::step_multiplier 从 DataConfidenceConfig 读取
            - configs/default.yaml 新增 data_confidence 段（high_step/low_step/contaminated_step/step_floor/step_ceiling)
            - ADR 014: oi!/oi_warn!/oi_note! + Severity Deny/Warn/Note + --strict/--verbose
            - rule.rs: abrupt_stop + sandbox_accent 从 oi! 降为 oi_warn!

2026-06-15  v0.1.31 ADR 011+012 全线落地 + YAML 配置系统 + ADR 013 Pipeline/Hook 架构
            - P0 #10/#11/#12 闭合：NeuroEnergyTracker + 脱敏检测 + dynamic cap 配置化
            - YAML config 系统：configs/default.yaml（60 参数）+ src/config.rs + --config CLI
            - ADR 013：PipelineStage 6 阶段 + PipelineHook trait + 7 hooks + main.rs 接入
            - 137 tests，19 模块（新增 config.rs + pipeline.rs），13 ADR（新增 013）
            - AnimiError::SafetyBreach + PsirDoc.degraded + RhythmTemplate 类型定义
            - 全部硬编码数字迁移到 YAML config

2026-06-12  v0.1.30 P1b 老化抢占 + PBM 硬件写锁——ADR 011/012 本轮 review 闭合

2026-06-12  v0.1.29 ADR 011 + ADR 012 定稿——P0 #10 漏桶 + P0 #11 #12 脱敏/dynamic cap

2026-06-12  v0.1.28 P1 #22 + #24 —— Registry 哈希排序 + Shape 校验对称

2026-06-12  v0.1.27 P1 #21 科学计数法浮点字面量

2026-06-10  v0.1.26 P0 #10 ADR 011 扩展——按维度漏桶 + Profile动态参数 + Pass7指数衰减包络

2026-06-10  v0.1.25 P0 #1 low_anchor_cap——锚点置信度 R<0.3→cap20 硬线

2026-06-10  v0.1.24 P0 #11 + #12——脱敏与动态基线cap

2026-06-10  v0.1.23 P0 #10 升级——Neuro-Leaky Bucket + EnergyTracker + 保护帧 + 三层纵深

2026-06-10  v0.1.22 持续低强度累积——P0 #10 时间窗口累积能量安全盲区

2026-06-10  v0.1.21 点缀比例帽语义修正 + 冷启动阻尼淡入窗
            - P1 #29 闭合
            - P2 #29 新增

2026-06-10  v0.1.20 豆包 review #2 闭合——阻尼冻结维度三漏洞全部 fix
            - P1 #26/#27/#28 闭合

2026-06-09  v0.1.19 豆包 review #2——damping_gradients悬空 + 点缀/主旋律冻结漏网

2026-06-09  v0.1.18 FORGET 刷新——Pass 6 闭合

2026-06-09  v0.1.17 豆包 review——Pass 6 三漏洞修复

2026-06-04  v0.1.16 P1 review round 3 —— smoothing→FSIR + DampingMatrix HashMap

2026-06-03  v0.1.15 P0 第1轮——5/9

2026-06-01  v0.1.14 P0/P1/P2 重构

2026-06-01  v0.1.13 深度架构审计——12个设计盲区

2026-06-01  v0.1.12 强度刻度设计决策

2026-06-01  v0.1.11 感受的全面性——外部锚点感受也是感受

2026-06-01  v0.1.10 Stream First 架构对齐

2026-06-01  v0.1.9 设计审计——ADR003 八→九 Pass，ADR004 管线图补 Pass 4

2026-05-30  v0.1.8 xattr 文件元数据

2026-05-30  v0.1.7 FPGA 为什么无处不在但你看不见

2026-05-30  v0.1.6 FPGA 计算存储一体

2026-05-30  v0.1.5 第二纪元模拟层——ZENO 生物模拟+物理求解器

2026-05-30  v0.1.4 实现路径更新——可视化编辑器 ZENO 节点图

2026-05-30  v0.1.3 技术审计——智能指针/指令集/FSIR JSON/驱动边界

2026-05-27  v0.1.2 MiniMax + Gemini 审计（11项）

2026-05-27  v0.1.1 初始扫描——架构规范 100%，代码 0%
```
