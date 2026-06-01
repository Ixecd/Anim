// src/registry.rs — Pattern Registry 的 v1.1 内建版本
//
// 单一定义点。main.rs / typeck / rule 都从这里读。
// v1.2+ 从外部 JSON/YAML 加载。

/// 原子类型——核心（官方审核）vs 沙箱（用户上传，未验证）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AtomClass {
    Core,
    Sandbox,
}

/// Registry 中的一个感受原子条目。
pub struct AtomEntry {
    pub name: &'static str,
    pub class: AtomClass,
    pub max_ratio: f64,
}

/// 内建 Registry——8 个核心原子。
static REGISTRY: &[AtomEntry] = &[
    AtomEntry {
        name: "calm_meditative",
        class: AtomClass::Core,
        max_ratio: 1.0,
    },
    AtomEntry {
        name: "belonging",
        class: AtomClass::Core,
        max_ratio: 0.5,
    },
    AtomEntry {
        name: "clarity",
        class: AtomClass::Core,
        max_ratio: 0.5,
    },
    AtomEntry {
        name: "safety",
        class: AtomClass::Core,
        max_ratio: 0.5,
    },
    AtomEntry {
        name: "post_achievement",
        class: AtomClass::Core,
        max_ratio: 0.3,
    },
    AtomEntry {
        name: "gentle_focus",
        class: AtomClass::Core,
        max_ratio: 0.5,
    },
    AtomEntry {
        name: "deep_rest",
        class: AtomClass::Core,
        max_ratio: 0.5,
    },
    AtomEntry {
        name: "warmth",
        class: AtomClass::Core,
        max_ratio: 0.5,
    },
];

/// 内建形状列表。
static SHAPES: &[&str] = &[
    "gradual_rise_fall",
    "sharp_peak",
    "steady",
    "slow_decay",
    "wave",
];

pub fn lookup_atom(name: &str) -> Option<&'static AtomEntry> {
    REGISTRY.iter().find(|e| e.name == name)
}

pub fn is_core(name: &str) -> bool {
    lookup_atom(name).is_some_and(|e| e.class == AtomClass::Core)
}

pub fn max_ratio(name: &str) -> Option<f64> {
    lookup_atom(name).map(|e| e.max_ratio)
}

pub fn atom_names() -> impl Iterator<Item = &'static str> {
    REGISTRY.iter().map(|e| e.name)
}

pub fn shape_names() -> impl Iterator<Item = &'static str> {
    SHAPES.iter().copied()
}
