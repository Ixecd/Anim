// src/dsir.rs — DSIR 类型定义（Pass 7 DeviceMap 输出）
//
// DSIR = 设备感知的感受结构。
// 同一份 PSIR → 不同设备组合 → 不同的 DSIR。
// 设备缺失 → 降级标记。设备变化 → 不重算 PSIR。

use serde::{Deserialize, Serialize};

/// 设备标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceId {
    /// 耳后设备——迷走神经耳支刺激 + 心率采集 + 骨传导。
    Ear,
    /// 腕部设备——皮肤电导采集 + 温度控制。
    Wrist,
    /// 后颈设备——本体感受采集 + 低频振动。
    Neck,
    /// 颞部设备——EEG 采集 + 认知状态信号。
    Temple,
}

/// 设备集合——当前 Session 连接的所有设备。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSet {
    pub devices: Vec<DeviceId>,
}

impl Default for DeviceSet {
    /// 默认设备集——耳后（唯一硬线设备）。v0.3 骨架。
    fn default() -> Self {
        DeviceSet {
            devices: vec![DeviceId::Ear],
        }
    }
}

impl DeviceSet {
    /// Ear is mandatory — no Ear, no Session.
    pub fn has_ear(&self) -> bool {
        self.devices.contains(&DeviceId::Ear)
    }

    pub fn has(&self, id: DeviceId) -> bool {
        self.devices.contains(&id)
    }
}

/// 单个设备的信号分配结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAssignment {
    /// 目标设备。
    pub device: DeviceId,
    /// 分配给该设备的强度比例 [0.0, 1.0]。
    pub intensity_share: f64,
    /// 帧间隔（毫秒）。ear=1ms, wrist=8ms, neck=2ms, temple=4ms。
    pub frame_interval_ms: u32,
    /// 是否因设备缺失而降级。
    pub degraded: bool,
    /// 降级原因（若 degraded=true）。
    pub degrade_reason: Option<String>,
}

/// DSIR 文档——设备感知的感受结构。
///
/// Pass 7 (DeviceMap) 的输出。PSIR × DeviceSet → DsirDoc。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsirDoc {
    /// 来源 PSIR 名称。
    pub name: String,
    /// 形状名称——由 PSIR 的形状名决定。
    pub shape_name: String,
    /// 主旋律维度——决定了主要信号分配给哪个设备。
    pub main_dimension: String,
    /// 个人校准后的强度区间。
    pub applied_min: u32,
    pub applied_max: u32,
    /// oi 平滑衰减序列（pass-through）。
    pub smoothing_multipliers: Vec<f64>,
    /// 设备分配表。
    pub assignments: Vec<DeviceAssignment>,
    /// 是否有任何设备因缺失而降级。
    pub any_degraded: bool,
}

impl DsirDoc {
    /// 序列化为 JSON。
    pub fn to_json(&self) -> Result<String, crate::error::AnimiError> {
        serde_json::to_string_pretty(self).map_err(|e| crate::error::AnimiError::InternalError {
            file_name: crate::error::current_file(),
            msg: format!("DSIR JSON 序列化失败: {}", e),
            severity: crate::error::Severity::Deny,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_device_set_has_ear() {
        let ds = DeviceSet::default();
        assert!(ds.has_ear());
        assert!(ds.has(DeviceId::Ear));
        assert!(!ds.has(DeviceId::Wrist));
    }

    #[test]
    fn dsir_json_roundtrip() {
        let dsir = DsirDoc {
            name: "calm".into(),
            shape_name: "gradual_rise_fall".into(),
            main_dimension: "Emotional".into(),
            applied_min: 15,
            applied_max: 45,
            smoothing_multipliers: vec![1.0, 0.6, 0.3, 0.1, 0.0],
            assignments: vec![DeviceAssignment {
                device: DeviceId::Ear,
                intensity_share: 1.0,
                frame_interval_ms: 1,
                degraded: false,
                degrade_reason: None,
            }],
            any_degraded: false,
        };
        let json = dsir.to_json().unwrap();
        assert!(json.contains("calm"));
        assert!(json.contains("Ear"));
        assert!(json.contains("1.0"));
    }
}
