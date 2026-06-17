// src/device_map.rs — Pass 7: PSIR × DeviceSet → DSIR
//
// 必须在 Personalize 之后——我们知道此人需要多少信号——再分配设备。
// v0.3 骨架：默认设备集 = { ear }。耳后缺失 = 硬线拒绝编译。
// 完整设备集 + 多设备帧率差异 → v0.4+。

use crate::dsir::{DeviceAssignment, DeviceId, DeviceSet, DsirDoc};
use crate::error::AnimiError;
use crate::psir::PsirDoc;

/// 将 PSIR 映射到设备集合，生成 DSIR。
///
/// v0.3 骨架——仅 ear 设备。ear 缺失 = 立即拒绝。
pub fn device_map(psir: &PsirDoc, device_set: &DeviceSet) -> Result<DsirDoc, AnimiError> {
    // ── 硬线：ear 缺失 → 拒绝编译 ──
    if !device_set.has_ear() {
        return Err(AnimiError::InternalError {
            file_name: crate::error::current_file(),
            msg: "耳后设备未连接——迷走神经通路不可用。Session 不可启动。".into(),
            severity: crate::error::Severity::Deny,
        });
    }

    let assignments = vec![DeviceAssignment {
        device: DeviceId::Ear,
        intensity_share: 1.0,
        frame_interval_ms: 1,
        degraded: false,
        degrade_reason: None,
    }];

    // ── outline 降级检查（v0.4+） ──
    // v0.3 骨架——wrist/neck/temple 缺失是正常状态，不标记降级。
    // v0.4+ 完整设备集时才启用降级检测。

    // ── oi 平滑序列 pass-through ──
    let smoothing_multipliers: Vec<f64> = psir
        .smoothing
        .as_ref()
        .map(|s| s.steps.iter().map(|step| step.multiplier).collect())
        .unwrap_or_default();

    Ok(DsirDoc {
        name: psir.name.clone(),
        shape_name: psir.shape.name.clone(),
        main_dimension: format!("{:?}", "Emotional"), // v0.3 默认情绪维度
        applied_min: psir.intensity.applied_min,
        applied_max: psir.intensity.applied_max,
        smoothing_multipliers,
        assignments,
        any_degraded: false, // v0.3 骨架——不检测设备缺失
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::psir::{PersonalizedFeeling, PsirDoc, PsirIntensity, PsirPbmStamp, PsirShape};

    fn make_psir() -> PsirDoc {
        PsirDoc {
            name: "calm".into(),
            main: PersonalizedFeeling {
                atom: "calm_meditative".into(),
                baseline_offset: 0.40,
                damped: false,
            },
            accents: vec![],
            shape: PsirShape {
                name: "gradual_rise_fall".into(),
            },
            intensity: PsirIntensity {
                original_min: 15,
                original_max: 45,
                applied_min: 10,
                applied_max: 38,
                cap: 100,
            },
            smoothing: None,
            cold_start: true,
            defence_activated: false,
            defence_level: None,
            degraded: false,
            pbm_stamp: PsirPbmStamp {
                pbm_updated_at: "2026-06-15T00:00:00Z".into(),
                session_count: 1,
                cold_start: true,
            },
        }
    }

    #[test]
    fn device_map_ear_only_passes() {
        let psir = make_psir();
        let ds = DeviceSet::default();
        let dsir = device_map(&psir, &ds).unwrap();
        assert_eq!(dsir.name, "calm");
        assert_eq!(dsir.shape_name, "gradual_rise_fall");
        assert_eq!(dsir.assignments.len(), 1);
        assert_eq!(dsir.assignments[0].device, DeviceId::Ear);
        assert!(!dsir.any_degraded);
    }

    #[test]
    fn missing_ear_rejected() {
        let psir = make_psir();
        let ds = DeviceSet { devices: vec![] };
        let result = device_map(&psir, &ds);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("耳后"));
    }

    #[test]
    fn missing_wrist_no_degrade_in_v0_3() {
        let psir = make_psir();
        let ds = DeviceSet {
            devices: vec![DeviceId::Ear],
        }; // 缺 wrist
        let dsir = device_map(&psir, &ds).unwrap();
        // v0.3 骨架——缺设备不标记降级
        assert!(!dsir.any_degraded);
    }
}
