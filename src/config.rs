// src/config.rs — Anim 交织器运行时参数
//
// 所有可调参数从 configs/default.yaml 加载。
// 未指定的 key 回退到 Rust Default（即当前设计估值）。
//
// 不是热更新——Anim 是 CLI 工具，一次调用 = 一个编译 Session。
// --config 指向不同 YAML 即可切换。生产环境参数由 Core PBM 快照注入。

use serde::Deserialize;

// ═══════════════════════════════════════════════════════════════
// 顶层配置
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AnimConfig {
    pub profiles: ProfilesConfig,
    pub leaky_bucket: LeakyBucketConfig,
    pub monotony: MonotonyConfig,
    pub sigmoidal: SigmoidalConfig,
    pub damping: DampingConfig,
    pub cold_start: ColdStartConfig,
    pub pbm_coefficients: PbmCoefficientsConfig,
    pub oi_smoothing: OiSmoothingConfig,
    pub caps: CapsConfig,
    pub hooks: HooksConfig,
}

impl AnimConfig {
    /// 从 YAML 文件加载，缺失字段用 Default 填充。
    pub fn load(path: &str) -> Result<Self, String> {
        let yaml_str =
            std::fs::read_to_string(path).map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_yaml::from_str(&yaml_str).map_err(|e| format!("YAML 解析失败: {}", e))
    }
}

// ═══════════════════════════════════════════════════════════════
// 各配置段
// ═══════════════════════════════════════════════════════════════

/// 单个 Profile 的参数。
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileParams {
    pub leak_rates: [f64; 4],
    pub critical_thresholds: [f64; 4],
}

/// 六种用户安全 Profile。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ProfilesConfig {
    pub standard: ProfileParams,
    pub low_anchor: ProfileParams,
    pub defence_d1: ProfileParams,
    pub defence_d2: ProfileParams,
    pub defence_d3: ProfileParams,
}

impl Default for ProfilesConfig {
    fn default() -> Self {
        ProfilesConfig {
            standard: ProfileParams {
                leak_rates: [2.0, 2.0, 2.0, 2.0],
                critical_thresholds: [600.0, 600.0, 600.0, 600.0],
            },
            low_anchor: ProfileParams {
                leak_rates: [1.0, 1.0, 1.0, 1.0],
                critical_thresholds: [120.0, 120.0, 120.0, 120.0],
            },
            defence_d1: ProfileParams {
                leak_rates: [1.8, 1.8, 1.8, 1.8],
                critical_thresholds: [400.0, 400.0, 400.0, 400.0],
            },
            defence_d2: ProfileParams {
                leak_rates: [1.0, 1.0, 1.0, 1.0],
                critical_thresholds: [200.0, 200.0, 200.0, 200.0],
            },
            defence_d3: ProfileParams {
                leak_rates: [0.05, 0.05, 0.05, 0.05],
                critical_thresholds: [30.0, 30.0, 30.0, 30.0],
            },
        }
    }
}

/// NeuroEnergyTracker 可调参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LeakyBucketConfig {
    pub max_dt_seconds: f64,
    pub nonlinear_gamma: f64,
    pub refractory_frames: u32,
    pub sigma: f64,
    pub global_alpha: f64,
    pub global_beta: f64,
}

impl Default for LeakyBucketConfig {
    fn default() -> Self {
        LeakyBucketConfig {
            max_dt_seconds: 0.1,
            nonlinear_gamma: 1.0,
            refractory_frames: 500,
            sigma: 0.3,
            global_alpha: 0.3,
            global_beta: 0.9,
        }
    }
}

/// 脱敏检测参数——ADR 012。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MonotonyConfig {
    pub delta_threshold: u32,
    pub detection_frames: u32,
    pub steady_frames_multiplier: u32,
}

impl Default for MonotonyConfig {
    fn default() -> Self {
        MonotonyConfig {
            delta_threshold: 2,
            detection_frames: 500,
            steady_frames_multiplier: 3,
        }
    }
}

/// sigmoidal 强度缩放参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SigmoidalConfig {
    pub k: f64,
    pub x0: f64,
    pub compression_alpha: f64,
}

impl Default for SigmoidalConfig {
    fn default() -> Self {
        SigmoidalConfig {
            k: 6.0,
            x0: 0.5,
            compression_alpha: 0.5,
        }
    }
}

/// 跨维度阻尼矩阵参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DampingConfig {
    pub gradient_thresholds: GradientThresholds,
    /// 索引：[Visceral, Emotional, Tactile, Auditory]
    pub initial_multipliers: [f64; 4],
    pub ema_alpha: f64,
    pub cold_start_transition_window: u32,
}

impl Default for DampingConfig {
    fn default() -> Self {
        DampingConfig {
            gradient_thresholds: GradientThresholds::default(),
            initial_multipliers: [0.35, 0.50, 0.45, 0.50],
            ema_alpha: 0.9,
            cold_start_transition_window: 5,
        }
    }
}

/// 跨维度冻结梯度阈值——ADR 007 §7.7。
/// 梯度 > threshold × 步长 → 触发跨维度冻结。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GradientThresholds {
    pub visceral: f64,
    pub emotional: f64,
    pub tactile: f64,
    pub auditory: f64,
}

impl Default for GradientThresholds {
    fn default() -> Self {
        GradientThresholds {
            visceral: 1.5,
            emotional: 2.0,
            tactile: 3.0,
            auditory: 2.0,
        }
    }
}

/// 冷启动参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ColdStartConfig {
    pub sessions_threshold: u32,
}

impl Default for ColdStartConfig {
    fn default() -> Self {
        ColdStartConfig {
            sessions_threshold: 10,
        }
    }
}

