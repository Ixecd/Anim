// src/safety.rs — Pass 3：用户安全 + ADR 011/012 漏桶/脱敏
//
// 对这个人的拒绝。读用户档案——防御激活层级、强度 cap、锚点置信度。
// 必须在设备本地运行——用户穿戴并开始 Session 时。
//
// v1.1 桩——全部通过。后续版本接入：
//   - 强度等比缩放（scale_intensity）
//   - 防御激活交叉判定
//   - 强度上限 cap（如低锚点置信度 max 20）
//   - 防御原子黑名单（禁主不禁点）
//   - 触觉维度锁定
//
// v0.4: 未成年不等于年龄。成年 = 锚在里面（R > 0.3），不是法律上的 18 岁。
//   `anchor_confidence` 由 Feelings-Core PBM 提供——Anim 只做硬上限。
//   见 Feelings docs/society/adulthood-as-anchor.md。

use crate::ast::*;
use crate::config::AnimConfig;
use crate::error::{AnimiError, Severity};
use crate::pbm::{DefenceLevel, PbmDimension};

// ── Pass 3 公共 API ──────────────────────────────────────────

/// 对源码执行用户安全规则检查。
///
/// v1.1——全部通过。后续版本需要加载用户档案。
pub fn check(_source: &FeelingSource) -> Result<(), AnimiError> {
    Ok(())
}

/// 检查 + 强度缩放——返回缩放后的 Interval。
pub fn check_with_scale(
    source: &FeelingSource,
    user_cap: u32,
    _config: &AnimConfig,
) -> Result<Intensity, AnimiError> {
    check(source)?;
    Ok(scale_intensity(&source.intensity, user_cap))
}

/// 低锚点置信度 cap —— 硬上限 20。
///
/// 不是"未成年"——不是年龄。是锚点在外面的人——还没学会自己管住自己。
/// anchor_confidence = Feelings-Core PBM 提供的内部锚点成熟度置信度 R (0.0~1.0)。
/// R < 0.3 → cap = min(20, user_cap)。不是惩罚——是"你还没准备好——我先替你守着"。
///
/// v0.4: Anim 侧只做硬上限。R 由 Core 提供——Anim 不计算。
///       当前默认 None（无数据→不拦截）。v0.4 接 Core 后改为必传。
pub fn low_anchor_cap(user_cap: u32, anchor_confidence: Option<f64>, config: &AnimConfig) -> u32 {
    match anchor_confidence {
        Some(r) if r < config.caps.low_anchor_boundary => user_cap.min(config.caps.low_anchor_cap),
        _ => user_cap,
    }
}

/// 强度等比缩放——当 user_cap < source max 时，等比缩放整个区间。
pub fn scale_intensity(intensity: &Intensity, user_cap: u32) -> Intensity {
    if intensity.max <= user_cap {
        return intensity.clone();
    }
    let ratio = user_cap as f64 / intensity.max as f64;
    Intensity {
        min: (intensity.min as f64 * ratio).round() as u32,
        max: user_cap,
    }
}

// ═══════════════════════════════════════════════════════════════
// ADR 011/012 — UserSafetyProfile + RhythmTemplate
// ═══════════════════════════════════════════════════════════════

/// 安全 Profile 类型——决定漏桶参数矩阵的梯度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileKind {
    Standard,
    LowAnchor,
    DefenceD1,
    DefenceD2,
    DefenceD3,
}

/// 用户安全档案——持有四维漏桶参数 + 节律模板 + 锚点置信度。
///
/// Session 启动时由 Feelings-Core 一次性下发快照——Anim 管线只读——不可变。
#[derive(Debug, Clone, Copy)]
pub struct UserSafetyProfile {
    pub kind: ProfileKind,
    /// 四维独立 LeakRate（强度分/秒）。索引：[Visceral, Emotional, Tactile, Auditory]。
    pub leak_rates: [f64; 4],
    /// 四维独立 CriticalThreshold。
    pub critical_thresholds: [f64; 4],
    pub anchor_confidence: Option<f64>,
    pub defence_level: Option<DefenceLevel>,
}

