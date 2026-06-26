// src/config.rs — Anim 交织器运行时参数
//
// Anim 职责收敛。LeakyBucket/Damping/ColdStart 等配置已迁移至 Feelings-Core。

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AnimConfig {
    pub caps: CapsConfig,
    pub oi_smoothing: OiSmoothingConfig,
}

impl AnimConfig {
    pub fn load(path: &str) -> Result<Self, String> {
        let yaml_str =
            std::fs::read_to_string(path).map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_yaml::from_str(&yaml_str).map_err(|e| format!("YAML 解析失败: {}", e))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::derivable_impls)]
pub struct CapsConfig {
    #[serde(default = "default_global_intensity")]
    pub global_intensity: u32,
    #[serde(default = "default_abrupt_stop_max")]
    pub abrupt_stop_max: u32,
    #[serde(default = "default_sandbox_accent_max")]
    pub sandbox_accent_max: f64,
    #[serde(default = "default_low_anchor_boundary")]
    pub low_anchor_boundary: f64,
    #[serde(default = "default_low_anchor_cap")]
    pub low_anchor_cap: u32,
}

fn default_global_intensity() -> u32 {
    100
}
fn default_abrupt_stop_max() -> u32 {
    20
}
fn default_sandbox_accent_max() -> f64 {
    0.1
}
fn default_low_anchor_boundary() -> f64 {
    0.3
}
fn default_low_anchor_cap() -> u32 {
    20
}

impl Default for CapsConfig {
    fn default() -> Self {
        CapsConfig {
            global_intensity: 100,
            abrupt_stop_max: 20,
            sandbox_accent_max: 0.1,
            low_anchor_boundary: 0.3,
            low_anchor_cap: 20,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::derivable_impls)]
pub struct OiSmoothingConfig {
    #[serde(default = "default_hard_cut_boundary")]
    pub hard_cut_boundary: u32,
    #[serde(default = "default_decay_sequence")]
    pub decay_sequence: Vec<f64>,
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
}

fn default_hard_cut_boundary() -> u32 {
    20
}
fn default_decay_sequence() -> Vec<f64> {
    vec![1.0, 0.6, 0.3, 0.1, 0.0]
}
fn default_max_steps() -> usize {
    5
}

impl Default for OiSmoothingConfig {
    fn default() -> Self {
        OiSmoothingConfig {
            hard_cut_boundary: 20,
            decay_sequence: vec![1.0, 0.6, 0.3, 0.1, 0.0],
            max_steps: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_design_estimates() {
        let c = AnimConfig::default();
        assert_eq!(c.caps.global_intensity, 100);
        assert_eq!(c.caps.low_anchor_cap, 20);
        assert_eq!(c.caps.abrupt_stop_max, 20);
    }

    #[test]
    fn partial_yaml_overrides_defaults() {
        let yaml = "caps:\n  global_intensity: 80\n";
        let cfg: AnimConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.caps.global_intensity, 80);
        assert_eq!(cfg.oi_smoothing.hard_cut_boundary, 20);
        assert_eq!(cfg.caps.low_anchor_cap, 20);
    }
}
