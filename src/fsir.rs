// src/fsir.rs — Pass 5：FSIR 生成
//
// AST（FeelingSource）→ FSIR 文档（FsirDoc）。
// FSIR 是感受结构的"通用本体"——不绑定任何个人生理参数或设备。
// 同一份 .anim → 同一份 FSIR → 不同的人上 PSIR 不同。

use crate::ast::*;
use crate::error::AnimiError;
use serde::{Deserialize, Serialize};

/// FSIR 文档——交织产物的顶层结构。
///
/// 这是 `.anim` 源码经过 Pass 0-4 之后的第一份 IR。
/// 所有后续 Pass（Personalize/DeviceMap/CodeGen）的输入。
///
/// 双序列化格式：
///   - JSON（to_json / from_json）——人类可读，Git diff，调试
///   - Postcard 二进制（to_binary / from_binary）——Go Server→Rust 设备预编译缓存
///     零拷贝反序列化，`#[no_std]` 兼容，Feelings-OS 裸机可直接加载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsirDoc {
    /// 编译元数据。
    pub meta: FsirMeta,

    /// 感受包名称——来自 `feeling <name>`。
    pub name: String,

    /// 混音结构。
    pub mix: FsirMix,

    /// 形状曲线。
    pub shape: FsirShape,

    /// 强度区间。
    pub intensity: FsirIntensity,

    /// oi 帧平滑过渡策略（Pass 4 guard::inject 产出）。
    ///
    /// 当运行期安全插桩检测到一帧应被拒绝时——
    /// 硬件衰减状态机按此序列在 4-8ms 内平滑归零。
    /// 后续 Pass（Personalize/CodeGen）从此字段读取衰减系数。
    pub smoothing: Option<crate::guard::OiSmoothing>,
}

/// FSIR 交织元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsirMeta {
    /// 交织器版本。
    pub animi_version: String,

    /// 交织时间（RFC 3339）。
    pub compiled_at: String,

    /// 源码的 SHA-256 哈希。v1.1 未接入 SPL——为 None。
    pub source_hash: Option<String>,

    /// Pattern Registry 的 SHA-256 哈希。v1.1 未接入——为 None。
    /// v1.2+ 缓存复用前先比对——不匹配则丢弃缓存重新交织。
    pub pattern_registry_hash: Option<String>,

    /// 安全规则版本号。v1.1 未接入——为 None。
    /// v1.2+ 全局安全规则更新后，旧缓存自动失效。
    pub safety_rules_version: Option<u32>,
}

/// FSIR 混音结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsirMix {
    /// 主旋律感受原子。
    pub main: String,

    /// 点缀列表。
    pub accents: Vec<FsirAccent>,
}

/// FSIR 点缀条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsirAccent {
    /// 感受原子名。
    pub atom: String,

    /// 配比系数 [0.0, 1.0]。
    pub ratio: f64,
}

/// FSIR 形状曲线。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsirShape {
    /// 形状名称（如 gradual_rise_fall）。
    pub name: String,
}

/// FSIR 强度区间。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsirIntensity {
    /// 最低强度。
    pub min: u32,

    /// 最高强度。
    pub max: u32,
}

impl FsirDoc {
    /// 从 AST 构建 FSIR 文档。
    ///
    /// 调用方必须先跑完 Pass 0-4 的安全校验。
    /// FSIRGen 本身不做校验——只做转换。
    pub fn from_ast(
        source: &FeelingSource,
        source_hash: Option<String>,
        registry_hash: Option<String>,
        intensity: &FsirIntensity,
        smoothing: Option<crate::guard::OiSmoothing>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        FsirDoc {
            meta: FsirMeta {
                animi_version: "0.1.0".into(),
                compiled_at: now,
                source_hash,
                pattern_registry_hash: registry_hash,
                safety_rules_version: None,
            },
            name: source.name.clone(),
            mix: FsirMix {
                main: source.mix.main.name.clone(),
                accents: source
                    .mix
                    .accents
                    .iter()
                    .map(|a| FsirAccent {
                        atom: a.atom.name.clone(),
                        ratio: a.ratio,
                    })
                    .collect(),
            },
            shape: FsirShape {
                name: source.shape.name.clone(),
            },
            intensity: intensity.clone(),
            smoothing,
        }
    }