/// PBM 四维冷启动基线偏移系数——Feelings-ROADMAP §1.2。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PbmCoefficientsConfig {
    pub visceral: f64,
    pub emotional: f64,
    pub tactile: f64,
    pub auditory: f64,
}

impl Default for PbmCoefficientsConfig {
    fn default() -> Self {
        PbmCoefficientsConfig {
            visceral: 0.75,
            emotional: 0.40,
            tactile: 0.80,
            auditory: 0.85,
        }
    }
}

/// oi 帧平滑过渡参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OiSmoothingConfig {
    pub hard_cut_boundary: u32,
    pub decay_sequence: [f64; 5],
}

impl Default for OiSmoothingConfig {
    fn default() -> Self {
        OiSmoothingConfig {
            hard_cut_boundary: 20,
            decay_sequence: [1.0, 0.6, 0.3, 0.1, 0.0],
        }
    }
}

/// 全局强度上限。
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CapsConfig {
    pub global_intensity: u32,
    pub abrupt_stop_max: u32,
    pub sandbox_accent_max: f64,
    pub accent_ratio_fallback: f64,
    pub low_anchor_boundary: f64,
    pub low_anchor_cap: u32,
}

impl Default for CapsConfig {
    fn default() -> Self {
        CapsConfig {
            global_intensity: 100,
            abrupt_stop_max: 20,
            sandbox_accent_max: 0.3,
            accent_ratio_fallback: 0.30,
            low_anchor_boundary: 0.3,
            low_anchor_cap: 20,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Hook 开关 —— ADR 013 Pipeline + Hook 架构
// ═══════════════════════════════════════════════════════════════

/// 单个 hook 的开关配置。
#[derive(Debug, Clone, Deserialize)]
pub struct HookSwitch {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

impl Default for HookSwitch {
    fn default() -> Self { HookSwitch { enabled: true } }
}

/// 所有 hook 的启用/禁开关。
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct HooksConfig {
    #[serde(default)]
    pub static_safety: HookSwitch,
    #[serde(default)]
    pub low_anchor_cap: HookSwitch,
    #[serde(default)]
    pub leaky_bucket_intake: HookSwitch,
    #[serde(default)]
    pub monotony: HookSwitch,
    #[serde(default)]
    pub cross_dim_coupling: HookSwitch,
    #[serde(default)]
    pub cold_start: HookSwitch,
    #[serde(default)]
    pub oi_smoothing: HookSwitch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_design_estimates() {
        let cfg = AnimConfig::default();
        // Profiles
        assert_eq!(cfg.profiles.standard.leak_rates, [2.0, 2.0, 2.0, 2.0]);
        assert_eq!(
            cfg.profiles.standard.critical_thresholds,
            [600.0, 600.0, 600.0, 600.0]
        );
        assert_eq!(cfg.profiles.defence_d3.leak_rates, [0.05, 0.05, 0.05, 0.05]);
        assert_eq!(
            cfg.profiles.defence_d3.critical_thresholds,
            [30.0, 30.0, 30.0, 30.0]
        );
        // Leaky bucket
        assert_eq!(cfg.leaky_bucket.max_dt_seconds, 0.1);
        assert_eq!(cfg.leaky_bucket.nonlinear_gamma, 1.0);
        assert_eq!(cfg.leaky_bucket.refractory_frames, 500);
        assert_eq!(cfg.leaky_bucket.sigma, 0.3);
        assert!((cfg.leaky_bucket.global_alpha - 0.3).abs() < 1e-10);
        assert!((cfg.leaky_bucket.global_beta - 0.9).abs() < 1e-10);
        // Monotony
        assert_eq!(cfg.monotony.delta_threshold, 2);
        assert_eq!(cfg.monotony.detection_frames, 500);
        // Sigmoidal
        assert!((cfg.sigmoidal.k - 6.0).abs() < 1e-10);
        assert!((cfg.sigmoidal.x0 - 0.5).abs() < 1e-10);
        // Damping
        assert_eq!(cfg.damping.initial_multipliers, [0.35, 0.50, 0.45, 0.50]);
        assert!((cfg.damping.ema_alpha - 0.9).abs() < 1e-10);
        // Cold start
        assert_eq!(cfg.cold_start.sessions_threshold, 10);
        // PBM coefficients
        assert!((cfg.pbm_coefficients.visceral - 0.75).abs() < 1e-10);
        assert!((cfg.pbm_coefficients.emotional - 0.40).abs() < 1e-10);
        // Oi smoothing
        assert_eq!(cfg.oi_smoothing.hard_cut_boundary, 20);
        assert_eq!(cfg.oi_smoothing.decay_sequence, [1.0, 0.6, 0.3, 0.1, 0.0]);
        // Caps
        assert_eq!(cfg.caps.global_intensity, 100);
        assert_eq!(cfg.caps.abrupt_stop_max, 20);
        assert!((cfg.caps.sandbox_accent_max - 0.3).abs() < 1e-10);
    }

    #[test]
    fn partial_yaml_overrides_defaults() {
        let yaml = r#"
caps:
  global_intensity: 50
leaky_bucket:
  max_dt_seconds: 0.05
"#;
        let cfg: AnimConfig = serde_yaml::from_str(yaml).unwrap();
        // 覆盖的部分
        assert_eq!(cfg.caps.global_intensity, 50);
        assert!((cfg.leaky_bucket.max_dt_seconds - 0.05).abs() < 1e-10);
        // 未覆盖的部分——回退到 Default
        assert_eq!(cfg.caps.abrupt_stop_max, 20);
        assert_eq!(cfg.leaky_bucket.nonlinear_gamma, 1.0);
        assert_eq!(cfg.cold_start.sessions_threshold, 10);
    }
}
