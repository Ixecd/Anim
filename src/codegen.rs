// src/codegen.rs — Pass 8: DSIR → ESIR 帧序列生成
//
// 形状 → 时间轴展开 → ESIR 帧序列。
// v0.3 骨架：6 种预定义形状 + Postcard 二进制输出。
// 闭环偏差修正 + 自适应帧密度 → v0.5。
// FPGA 对接 → v0.5。

use crate::dsir::DsirDoc;
use crate::error::AnimiError;
use crate::esir::{EsirDoc, EsirFrame};

/// 形状时间轴配置。
struct ShapeProfile {
    /// 准备段（帧数）——从零强度到目标强度的缓升。
    rise_frames: u32,
    /// 持续段（帧数）——恒定目标强度。
    sustain_frames: u32,
    /// 消退段（帧数）——从目标强度降到零。
    fall_frames: u32,
    /// 静默尾部（帧数）——保底空帧。
    tail_frames: u32,
}

impl ShapeProfile {
    fn from_shape_name(name: &str, intensity_min: u32, intensity_max: u32) -> Self {
        let peak = intensity_max.max(intensity_min);
        match name {
            "sharp_peak" => ShapeProfile {
                rise_frames: 5,
                sustain_frames: 3,
                fall_frames: 5,
                tail_frames: 5,
            },
            "steady" => ShapeProfile {
                rise_frames: 10,
                sustain_frames: peak.min(100),
                fall_frames: 10,
                tail_frames: 5,
            },
            "slow_decay" => ShapeProfile {
                rise_frames: 10,
                sustain_frames: 10,
                fall_frames: peak.max(20),
                tail_frames: 5,
            },
            "wave" => ShapeProfile {
                rise_frames: 8,
                sustain_frames: 10,
                fall_frames: 8,
                tail_frames: 8,
            },
            "abrupt_stop" => ShapeProfile {
                rise_frames: 5,
                sustain_frames: 3,
                fall_frames: 1, // 硬截断——一帧归零
                tail_frames: 5,
            },
            // gradual_rise_fall（默认）
            _ => ShapeProfile {
                rise_frames: 15,
                sustain_frames: 50,
                fall_frames: 15,
                tail_frames: 10,
            },
        }
    }

    fn total_frames(&self) -> u32 {
        self.rise_frames + self.sustain_frames + self.fall_frames + self.tail_frames
    }
}

/// DSIR → ESIR 帧序列。
///
/// v0.3 骨架：固定 1ms/帧，首帧 + 尾帧嵌入安全校验标记。
pub fn codegen(dsir: &DsirDoc) -> Result<EsirDoc, AnimiError> {
    let profile =
        ShapeProfile::from_shape_name(&dsir.shape_name, dsir.applied_min, dsir.applied_max);
    let peak = dsir.applied_max.max(dsir.applied_min);
    let _min = dsir.applied_min.min(dsir.applied_max);

    let total_frames = profile.total_frames();
    let mut frames = Vec::with_capacity(total_frames as usize);
    let mut ts: u64 = 0;

    // ── rise 段 ──
    for i in 0..profile.rise_frames {
        let t = if profile.rise_frames > 1 {
            i as f64 / (profile.rise_frames - 1) as f64
        } else {
            0.0
        };
        let intensity = (t * peak as f64).round() as u32;
        let mut frame = make_frame(frames.len() as u32 + 1, ts, intensity, dsir);
        if i == 0 {
            frame.flags |= 0x01; // 首帧 = 安全校验帧
        }
        frames.push(frame);
        ts += 1000; // 1ms
    }

    // ── sustain 段 ──
    for _ in 0..profile.sustain_frames {
        let frame = make_frame(frames.len() as u32 + 1, ts, peak, dsir);
        frames.push(frame);
        ts += 1000;
    }

    // ── fall 段 ──
    for i in 0..profile.fall_frames {
        let t = if profile.fall_frames > 1 {
            i as f64 / (profile.fall_frames - 1) as f64
        } else {
            0.0
        };
        let intensity = (peak as f64 * (1.0 - t)).round() as u32;
        let frame = make_frame(frames.len() as u32 + 1, ts, intensity, dsir);
        frames.push(frame);
        ts += 1000;
    }

    // ── tail 段——静默保底 ──
    for i in 0..profile.tail_frames {
        let mut frame = make_frame(frames.len() as u32 + 1, ts, 0, dsir);
        frame.flags |= 0x02; // 恢复帧
        if i == profile.tail_frames - 1 {
            frame.flags |= 0x01; // 尾帧 = 安全校验帧
        }
        frames.push(frame);
        ts += 1000;
    }

    let frame_count = frames.len() as u32;
    Ok(EsirDoc {
        name: dsir.name.clone(),
        shape_name: dsir.shape_name.clone(),
        frame_count,
        duration_ms: frame_count, // 1ms/帧
        frames,
    })
}