    /// 序列化为 JSON 字符串。
    pub fn to_json(&self) -> Result<String, AnimiError> {
        serde_json::to_string_pretty(self).map_err(|e| AnimiError::InternalError {
            file_name: crate::error::current_file(),
            msg: format!("FSIR JSON 序列化失败: {}", e),
        })
    }

    /// 从 JSON 字符串反序列化。
    pub fn from_json(json: &str) -> Result<Self, AnimiError> {
        serde_json::from_str(json).map_err(|e| AnimiError::InternalError {
            file_name: crate::error::current_file(),
            msg: format!("FSIR JSON 反序列化失败: {}", e),
        })
    }

    /// 序列化为 Postcard 二进制（大端序风格 wire format）。
    ///
    /// Postcard 是 `#[no_std]` 兼容的 Serde 二进制格式：
    ///   - 变长整数编码（u32/u64 不会浪费高位零字节）
    ///   - 字符串/序列前导 varint 长度
    ///   - 结构体字段顺序串联（无分隔符/无填充）
    ///
    /// Go Server 端需要一个 Postcard encoder 来生成相同的字节流。
    /// Postcard wire format 规范简单（~2 页），适合手写 Go 端。
    pub fn to_binary(&self) -> Result<Vec<u8>, AnimiError> {
        postcard::to_allocvec(self).map_err(|e| AnimiError::InternalError {
            file_name: crate::error::current_file(),
            msg: format!("FSIR 二进制序列化失败: {}", e),
        })
    }

    /// 从 Postcard 二进制反序列化。
    ///
    /// Feelings-OS 裸机环境可用——Postcard 不依赖 std，不依赖 alloc 之外的任何东西。
    pub fn from_binary(bytes: &[u8]) -> Result<Self, AnimiError> {
        postcard::from_bytes(bytes).map_err(|e| AnimiError::InternalError {
            file_name: crate::error::current_file(),
            msg: format!("FSIR 二进制反序列化失败: {}", e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::typeck::TypeChecker;

    fn interlink(src: &str) -> Result<FsirDoc, AnimiError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let reg = crate::registry::Registry::default();
        let checker = TypeChecker::new(&reg);
        checker.check(&ast)?;
        let scaled = FsirIntensity {
            min: ast.intensity.min,
            max: ast.intensity.max,
        };
        Ok(FsirDoc::from_ast(&ast, None, None, &scaled, None))
    }

    #[test]
    fn fsir_json_minimal() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.3]
    }
    shape: gradual_rise_fall
    intensity: [15, 45]
}
"#;
        let doc = interlink(src).unwrap();
        let json = doc.to_json().unwrap();

