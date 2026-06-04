// src/registry.rs — Pattern Registry
//
// 单一定义点。main.rs / typeck / rule 都从这里读。
// 支持从外部 JSON 文件加载——未提供则用内建 fallback。

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::AnimiError;

/// 原子类型——核心（官方审核）vs 沙箱（用户上传，未验证）。
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AtomClass {
    Core,
    Sandbox,
}

/// Registry 中的一个感受原子条目（可序列化版本——用于外部 JSON 加载）。
#[derive(Debug, Clone, Deserialize)]
struct AtomDef {
    name: String,
    class: AtomClass,
    max_ratio: f64,
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
            file_name: path.to_string(),
            msg: format!("读取 Registry 失败: {}", e),
        })?;
        let defs: Vec<AtomDef> =
            serde_json::from_str(&json).map_err(|e| AnimiError::InternalError {
                file_name: path.to_string(),
                msg: format!("Registry JSON 解析失败: {}", e),
            })?;
        let atoms: Vec<AtomEntry> = defs
            .into_iter()
            .map(|d| AtomEntry {
                name: d.name,
                class: d.class,
                max_ratio: d.max_ratio,
            })
            .collect();
        Ok(Registry { atoms })
    }

    /// 内建 fallback——8 个核心原子。
    fn from_builtin() -> Self {
        let builtin = vec![
            ("calm_meditative", AtomClass::Core, 1.0),
            ("belonging", AtomClass::Core, 0.5),
            ("clarity", AtomClass::Core, 0.5),
            ("safety", AtomClass::Core, 0.5),
            ("post_achievement", AtomClass::Core, 0.3),
            ("gentle_focus", AtomClass::Core, 0.5),
            ("deep_rest", AtomClass::Core, 0.5),
            ("warmth", AtomClass::Core, 0.5),
        ];
        let atoms = builtin
            .into_iter()
            .map(|(n, c, r)| AtomEntry {
                name: n.into(),
                class: c,
                max_ratio: r,
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
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        for atom in &self.atoms {
            hasher.update(atom.name.as_bytes());
            hasher.update(format!("{:?}", atom.class).as_bytes());
            hasher.update(atom.max_ratio.to_be_bytes());
        }
        hex::encode(hasher.finalize())
    }

    pub fn max_ratio(&self, name: &str) -> Option<f64> {
        self.lookup(name).map(|e| e.max_ratio)
    }

    pub fn atom_names(&self) -> impl Iterator<Item = &str> {
        self.atoms.iter().map(|e| e.name.as_str())
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
