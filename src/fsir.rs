// src/fsir.rs — Pass 5：FSIR 生成
//
// AST（FeelingSource）→ FSIR 文档（FsirDoc）。
// FSIR 是感受结构的"通用本体"——不绑定任何个人生理参数或设备。
// 同一份 .anim → 同一份 FSIR → 不同的人上 PSIR 不同。

use crate::ast::*;
use crate::error::AnimiError;
use serde::Serialize;

/// FSIR 文档——编译产物的顶层结构。
///
/// 这是 `.anim` 源码经过 Pass 0-4 之后的第一份 IR。
/// 所有后续 Pass（Personalize/DeviceMap/CodeGen）的输入。
#[derive(Debug, Clone, Serialize)]
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
}

/// FSIR 编译元数据。
#[derive(Debug, Clone, Serialize)]
pub struct FsirMeta {
    /// 编译器版本。
    pub animi_version: String,

    /// 编译时间（RFC 3339）。
    pub compiled_at: String,

    /// 源码的 SHA-256 哈希（后续版本接入 SPL 锚定）。
    /// v1.1 暂为空字符串。
    pub source_hash: String,
}

/// FSIR 混音结构。
#[derive(Debug, Clone, Serialize)]
pub struct FsirMix {
    /// 主旋律感受原子。
    pub main: String,

    /// 点缀列表。
    pub accents: Vec<FsirAccent>,
}

/// FSIR 点缀条目。
#[derive(Debug, Clone, Serialize)]
pub struct FsirAccent {
    /// 感受原子名。
    pub atom: String,

    /// 配比系数 [0.0, 1.0]。
    pub ratio: f64,
}

/// FSIR 形状曲线。
#[derive(Debug, Clone, Serialize)]
pub struct FsirShape {
    /// 形状名称（如 gradual_rise_fall）。
    pub name: String,
}

/// FSIR 强度区间。
#[derive(Debug, Clone, Serialize)]
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
    pub fn from_ast(source: &FeelingSource, source_hash: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| format!("{}", d.as_secs()))
            .unwrap_or_else(|_| "unknown".into());

        FsirDoc {
            meta: FsirMeta {
                animi_version: "0.1.0".into(),
                compiled_at: now,
                source_hash: source_hash.into(),
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
            intensity: FsirIntensity {
                min: source.intensity.min,
                max: source.intensity.max,
            },
        }
    }

    /// 序列化为 JSON 字符串。
    pub fn to_json(&self) -> Result<String, AnimiError> {
        serde_json::to_string_pretty(self).map_err(|e| AnimiError::InternalError {
            msg: format!("FSIR JSON 序列化失败: {}", e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::typeck::TypeChecker;

    fn compile(src: &str) -> Result<FsirDoc, AnimiError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let checker = TypeChecker::new();
        checker.check(&ast)?;
        Ok(FsirDoc::from_ast(&ast, ""))
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
        let doc = compile(src).unwrap();
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
        let doc = compile(src).unwrap();
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
        let doc = compile(src).unwrap();
        let json = doc.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["meta"]["animi_version"], "0.1.0");
        // compiled_at 是动态时间戳——只验证存在
        assert!(parsed["meta"]["compiled_at"].is_string());
    }
}
