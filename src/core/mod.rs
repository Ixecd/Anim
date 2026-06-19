// src/core/mod.rs — Feelings-Core 预留模块
//
// 当前 Anim 替 Core 承担了运行时职责：PBM 基线、NeuroEnergyTracker、
// Personalize 个人校准、ColdStartGuard、Session 管理、Pipeline Hook 调度。
//
// Core 出生前——这些实现在 Anim 编译。Core 出生后——这里变成接口定义层，
// Anim 通过 trait 调用 Core，不持有运行时状态。
//
// 迁移路线 (v0.7 → v1.0):
//   第一步: 类型定义留在 Anim，通过 trait 抽象让 Core 提供实现
//   第二步: 类型定义迁到 Core，Anim 通过 FFI / IPC 调用
//   第三步: Anim 变成纯编译器——编译期不碰任何个人数据

// ═══════════════════════════════════════════════════════════════
// 一、PBM — 个人基线矩阵（Personal Baseline Matrix）
// ═══════════════════════════════════════════════════════════════
//
// 当前位置: src/pbm.rs
// Core 职责:
//   - PBM 存储与加密 (设备本地，数据不离设备)
//   - 冷启动守护 (ColdStartGuard)
//   - 阻尼矩阵实时梯度 (DampingState)
//   - 四维 PBM 偏移值 [Visceral, Emotional, Tactile, Auditory]
//   - 数据置信度标注 (DataConfidence)
//   - Session 标签 (SessionLabel → PbmUpdateStrategy)
//   - DefenceLevel 判定 (VSA 相变触发→D1/D2/D3)
//
// Anim 保留:
//   - DampingMatrix 静态映射表 (跨维度冻结规则——不依赖个人数据)
//   - PbmDimension 枚举 (四维度——共享类型)

// ═══════════════════════════════════════════════════════════════
// 二、Personalize — FSIR → PSIR 个人校准
// ═══════════════════════════════════════════════════════════════
//
// 当前位置: src/personalize.rs
// Core 职责:
//   - FSIR × PBM → PSIR (个人基线校准)
//   - sigmoidal_scale() (个人强度缩放——需要 PBM 系数)
//   - PbmColdStartCoefficients (四维差异化冷启动系数)
//   - 冷启动阻尼淡入窗 (damping_window_alpha + freeze_factor)
//   - 冻结维度判定 (is_frozen——需要实时 PBM 数据)
//   - 脱敏检测 (信号变异度——需要跨帧状态)
//   - leaky bucket intake (需要 Session 级 tracker 状态)
//
// Anim 保留:
//   - FsirDoc → PsirDoc 类型转换 (不含个人数据的部分)
//   - 点缀配比帽裁切 (基于 Registry，不基于个人数据)

// ═══════════════════════════════════════════════════════════════
// 三、NeuroEnergyTracker — 四维漏桶
// ═══════════════════════════════════════════════════════════════
//
// 当前位置: src/safety.rs
// Core 职责:
//   - NeuroEnergyTracker 实例管理 (Session 级状态)
//   - intake_and_verify() (实时帧级能量追踪)
//   - verify_cross_dimension() (跨维度耦合检测)
//   - 不应期管理 (refractory_active / refractory_counter)
//   - 全局桶 (global_alpha / global_beta)
//
// Anim 保留:
//   - 漏桶参数配置 (从 YAML config 读取——编译期已知)
//   - effective_leak_rate 公式 (纯数学——不依赖个人数据)

// ═══════════════════════════════════════════════════════════════
// 四、Session — 会话管理
// ═══════════════════════════════════════════════════════════════
//
// 当前位置: 分散在 pbm.rs / safety.rs / personalize.rs / pipeline.rs
// Core 职责:
//   - Session 生命周期 (start / end / abort)
//   - Session 内用户 cap 管理 (是否变更，是否受 DefenceLevel 影响)
//   - 跨 Session 状态持久化 (6h 冷却 / 连续违规计数 → DefenceLevel 升级)
//   - 帧窗口计数器 (5000 帧硬上限——约束二)
//   - 跨帧状态: previous_intensities / monotony_counters
//
// Anim 保留:
//   - 编译期约束校验 (形状硬限制 / 跨维度耦合检测——不需要 Session 状态)

// ═══════════════════════════════════════════════════════════════
// 五、Pipeline — 管线调度
// ═══════════════════════════════════════════════════════════════
//
// 当前位置: src/pipeline.rs
// Core 职责:
//   - Hook 运行时调度 (Session 启动→Hook 初始化→帧级 Hook 调用)
//   - Hook 生命周期 (启用/禁用——基于 Session 上下文)
//
// Anim 保留:
//   - 管线阶段定义 (PipelineStage 枚举——编译期划分)
//   - 静态 Hook (static_safety——不依赖个人数据)

// ═══════════════════════════════════════════════════════════════
// 六、Core trait 接口 (v1.0 目标)
// ═══════════════════════════════════════════════════════════════
//
// Core 出生后，Anim 通过以下 trait 调用 Core，不持有运行时状态：
//
//   trait CorePbm {
//       fn anchor_confidence(&self, user_id: &str) -> Option<f64>;
//       fn defence_level(&self, user_id: &str) -> Option<DefenceLevel>;
//       fn cold_start_coefficients(&self) -> PbmColdStartCoefficients;
//       fn damping_state(&self) -> Option<DampingState>;
//       fn session_count(&self) -> u32;
//   }
//
//   trait CoreSession {
//       fn start_session(&mut self, config: &SessionConfig) -> SessionId;
//       fn end_session(&mut self, id: SessionId, label: SessionLabel);
//       fn tracker(&mut self) -> &mut NeuroEnergyTracker;
//       fn frame_window(&self, dim: PbmDimension) -> u32;
//   }
//
//   trait CorePersonalize {
//       fn personalize(&self, fsir: &FsirDoc, user_id: &str) -> Result<PsirDoc>;
//   }
//
// 当前 (Core 零代码): 所有 trait 由 Anim 内部默认实现替代。
