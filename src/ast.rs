// src/ast.rs — Anim AST 类型定义
//
// 对应规范：Feelings-LANGUAGE.md §三、Anim 源码结构
// .anim 源码经过词法分析和语法分析后，生成这里的 AST 节点。
// AST 是交织器所有后续 Pass 的输入。

use serde::Serialize;

/// 源码位置——无损追踪。跨 Pass 传递，不依赖 Thread Local。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

/// 一份 .anim 源码的完整 AST。
///
/// 三层结构：
///   包名（feeling name）  = 基本感受大类。人类标签。如 `calm`、`focus`、`rest`。
///   主旋律（main atom）    = 基本感受的某种细分变体。岛叶信号。如 `calm_meditative`。
///   点缀（accents）        = 对主旋律的修饰。共同织成感受绳的另一股丝。
///
/// 同一个包名下可以有多种主旋律——"冷静"不是一个东西，是一族状态。
///
/// 顶层结构：
/// ```text
/// feeling calm {
///     mix {
///         main: calm_meditative
///         accents: [belonging 0.3, clarity 0.2]
///     }
///     shape: gradual_rise_fall
///     intensity: [15, 45]
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct FeelingSource {
    /// 感受包名称——feeling 关键字后面的标识符。
    pub name: String,

    /// 混音结构——主旋律 + 点缀列表。
    pub mix: Mix,

    /// 形状曲线名称（如 gradual_rise_fall、sharp_peak）。
    pub shape: Shape,

    /// 强度区间 [min, max]，单位由设备固件定义。
    pub intensity: Intensity,
}

/// 混音结构——定义感受的主旋律和点缀配比。
///
/// 主旋律 = 感受包的核心感受原子。只有 1 个。
/// 点缀 = 辅助感受原子列表，每个带一个配比系数（0.0-1.0）。
#[derive(Debug, Clone, Serialize)]
pub struct Mix {
    /// 主旋律——感受包的核心。只有 1 个。
    pub main: FeelingAtom,

    /// 点缀列表——辅助感受原子 + 各自配比。
    pub accents: Vec<Accent>,
}

/// 点缀条目——一个感受原子 + 它的配比系数。
#[derive(Debug, Clone, Serialize)]
pub struct Accent {
    pub atom: FeelingAtom,
    pub ratio: f64,
    pub span: Span,
}

/// 感受原子——Pattern Registry 中注册的一个命名感受单元。
///
/// 名称格式：小写字母 + 下划线。如 `calm_meditative`、`post_achievement`。
/// 类型检查（Pass 1）会验证这个名称是否在 Registry 中存在。
#[derive(Debug, Clone, Serialize)]
pub struct FeelingAtom {
    pub name: String,
    pub span: Span,
}

/// 形状曲线——定义感受随时间展开的方式。
///
/// 预定义的形状名称如 `gradual_rise_fall`（缓升缓降）、`sharp_peak`（尖峰）。
#[derive(Debug, Clone, Serialize)]
pub struct Shape {
    /// 形状名称。如 "gradual_rise_fall"。
    pub name: String,
}

/// 强度区间——感受的最低和最高强度。
///
/// 单位由设备固件定义。不是毫安——是抽象强度单位。
#[derive(Debug, Clone, Serialize)]
pub struct Intensity {
    /// 最低强度。
    pub min: u32,

    /// 最高强度。必须 >= min。
    pub max: u32,
}

impl Intensity {
    /// 验证强度区间合法性。
    pub fn validate(&self) -> Result<(), String> {
        if self.max < self.min {
            return Err(format!(
                "强度区间 [{}..{}] 不合法：max 必须 >= min",
                self.min, self.max
            ));
        }
        Ok(())
    }
}