fn make_frame(id: u32, ts: u64, intensity: u32, _dsir: &DsirDoc) -> EsirFrame {
    EsirFrame {
        frame_id: id,
        timestamp_us: ts,
        intensity: intensity.min(100),
        frequency_hz: 25,
        pulse_width_us: 200,
        flags: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsir::{DeviceAssignment, DeviceId, DsirDoc};

    fn make_dsir(shape: &str, min: u32, max: u32) -> DsirDoc {
        DsirDoc {
            name: "calm".into(),
            shape_name: shape.into(),
            main_dimension: "Emotional".into(),
            applied_min: min,
            applied_max: max,
            smoothing_multipliers: vec![],
            assignments: vec![DeviceAssignment {
                device: DeviceId::Ear,
                intensity_share: 1.0,
                frame_interval_ms: 1,
                degraded: false,
                degrade_reason: None,
            }],
            any_degraded: false,
        }
    }

    #[test]
    fn codegen_generates_frames() {
        let dsir = make_dsir("gradual_rise_fall", 15, 45);
        let esir = codegen(&dsir).unwrap();
        assert!(esir.frame_count > 0);
        assert_eq!(esir.name, "calm");
        // 帧序列应从低到高再到零
        assert!(esir.frames.first().unwrap().intensity <= 10);
        // 中间帧接近峰值
        let mid = esir.frames[esir.frames.len() / 2];
        assert!(mid.intensity > 0);
        // 尾帧归零
        assert_eq!(esir.frames.last().unwrap().intensity, 0);
    }

    #[test]
    fn codegen_sharp_peak_short() {
        let dsir = make_dsir("sharp_peak", 30, 80);
        let esir = codegen(&dsir).unwrap();
        assert!(esir.frame_count > 0);
        // sharp_peak 应该有相对较短的帧序列
        assert!(esir.frame_count <= 30);
    }

    #[test]
    fn codegen_steady_long_sustain() {
        let dsir = make_dsir("steady", 5, 20);
        let esir = codegen(&dsir).unwrap();
        // steady: sustain = min(20,100) = 20 帧
        assert!(esir.frame_count >= 30);
        assert_eq!(esir.frames.last().unwrap().intensity, 0);
    }

    #[test]
    fn codegen_abrupt_stop_fast_fall() {
        let dsir = make_dsir("abrupt_stop", 10, 20);
        let esir = codegen(&dsir).unwrap();
        // abrupt_stop: fall=1帧
        assert!(esir.frame_count > 0);
        // 首帧 = 安全校验帧
        assert_eq!(esir.frames[0].flags & 0x01, 0x01);
    }

    #[test]
    fn esir_binary_roundtrip() {
        let dsir = make_dsir("gradual_rise_fall", 15, 45);
        let esir = codegen(&dsir).unwrap();
        let bytes = esir.to_binary().unwrap();
        let esir2 = EsirDoc::from_binary(&bytes).unwrap();
        assert_eq!(esir.frame_count, esir2.frame_count);
        assert_eq!(esir.frames.len(), esir2.frames.len());
    }
}