impl Default for UserSafetyProfile {
    fn default() -> Self {
        UserSafetyProfile {
            kind: ProfileKind::Standard,
            leak_rates: [2.0, 2.0, 2.0, 2.0],
            critical_thresholds: [600.0, 600.0, 600.0, 600.0],
            anchor_confidence: None,
            defence_level: None,
        }
    }
}

impl UserSafetyProfile {
    pub fn standard() -> Self {
        Self::default()
    }

    /// 从 ProfilesConfig 读取指定 profile 的参数。
    pub fn from_config_profile(cfg: &crate::config::ProfilesConfig, kind: ProfileKind) -> Self {
        let p = match kind {
            ProfileKind::Standard => &cfg.standard,
            ProfileKind::LowAnchor => &cfg.low_anchor,
            ProfileKind::DefenceD1 => &cfg.defence_d1,
            ProfileKind::DefenceD2 => &cfg.defence_d2,
            ProfileKind::DefenceD3 => &cfg.defence_d3,
        };
        let (anchor, def_level) = match kind {
            ProfileKind::LowAnchor => (Some(0.2), None),
            ProfileKind::DefenceD1 => (None, Some(DefenceLevel::D1)),
            ProfileKind::DefenceD2 => (None, Some(DefenceLevel::D2)),
            ProfileKind::DefenceD3 => (None, Some(DefenceLevel::D3)),
            _ => (None, None),
        };
        UserSafetyProfile {
            kind,
            leak_rates: p.leak_rates,
            critical_thresholds: p.critical_thresholds,
            anchor_confidence: anchor,
            defence_level: def_level,
        }
    }

    pub fn low_anchor() -> Self {
        Self::from_config_profile(
            &crate::config::ProfilesConfig::default(),
            ProfileKind::LowAnchor,
        )
    }

    pub fn from_defence_level(level: DefenceLevel) -> Self {
        let kind = match level {
            DefenceLevel::D1 => ProfileKind::DefenceD1,
            DefenceLevel::D2 => ProfileKind::DefenceD2,
            DefenceLevel::D3 => ProfileKind::DefenceD3,
        };
        Self::from_config_profile(&crate::config::ProfilesConfig::default(), kind)
    }

    pub fn leak_rate(&self, dim: PbmDimension) -> f64 {
        self.leak_rates[dim_index(dim)]
    }
    pub fn critical_threshold(&self, dim: PbmDimension) -> f64 {
        self.critical_thresholds[dim_index(dim)]
    }
}

/// PbmDimension → [f64; 4] 数组索引。
pub fn dim_index(dim: PbmDimension) -> usize {
    match dim {
        PbmDimension::Visceral => 0,
        PbmDimension::Emotional => 1,
        PbmDimension::Tactile => 2,
        PbmDimension::Auditory => 3,
    }
}

/// 四个维度的迭代顺序——和 [f64; 4] 索引一致。
pub const DIM_ORDER: [PbmDimension; 4] = [
    PbmDimension::Visceral,
    PbmDimension::Emotional,
    PbmDimension::Tactile,
    PbmDimension::Auditory,
];

// ── ADR 012 节律模板 ──────────────────────────────────────────

/// 信号节律模板——ADR 012 §一。"每 N 帧高强度后强制插入 M 帧低强度/静默"。
#[derive(Debug, Clone, Copy)]
pub struct RhythmTemplate {
    pub n_active: u32,
    pub m_recovery: u32,
}

impl Default for RhythmTemplate {
    fn default() -> Self {
        RhythmTemplate {
            n_active: u32::MAX,
            m_recovery: 0,
        }
    }
}

