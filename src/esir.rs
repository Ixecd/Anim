// src/esir.rs — ESIR 类型定义（Pass 8 CodeGen 输出）
//
// ESIR = 执行信号层——每一帧的具体物理参数。
// 当前阶段（v0.3）输出 Postcard 二进制帧序列到 .esir 文件。
// FPGA 对接 → v0.5。

use serde::{Deserialize, Serialize};

/// 单帧 ESIR 指令——发给一个设备的物理参数。
///
/// 16 字节固定宽度的 Postcard 二进制格式。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EsirFrame {
    /// 帧序号——从 1 开始。
    pub frame_id: u32,
    /// 微秒时间戳——Session 启动时归零。
    pub timestamp_us: u64,
    /// 当前帧强度（0-100）。
    pub intensity: u32,
    /// 刺激频率（0-500Hz）。
    pub frequency_hz: u16,
    /// 脉宽（0-500μs）。
    pub pulse_width_us: u16,
    /// 位标志：
    ///   bit0 = 安全校验帧
    ///   bit1 = 恢复帧
    ///   bit2 = 硬截断帧（强度直接从当前值归零）
    pub flags: u8,
}

impl Default for EsirFrame {
    fn default() -> Self {
        EsirFrame {
            frame_id: 0,
            timestamp_us: 0,
            intensity: 0,
            frequency_hz: 25,     // 默认迷走神经刺激频率
            pulse_width_us: 200,  // 默认脉宽
            flags: 0,
        }
    }
}

/// ESIR 帧序列文档——完整的 Session 帧列表。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsirDoc {
    /// 感受包名称。
    pub name: String,
    /// 来源形状名称。
    pub shape_name: String,
    /// 帧总数。
    pub frame_count: u32,
    /// 总时长（毫秒）。
    pub duration_ms: u32,
    /// 帧序列。
    pub frames: Vec<EsirFrame>,
}

impl EsirDoc {
    /// 序列化为 Postcard 二进制。
    pub fn to_binary(&self) -> Result<Vec<u8>, crate::error::AnimiError> {
        postcard::to_allocvec(self).map_err(|e| {
            crate::error::AnimiError::InternalError {
                file_name: crate::error::current_file(),
                msg: format!("ESIR 二进制序列化失败: {}", e),
                severity: crate::error::Severity::Deny,
            }
        })
    }

    /// 从 Postcard 二进制反序列化。
    pub fn from_binary(bytes: &[u8]) -> Result<Self, crate::error::AnimiError> {
        postcard::from_bytes(bytes).map_err(|e| {
            crate::error::AnimiError::InternalError {
                file_name: crate::error::current_file(),
                msg: format!("ESIR 二进制反序列化失败: {}", e),
                severity: crate::error::Severity::Deny,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esir_frame_default_fields() {
        let f = EsirFrame::default();
        assert_eq!(f.intensity, 0);
        assert_eq!(f.frequency_hz, 25);
        assert_eq!(f.pulse_width_us, 200);
        assert_eq!(f.flags, 0);
    }

    #[test]
    fn esir_doc_binary_roundtrip() {
        let doc = EsirDoc {
            name: "calm".into(),
            shape_name: "steady".into(),
            frame_count: 3,
            duration_ms: 3,
            frames: vec![
                EsirFrame { frame_id: 1, timestamp_us: 1000, intensity: 10, ..Default::default() },
                EsirFrame { frame_id: 2, timestamp_us: 2000, intensity: 15, ..Default::default() },
                EsirFrame { frame_id: 3, timestamp_us: 3000, intensity: 0, ..Default::default() },
            ],
        };
        let bytes = doc.to_binary().unwrap();
        let doc2 = EsirDoc::from_binary(&bytes).unwrap();
        assert_eq!(doc2.name, "calm");
        assert_eq!(doc2.frame_count, 3);
        assert_eq!(doc2.frames.len(), 3);
        assert_eq!(doc2.frames[0].intensity, 10);
        assert_eq!(doc2.frames[1].intensity, 15);
        assert_eq!(doc2.frames[2].intensity, 0);
    }
}
