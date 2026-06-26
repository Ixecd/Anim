// src/registry.rs — Pattern Registry
//
// 单一定义点。main.rs / typeck / rule 都从这里读。
// 支持从外部 JSON 文件加载——未提供则用内建 fallback。

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::{AnimiError, Severity};
use crate::pbm::PbmDimension;

/// 原子类型——核心（官方审核）vs 沙箱（用户上传，未验证）。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum AtomClass {
    Core = 0,
    Sandbox = 1,
}

/// 90+ 强度沙箱治理响应分类 —— ADR 009 v0.5 §十.8
///
/// 不是所有 90+ 都是威胁。原子级别的差异化治理：
/// - 成就/高峰体验 → 不降级，只监控
/// - 物理矛盾信号 → 直接熔断
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum SandboxResponse {
    Attainment = 0,
    #[default]
    Neutral = 1,
    Caution = 2,
    Shield = 3,
}

fn default_dimension() -> PbmDimension {
    PbmDimension::Emotional
}

/// Registry 中的一个感受原子条目（可序列化版本——用于外部 JSON 加载）。
#[derive(Debug, Clone, Deserialize)]
struct AtomDef {
    name: String,
    class: AtomClass,
    max_ratio: f64,
    #[serde(default = "default_dimension")]
    dimension: PbmDimension,
    #[serde(default)]
    sandbox_response: SandboxResponse,
}

/// 运行时 Registry。由外部 JSON 或内建列表初始化。
pub struct Registry {
    atoms: Vec<AtomEntry>,
}

/// Registry 中的一个感受原子条目。
#[derive(Debug, Clone)]
pub struct AtomEntry {
    pub name: String,
    pub class: AtomClass,
    pub max_ratio: f64,
    /// 该原子的主感受维度——用于 PBM 四维系数选择。
    pub dimension: PbmDimension,
    /// 90+ 强度沙箱治理响应分类。
    pub sandbox_response: SandboxResponse,
}

impl Default for Registry {
    fn default() -> Self {
        Self::from_builtin()
    }
}

impl Registry {
    /// 从外部 JSON 文件加载。失败 → 返回错误信息。
    pub fn from_file(path: &str) -> Result<Self, AnimiError> {
        let json = std::fs::read_to_string(path).map_err(|e| AnimiError::InternalError {
            severity: Severity::Deny,
            file_name: path.to_string(),
            msg: format!("读取 Registry 失败: {}", e),
        })?;
        let defs: Vec<AtomDef> =
            serde_json::from_str(&json).map_err(|e| AnimiError::InternalError {
                severity: Severity::Deny,
                file_name: path.to_string(),
                msg: format!("Registry JSON 解析失败: {}", e),
            })?;
        let atoms: Vec<AtomEntry> = defs
            .into_iter()
            .map(|d| {
                if d.max_ratio < 0.0
                    || d.max_ratio > 1.0
                    || d.max_ratio.is_nan()
                    || d.max_ratio.is_infinite()
                {
                    return Err(AnimiError::InternalError {
                        severity: Severity::Deny,
                        file_name: path.to_string(),
                        msg: format!(
                            "原子 '{}' 的 max_ratio {} 必须在 [0.0, 1.0] 范围内",
                            d.name, d.max_ratio
                        ),
                    });
                }
                Ok(AtomEntry {
                    name: d.name,
                    class: d.class,
                    max_ratio: d.max_ratio,
                    dimension: d.dimension,
                    sandbox_response: d.sandbox_response,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Registry { atoms })
    }

    /// 内建 fallback——8 个核心原子。
    fn from_builtin() -> Self {
        use PbmDimension::*;
        use SandboxResponse::*;
        let builtin = vec![
            ("calm_meditative", AtomClass::Core, 1.0, Emotional, Neutral),
            ("belonging", AtomClass::Core, 0.5, Emotional, Neutral),
            ("clarity", AtomClass::Core, 0.5, Auditory, Neutral),
            ("safety", AtomClass::Core, 0.5, Visceral, Shield),
            (
                "post_achievement",
                AtomClass::Core,
                0.3,
                Emotional,
                Attainment,
            ),
            ("gentle_focus", AtomClass::Core, 0.5, Auditory, Caution),
            ("deep_rest", AtomClass::Core, 0.5, Visceral, Shield),
            ("warmth", AtomClass::Core, 0.5, Tactile, Neutral),
        ];
        let atoms = builtin
            .into_iter()
            .map(|(n, c, r, d, s)| AtomEntry {
                name: n.into(),
                class: c,
                max_ratio: r,
                dimension: d,
                sandbox_response: s,
            })
            .collect();
        Registry { atoms }
    }

    pub fn lookup(&self, name: &str) -> Option<&AtomEntry> {
        self.atoms.iter().find(|e| e.name == name)
    }

    pub fn is_core(&self, name: &str) -> bool {
        self.lookup(name)
            .is_some_and(|e| e.class == AtomClass::Core)
    }

    /// Registry 的 SHA-256 哈希——用于 FSIR 缓存失效。
    /// v0.4: 先按名称排序再计算哈希——消除原子顺序对哈希的影响。
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        let mut sorted: Vec<&AtomEntry> = self.atoms.iter().collect();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        for atom in sorted {
            hasher.update(atom.name.as_bytes());
            hasher.update([atom.class as u8]);
            hasher.update(atom.max_ratio.to_be_bytes());
            hasher.update([atom.sandbox_response as u8]);
        }
        hex::encode(hasher.finalize())
    }

    pub fn max_ratio(&self, name: &str) -> Option<f64> {
        self.lookup(name).map(|e| e.max_ratio)
    }

    /// 查询原子的沙箱治理响应分类。未找到 → None。
    pub fn sandbox_response(&self, name: &str) -> Option<SandboxResponse> {
        self.lookup(name).map(|e| e.sandbox_response)
    }

    pub fn atom_names(&self) -> impl Iterator<Item = &str> {
        self.atoms.iter().map(|e| e.name.as_str())
    }
    pub fn shape_names(&self) -> impl Iterator<Item = &'static str> {
        SHAPES.iter().copied()
    }
}

/// 内建形状列表。
static SHAPES: &[&str] = &[
    "gradual_rise_fall",
    "sharp_peak",
    "steady",
    "slow_decay",
    "wave",
    "abrupt_stop",
];

pub fn shape_names() -> impl Iterator<Item = &'static str> {
    SHAPES.iter().copied()
}