impl RhythmTemplate {
    pub fn new(n_active: u32, m_recovery: u32) -> Self {
        RhythmTemplate {
            n_active,
            m_recovery,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// ADR 011 — NeuroEnergyTracker 四维漏桶
// ═══════════════════════════════════════════════════════════════

/// 四维独立生物漏桶——模拟神经递质重摄取/代谢清除。
#[derive(Debug, Clone)]
pub struct NeuroEnergyTracker {
    cumulative_energy: [f64; 4],
    last_tick_ns: Option<u64>,
    max_dt: f64,
    nonlinear_gamma: f64,
    refractory_active: [bool; 4],
    refractory_counter: [u32; 4],
    refractory_frames: u32,
    sigma: f64,
    global_alpha: f64,
    global_beta: f64,
}

impl Default for NeuroEnergyTracker {
    fn default() -> Self {
        Self::from_config(&crate::config::LeakyBucketConfig::default())
    }
}

impl NeuroEnergyTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 LeakyBucketConfig 构造——所有参数从 YAML config 读取。
    pub fn from_config(cfg: &crate::config::LeakyBucketConfig) -> Self {
        NeuroEnergyTracker {
            cumulative_energy: [0.0; 4],
            last_tick_ns: None,
            max_dt: cfg.max_dt_seconds,
            nonlinear_gamma: cfg.nonlinear_gamma,
            refractory_active: [false; 4],
            refractory_counter: [0; 4],
            refractory_frames: cfg.refractory_frames,
            sigma: cfg.sigma,
            global_alpha: cfg.global_alpha,
            global_beta: cfg.global_beta,
        }
    }

    pub fn with_max_dt(max_dt_seconds: f64) -> Self {
        NeuroEnergyTracker {
            max_dt: max_dt_seconds,
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub fn with_params(
        max_dt: f64,
        gamma: f64,
        refractory_frames: u32,
        sigma: f64,
        global_alpha: f64,
        global_beta: f64,
    ) -> Self {
        NeuroEnergyTracker {
            cumulative_energy: [0.0; 4],
            last_tick_ns: None,
            max_dt,
            nonlinear_gamma: gamma,
            refractory_active: [false; 4],
            refractory_counter: [0; 4],
            refractory_frames,
            sigma,
            global_alpha,
            global_beta,
        }
    }

    pub fn intake_and_verify(
        &mut self,
        intensity: u32,
        dim: PbmDimension,
        profile: &UserSafetyProfile,
        now_ns: u64,
    ) -> Result<(), AnimiError> {
        let idx = dim_index(dim);
        let threshold = profile.critical_threshold(dim);

        if self.refractory_active[idx] {
            self.refractory_counter[idx] = self.refractory_counter[idx].saturating_sub(1);
            if self.cumulative_energy[idx] < 0.5 * threshold {
                self.refractory_active[idx] = false;
                self.refractory_counter[idx] = 0;
            } else if self.refractory_counter[idx] == 0 {
                self.refractory_counter[idx] = self.refractory_frames;
            } else {
                if let Some(last) = self.last_tick_ns {
                    let dt = (now_ns.saturating_sub(last)) as f64 / 1_000_000_000.0;
                    if dt <= self.max_dt {
                        let leak = effective_leak_rate(
                            self.cumulative_energy[idx],
                            threshold,
                            profile.leak_rate(dim),
                            self.nonlinear_gamma,
                        );
                        self.cumulative_energy[idx] =
                            (self.cumulative_energy[idx] - leak * dt).max(0.0);
                    }
                }
                self.last_tick_ns = Some(now_ns);
                return Ok(());
            }
        }

        if let Some(last) = self.last_tick_ns {
            let dt = (now_ns.saturating_sub(last)) as f64 / 1_000_000_000.0;
            if dt <= self.max_dt {
                let leak = effective_leak_rate(
                    self.cumulative_energy[idx],
                    threshold,
                    profile.leak_rate(dim),
                    self.nonlinear_gamma,
                );
                self.cumulative_energy[idx] =
                    (self.cumulative_energy[idx] + intensity as f64 - leak * dt).max(0.0);
            } else {
                self.cumulative_energy[idx] += intensity as f64;
            }
        } else {
            self.cumulative_energy[idx] += intensity as f64;
        }

        self.last_tick_ns = Some(now_ns);

        if self.cumulative_energy[idx] > 0.9 * threshold && !self.refractory_active[idx] {
            self.refractory_active[idx] = true;
            self.refractory_counter[idx] = self.refractory_frames;
        }

        if self.cumulative_energy[idx] > threshold {
            return Err(AnimiError::SafetyBreach {
                severity: Severity::Deny,
                file_name: crate::error::current_file(),
                dimension: dim,
                current_energy: self.cumulative_energy[idx],
                threshold,
            });
        }
        Ok(())
    }

    pub fn verify_cross_dimension(&self, profile: &UserSafetyProfile) -> Result<(), AnimiError> {
        let energies = self.cumulative_energy;

        for i in 0..4 {
            let e_norm = energies[i] / profile.critical_thresholds[i].max(1.0);
            let coupling = self.sigma * e_norm;
            for j in 0..4 {
                if i == j {
                    continue;
                }
                let effective = profile.critical_thresholds[j] * (1.0 - coupling);
                if energies[j] > effective {
                    return Err(AnimiError::SafetyBreach {
                        severity: Severity::Deny,
                        file_name: crate::error::current_file(),
                        dimension: DIM_ORDER[j],
                        current_energy: energies[j],
                        threshold: effective,
                    });
                }
            }
        }

        let energy_global: f64 = DIM_ORDER
            .iter()
            .map(|d| energies[dim_index(*d)])
            .sum::<f64>()
            * self.global_alpha;
        let global_threshold: f64 =
            profile.critical_thresholds.iter().sum::<f64>() * self.global_beta;

        if energy_global > global_threshold {
            return Err(AnimiError::SafetyBreach {
                severity: Severity::Deny,
                file_name: crate::error::current_file(),
                dimension: PbmDimension::Emotional,
                current_energy: energy_global,
                threshold: global_threshold,
            });
        }
        Ok(())
    }

    pub fn energy(&self, dim: PbmDimension) -> f64 {
        self.cumulative_energy[dim_index(dim)]
    }
    pub fn all_energies(&self) -> [f64; 4] {
        self.cumulative_energy
    }

    pub fn reset(&mut self) {
        self.cumulative_energy = [0.0; 4];
        self.last_tick_ns = None;
        self.refractory_active = [false; 4];
        self.refractory_counter = [0; 4];
    }
}

/// 非线性泄漏速率——ADR 011 §非线性泄漏。
/// `LeakRate(E) = LeakRate_baseline × (1.0 + γ × E)`
pub fn effective_leak_rate(cumulative: f64, threshold: f64, baseline: f64, gamma: f64) -> f64 {
    if threshold <= 0.0 {
        return baseline;
    }
    let e_norm = (cumulative / threshold).clamp(0.0, 1.0);
    baseline * (1.0 + gamma * e_norm)
}

/// 获取单调纳秒时钟（当前阶段用 std，v0.5+ 走 Feelings-OS timerd PLL 时钟）。
pub fn monotonic_ns() -> u64 {
    #[cfg(not(test))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
    #[cfg(test)]
    {
        TEST_CLOCK.fetch_add(1_000_000, std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
static TEST_CLOCK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::typeck::TypeChecker;

    // ── Pass 3 tests ──────────────────────────────────────

    fn check_src(src: &str) -> Result<(), AnimiError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let reg = crate::registry::Registry::default();
        let checker = TypeChecker::new(&reg);
        checker.check(&ast)?;
        check(&ast)
    }

    #[test]
    fn safety_stub_passes() {
        let src = r#"
feeling calm {
    mix { main: calm_meditative accents: [] }
    shape: steady
    intensity: [10, 20]
}
"#;
        assert!(check_src(src).is_ok());
    }

    #[test]
    fn low_anchor_confidence_caps_at_20() {
        let cfg = crate::config::AnimConfig::default();
        assert_eq!(low_anchor_cap(100, Some(0.1), &cfg), 20);
        assert_eq!(low_anchor_cap(15, Some(0.1), &cfg), 15);
    }

    #[test]
    fn normal_anchor_confidence_no_cap() {
        let cfg = crate::config::AnimConfig::default();
        assert_eq!(low_anchor_cap(100, Some(0.5), &cfg), 100);
        assert_eq!(low_anchor_cap(100, Some(0.9), &cfg), 100);
    }

    #[test]
    fn no_anchor_data_no_cap() {
        let cfg = crate::config::AnimConfig::default();
        assert_eq!(low_anchor_cap(100, None, &cfg), 100);
    }

    #[test]
    fn low_anchor_boundary() {
        let cfg = crate::config::AnimConfig::default();
        assert_eq!(low_anchor_cap(100, Some(0.3), &cfg), 100);
        assert_eq!(low_anchor_cap(100, Some(0.29999), &cfg), 20);
    }

    #[test]
    fn scale_intensity_no_change() {
        let scaled = scale_intensity(&Intensity { min: 10, max: 20 }, 50);
        assert_eq!(scaled.min, 10);
        assert_eq!(scaled.max, 20);
    }

    #[test]
    fn scale_intensity_proportional() {
        let scaled = scale_intensity(&Intensity { min: 15, max: 60 }, 45);
        assert_eq!(scaled.min, 11);
        assert_eq!(scaled.max, 45);
    }

    #[test]
    fn check_with_scale_applies_cap() {
        let cfg = crate::config::AnimConfig::default();
        let src = r#"
feeling calm {
    mix { main: calm_meditative accents: [] }
    shape: steady intensity: [15, 60]
}
"#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let scaled = check_with_scale(&ast, 45, &cfg).unwrap();
        assert_eq!(scaled.min, 11);
        assert_eq!(scaled.max, 45);
    }

    // ── UserSafetyProfile tests ──────────────────────────

    #[test]
    fn default_profile_is_standard() {
        let p = UserSafetyProfile::default();
        assert_eq!(p.kind, ProfileKind::Standard);
        assert_eq!(p.leak_rate(PbmDimension::Visceral), 2.0);
        assert_eq!(p.critical_threshold(PbmDimension::Visceral), 600.0);
    }

    #[test]
    fn low_anchor_profile_halves_leak_rate() {
        let p = UserSafetyProfile::low_anchor();
        assert_eq!(p.kind, ProfileKind::LowAnchor);
        assert_eq!(p.leak_rate(PbmDimension::Emotional), 1.0);
        assert_eq!(p.critical_threshold(PbmDimension::Emotional), 120.0);
    }

    #[test]
    fn defence_d3_near_zero_leak() {
        let p = UserSafetyProfile::from_defence_level(DefenceLevel::D3);
        assert_eq!(p.kind, ProfileKind::DefenceD3);
        assert!((p.leak_rate(PbmDimension::Visceral) - 0.05).abs() < 1e-10);
        assert_eq!(p.critical_threshold(PbmDimension::Visceral), 30.0);
    }

    #[test]
    fn dim_index_maps_correctly() {
        assert_eq!(dim_index(PbmDimension::Visceral), 0);
        assert_eq!(dim_index(PbmDimension::Emotional), 1);
        assert_eq!(dim_index(PbmDimension::Tactile), 2);
        assert_eq!(dim_index(PbmDimension::Auditory), 3);
    }

    #[test]
    fn rhythm_template_default_is_no_limit() {
        let t = RhythmTemplate::default();
        assert_eq!(t.n_active, u32::MAX);
        assert_eq!(t.m_recovery, 0);
    }

    #[test]
    fn rhythm_template_custom() {
        let t = RhythmTemplate::new(100, 20);
        assert_eq!(t.n_active, 100);
        assert_eq!(t.m_recovery, 20);
    }

    // ── NeuroEnergyTracker tests ──────────────────────────

    #[test]
    fn zero_intensity_does_nothing() {
        let mut t = NeuroEnergyTracker::new();
        let p = UserSafetyProfile::standard();
        t.intake_and_verify(0, PbmDimension::Emotional, &p, 1_000_000)
            .unwrap();
        assert!((t.energy(PbmDimension::Emotional) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn first_frame_only_accumulates_no_leak() {
        let mut t = NeuroEnergyTracker::new();
        let p = UserSafetyProfile::standard();
        t.intake_and_verify(50, PbmDimension::Emotional, &p, 1_000_000)
            .unwrap();
        assert!((t.energy(PbmDimension::Emotional) - 50.0).abs() < 1e-10);
    }

    #[test]
    fn leak_reduces_energy_over_time() {
        let mut t = NeuroEnergyTracker::new();
        let p = UserSafetyProfile::standard();
        t.intake_and_verify(50, PbmDimension::Emotional, &p, 1_000_000)
            .unwrap();
        t.intake_and_verify(50, PbmDimension::Emotional, &p, 2_000_000)
            .unwrap();
        let e = t.energy(PbmDimension::Emotional);
        assert!(
            e < 100.0 && e > 99.9,
            "leak should reduce energy, got {}",
            e
        );
    }

    #[test]
    fn energy_never_goes_negative() {
        let mut t = NeuroEnergyTracker::new();
        let p = UserSafetyProfile::standard();
        t.intake_and_verify(10, PbmDimension::Emotional, &p, 1_000_000)
            .unwrap();
        t.intake_and_verify(0, PbmDimension::Emotional, &p, 1_001_000_000)
            .unwrap();
        assert!(t.energy(PbmDimension::Emotional) >= 0.0);
    }

    #[test]
    fn high_continuous_intensity_breaches_threshold() {
        let mut t = NeuroEnergyTracker::with_params(0.100, 0.0, 0, 0.0, 0.0, 1.0);
        let mut p = UserSafetyProfile::standard();
        p.critical_thresholds = [10.0; 4];
        let mut result = Ok(());
        for i in 0..100 {
            let ns = (i + 1) as u64 * 1_000_000;
            result = t.intake_and_verify(1, PbmDimension::Emotional, &p, ns);
            if result.is_err() {
                break;
            }
        }
        assert!(result.is_err());
    }

    #[test]
    fn silent_frames_drain_energy_no_breach() {
        let mut t = NeuroEnergyTracker::new();
        let p = UserSafetyProfile::standard();
        for i in 0..3 {
            t.intake_and_verify(20, PbmDimension::Emotional, &p, (i + 1) as u64 * 1_000_000)
                .unwrap();
        }
        let last = 4_000_000u64;
        for i in 0..100000 {
            t.intake_and_verify(
                0,
                PbmDimension::Emotional,
                &p,
                last + (i + 1) as u64 * 1_000_000,
            )
            .unwrap();
        }
        assert!(t.energy(PbmDimension::Emotional) < 1.0);
    }

    #[test]
    fn four_dimensions_independent() {
        let mut t = NeuroEnergyTracker::new();
        let p = UserSafetyProfile::standard();
        for i in 0..100 {
            t.intake_and_verify(5, PbmDimension::Emotional, &p, (i + 1) as u64 * 1_000_000)
                .unwrap();
        }
        assert!(t.energy(PbmDimension::Emotional) > 0.0);
        assert!(t.energy(PbmDimension::Visceral) < 0.01);
    }

    #[test]
    fn clock_hang_preserves_energy() {
        let mut t = NeuroEnergyTracker::with_max_dt(0.100);
        let p = UserSafetyProfile::standard();
        t.intake_and_verify(100, PbmDimension::Emotional, &p, 1_000_000)
            .unwrap();
        let e1 = t.energy(PbmDimension::Emotional);
        t.intake_and_verify(0, PbmDimension::Emotional, &p, 1_001_000_000)
            .unwrap();
        assert!(t.energy(PbmDimension::Emotional) >= e1);
    }

    #[test]
    fn nonlinear_leak_faster_at_high_energy() {
        let low = effective_leak_rate(10.0, 100.0, 2.0, 1.0);
        let high = effective_leak_rate(90.0, 100.0, 2.0, 1.0);
        assert!(high > low);
    }

    #[test]
    fn nonlinear_leak_max_at_2x() {
        let leak = effective_leak_rate(100.0, 100.0, 2.0, 1.0);
        assert!((leak - 4.0).abs() < 1e-10);
    }

    #[test]
    fn nonlinear_leak_baseline_at_zero() {
        assert!((effective_leak_rate(0.0, 100.0, 2.0, 1.0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn pwm_attack_blocked_by_nonlinear_leak() {
        let mut t = NeuroEnergyTracker::with_params(0.100, 1.0, 0, 0.0, 0.0, 1.0);
        let p = UserSafetyProfile {
            kind: ProfileKind::Standard,
            leak_rates: [2.0; 4],
            critical_thresholds: [80.0; 4],
            anchor_confidence: None,
            defence_level: None,
        };
        let mut breached = false;
        for i in 0..1000 {
            let intensity = if i % 10 < 5 { 80 } else { 0 };
            if t.intake_and_verify(
                intensity,
                PbmDimension::Emotional,
                &p,
                (i + 1) as u64 * 1_000_000,
            )
            .is_err()
            {
                breached = true;
                break;
            }
        }
        assert!(breached);
    }

    #[test]
    fn refractory_period_blocks_accumulation() {
        let mut t = NeuroEnergyTracker::with_params(0.100, 1.0, 50, 0.0, 0.0, 1.0);
        let p = UserSafetyProfile {
            kind: ProfileKind::Standard,
            leak_rates: [0.0; 4],
            critical_thresholds: [100.0; 4],
            anchor_confidence: None,
            defence_level: None,
        };
        t.intake_and_verify(95, PbmDimension::Emotional, &p, 1_000_000)
            .unwrap();
        let e1 = t.energy(PbmDimension::Emotional);
        t.intake_and_verify(10, PbmDimension::Emotional, &p, 2_000_000)
            .unwrap();
        assert!(t.energy(PbmDimension::Emotional) <= e1 + 1.0);
    }

    #[test]
    fn refractory_exits_when_energy_drops() {
        let mut t = NeuroEnergyTracker::with_params(0.100, 0.0, 50, 0.0, 0.0, 1.0);
        let p = UserSafetyProfile {
            kind: ProfileKind::Standard,
            leak_rates: [50.0; 4],
            critical_thresholds: [100.0; 4],
            anchor_confidence: None,
            defence_level: None,
        };
        t.intake_and_verify(95, PbmDimension::Emotional, &p, 1_000_000)
            .unwrap();
        assert!(t.refractory_active[dim_index(PbmDimension::Emotional)]);
        for i in 0..2000 {
            t.intake_and_verify(
                0,
                PbmDimension::Emotional,
                &p,
                2_000_000 + (i + 1) as u64 * 1_000_000,
            )
            .unwrap();
        }
        assert!(!t.refractory_active[dim_index(PbmDimension::Emotional)]);
    }

    #[test]
    fn global_bucket_catches_alternating_attack() {
        let mut t = NeuroEnergyTracker::with_params(0.100, 0.0, 0, 0.3, 0.3, 0.9);
        let p = UserSafetyProfile {
            kind: ProfileKind::Standard,
            leak_rates: [0.0; 4],
            critical_thresholds: [100.0; 4],
            anchor_confidence: None,
            defence_level: None,
        };
        let dims = [
            PbmDimension::Visceral,
            PbmDimension::Emotional,
            PbmDimension::Tactile,
            PbmDimension::Auditory,
        ];
        for (i, d) in dims.iter().enumerate() {
            t.intake_and_verify(50, *d, &p, (i + 1) as u64 * 1_000_000)
                .unwrap();
        }
        for round in 0..10 {
            for (j, d) in dims.iter().enumerate() {
                let ns = ((round + 1) * 4 + j + 1) as u64 * 1_000_000;
                if t.intake_and_verify(50, *d, &p, ns).is_err() {
                    return;
                }
            }
            if t.verify_cross_dimension(&p).is_err() {
                return;
            }
        }
        assert!(t.verify_cross_dimension(&p).is_err());
    }

    #[test]
    fn reset_clears_refractory_and_energy() {
        let mut t = NeuroEnergyTracker::default();
        let p = UserSafetyProfile {
            kind: ProfileKind::Standard,
            leak_rates: [0.0; 4],
            critical_thresholds: [100.0; 4],
            anchor_confidence: None,
            defence_level: None,
        };
        t.intake_and_verify(95, PbmDimension::Emotional, &p, 1_000_000)
            .unwrap();
        t.reset();
        assert!(!t.refractory_active[dim_index(PbmDimension::Emotional)]);
        assert!(t.energy(PbmDimension::Emotional) < 0.01);
    }
}