        assert!(json.contains("calm"));
        assert!(json.contains("calm_meditative"));
        assert!(json.contains("belonging"));
        assert!(json.contains("0.3"));
        assert!(json.contains("gradual_rise_fall"));
        assert!(json.contains("15"));
        assert!(json.contains("45"));
        assert!(json.contains("animi_version"));
    }

    #[test]
    fn fsir_json_serialize_deserialize_roundtrip() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.3, clarity 0.2]
    }
    shape: sharp_peak
    intensity: [0, 100]
}
"#;
        let doc = interlink(src).unwrap();
        let json = doc.to_json().unwrap();

        // 反序列化回来——验证 JSON 结构完整
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "calm");
        assert_eq!(parsed["mix"]["main"], "calm_meditative");
        assert_eq!(parsed["mix"]["accents"][0]["atom"], "belonging");
        assert_eq!(parsed["mix"]["accents"][1]["ratio"], 0.2);
        assert_eq!(parsed["shape"]["name"], "sharp_peak");
        assert_eq!(parsed["intensity"]["min"], 0);
        assert_eq!(parsed["intensity"]["max"], 100);
    }

    #[test]
    fn fsir_binary_roundtrip() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.3, clarity 0.2]
    }
    shape: gradual_rise_fall
    intensity: [15, 45]
}
"#;
        let doc = interlink(src).unwrap();

        // round-trip: struct → binary → struct
        let bytes = doc.to_binary().unwrap();
        assert!(!bytes.is_empty(), "二进制输出不应为空");
        let doc2 = FsirDoc::from_binary(&bytes).unwrap();

        assert_eq!(doc2.name, "calm");
        assert_eq!(doc2.mix.main, "calm_meditative");
        assert_eq!(doc2.mix.accents.len(), 2);
        assert_eq!(doc2.mix.accents[0].atom, "belonging");
        assert_eq!(doc2.mix.accents[0].ratio, 0.3);
        assert_eq!(doc2.shape.name, "gradual_rise_fall");
        assert_eq!(doc2.intensity.min, 15);
        assert_eq!(doc2.intensity.max, 45);
    }

    #[test]
    fn fsir_json_binary_consistency() {
        // 同一份 FSIR → JSON 的语义内容 = binary 的语义内容
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [safety 0.1, warmth 0.2]
    }
    shape: slow_decay
    intensity: [5, 25]
}
"#;
        let doc = interlink(src).unwrap();

        // 分别从 JSON 和 binary 恢复
        let json = doc.to_json().unwrap();
        let bytes = doc.to_binary().unwrap();

        let from_json = FsirDoc::from_json(&json).unwrap();
        let from_binary = FsirDoc::from_binary(&bytes).unwrap();

        // 结构字段完全一致
        assert_eq!(from_json.name, from_binary.name);
        assert_eq!(from_json.mix.main, from_binary.mix.main);
        assert_eq!(from_json.mix.accents.len(), from_binary.mix.accents.len());
        for (a, b) in from_json
            .mix
            .accents
            .iter()
            .zip(from_binary.mix.accents.iter())
        {
            assert_eq!(a.atom, b.atom);
            assert_eq!(a.ratio, b.ratio);
        }
        assert_eq!(from_json.shape.name, from_binary.shape.name);
        assert_eq!(from_json.intensity.min, from_binary.intensity.min);
        assert_eq!(from_json.intensity.max, from_binary.intensity.max);
    }

    #[test]
    fn fsir_binary_compactness() {
        // 二进制格式应比 JSON 紧凑（JSON 有大量空白和引号）
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.3, clarity 0.2, safety 0.1, warmth 0.1]
    }
    shape: gradual_rise_fall
    intensity: [15, 45]
}
"#;
        let doc = interlink(src).unwrap();
        let json = doc.to_json().unwrap();
        let bytes = doc.to_binary().unwrap();

        // postcard 二进制应显著小于带格式的 JSON
        assert!(
            bytes.len() < json.len(),
            "binary {} bytes should be smaller than JSON {} bytes",
            bytes.len(),
            json.len()
        );
    }

    #[test]
    fn fsir_binary_corrupted_rejected() {
        // 随机垃圾字节 → from_binary 应返回 Err
        let garbage = vec![0xFF, 0x00, 0xAB, 0xCD, 0x01, 0x02];
        let result = FsirDoc::from_binary(&garbage);
        assert!(result.is_err());
    }

    #[test]
    fn fsir_meta_fields_present() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let doc = interlink(src).unwrap();
        let json = doc.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["meta"]["animi_version"], "0.1.0");
        assert!(parsed["meta"]["source_hash"].is_null());
        assert!(parsed["meta"]["pattern_registry_hash"].is_null()); // None → null in JSON
        assert!(parsed["meta"]["safety_rules_version"].is_null());
        assert!(parsed["meta"]["compiled_at"].is_string());
    }
}
