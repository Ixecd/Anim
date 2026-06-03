// src/guard.rs — Pass 4：运行期插桩预埋 + oi 帧平滑过渡
//
// 交织期预埋——运行期激活。不在这里判断。
// 真正的判断在 FPGA 端由寄存器比较完成。
// 这一层做的事——生成栅栏代码——不是执行栅栏。
//
// v1.2 实现：
//   - OiSmoothing 衰减曲线——oi 帧拒绝时不硬截断，平滑过渡到安全基线
//   - inject() 根据源码强度选择平滑策略
//   - 待 Pass 8 CodeGen 接入：ESIR 帧级插值 + 硬件衰减状态机

use crate::ast::*;
use crate::error::AnimiError;
use serde::{Deserialize, Serialize};

// ── OiSmoothing —— oi 帧缺失平滑过渡 ──────────────────────────

/// 一次衰减步——当前帧强度 × multiplier。
///
/// `multiplier = 1.0` 表示本帧不变。`0.3` 表示降到当前强度的 30%。
/// `0.0` 表示回到安全基线（保底空包）。不是硬截断——是平滑降。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecayStep {
    /// 衰减乘数 [0.0, 1.0]。
    pub multiplier: f64,
}

impl DecayStep {
    /// 创建一个衰减步。multiplier 必须在 [0.0, 1.0] 范围内。
    pub fn new(multiplier: f64) -> Result<Self, &'static str> {
        if !(0.0..=1.0).contains(&multiplier) || multiplier.is_nan() {
            return Err("衰减乘数必须在 [0.0, 1.0] 范围内");
        }
        Ok(DecayStep { multiplier })
    }
}

/// oi 帧平滑过渡曲线。
///
/// 当运行期安全插桩（Pass 8 ESIR 帧级）检测到一帧应被拒绝时——
/// OiSmoothing 替代直接跳过（= abrupt_stop），在接下来 N 帧中平滑衰减到安全基线。
///
/// # 为什么需要平滑过渡
///
/// 直接跳过一帧 = 对岛叶而言是 **abrupt_stop**。
/// 岛叶的预期链（上一帧信号→"下一帧应该还有信号"）在无信号到达时断裂。
/// 强度越高，断裂冲击越大。ADR 003 的规则——abrupt_stop 形状强度 > 20 需要知情同意——
/// 在运行期间接放大了这个约束：强度 > 20 的帧被 oi 拒绝时，
/// 不能直接跳到零。必须在 4-8ms 内平滑降回安全基线。
///
/// # 和硬件衰减状态机（ADR 004）的关系
///
/// OiSmoothing 在交织期（Pass 4）预埋衰减系数。
/// Pass 8 CodeGen 将这些系数嵌入 ESIR 帧级安全插桩。
/// 触发时——FPGA 硬件衰减状态机按这些系数在 4-8ms 内平滑归零。
/// 不是软件循环。是门级逻辑。微秒级响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OiSmoothing {
    /// 衰减序列——从当前帧到安全基线的过渡。
    ///
    /// steps[0] = 当前帧（被拒绝的那一帧）应输出的衰减值。
    ///   multiplier = 1.0 = 完整输出被拒绝帧。仅在极端低强度（≤20）时使用。
    ///   multiplier = 0.0 = 直接到安全基线。仅强度 ≤ 20。
    /// steps[N-1] = 最后一帧过渡，然后接入保底空包。
    pub steps: Vec<DecayStep>,
}

impl OiSmoothing {
    /// 衰减步的总帧数。4 步 = 4ms（1ms/帧）。
    pub fn frame_count(&self) -> usize {
        self.steps.len()
    }

    /// 衰减总时长（毫秒）。frame_count × 1ms。
    pub fn duration_ms(&self) -> usize {
        self.frame_count()
    }

    /// 是否为硬截断（一帧到零）。仅强度 ≤ 20 允许。
    pub fn is_hard_cut(&self) -> bool {
        self.steps.len() == 1 && self.steps[0].multiplier == 0.0
    }

    /// 验证衰减曲线合法性。
    pub fn validate(&self) -> Result<(), String> {
        if self.steps.is_empty() {
            return Err("OiSmoothing 至少需要 1 个衰减步".into());
        }
        if self.frame_count() > 8 {
            return Err(format!(
                "衰减步数 {} 超过上限 8（8ms）。更长的衰减 = 信号残留过多",
                self.frame_count()
            ));
        }
        // 每一步 multiplier 必须在 [0,1]
        for (i, step) in self.steps.iter().enumerate() {
            if step.multiplier.is_nan() || !(0.0..=1.0).contains(&step.multiplier) {
                return Err(format!(
                    "第 {} 步衰减乘数 {} 超出 [0.0, 1.0]",
                    i, step.multiplier
                ));
            }
        }
        // 衰减必须单调不增
        for w in self.steps.windows(2) {
            if w[1].multiplier > w[0].multiplier {
                return Err(format!(
                    "衰减序列必须单调不增——{} → {} 是上升的",
                    w[0].multiplier, w[1].multiplier
                ));
            }
        }
        // 最后一步必须归零（否则信号永不终止）
        if self.steps.last().unwrap().multiplier != 0.0 {
            return Err("最后一步衰减乘数必须是 0.0（归零到安全基线）".into());
        }
        Ok(())
    }
}

// ── 平滑策略选择 ────────────────────────────────────────────

/// 根据源码强度区间选择 oi 帧拒绝时的平滑策略。
///
/// 强度 ≤ 20：一帧硬截断——直接跳到安全基线。突停幅度低，岛叶可承受。
/// 强度 > 20：4 帧非线性衰减——[×1.0, ×0.6, ×0.3, ×0.1, ×0.0]。
///             和 ADR 004 的硬件衰减状态机系数完全对齐。
pub fn smoothing_for_intensity(min: u32, max: u32) -> OiSmoothing {
    let peak = max.max(min); // 峰值 = 两个端点中较大的
    if peak <= 20 {
        // 低强度——可以硬截断
        OiSmoothing {
            steps: vec![DecayStep { multiplier: 0.0 }],
        }
    } else {
        // 中高强度——4 帧非线性衰减
        // Frame_N:   当前帧——信号被拒绝，但衰减状态机读取本帧原始值 × 1.0 作为衰减起点
        // Frame_N+1: ×0.6
        // Frame_N+2: ×0.3
        // Frame_N+3: ×0.1
        // Frame_N+4: ×0.0（安全基线保底空包）
        OiSmoothing {
            steps: vec![
                DecayStep { multiplier: 1.0 },
                DecayStep { multiplier: 0.6 },
                DecayStep { multiplier: 0.3 },
                DecayStep { multiplier: 0.1 },
                DecayStep { multiplier: 0.0 },
            ],
        }
    }
}

// ── inject：运行期插桩预埋 ──────────────────────────────────

/// 向 AST 注入运行期安全栅栏 + oi 帧平滑过渡配置。
///
/// v1.2：根据源码强度区间选择 OiSmoothing 策略。
/// v1.3+（Pass 8 接入后）：将 OiSmoothing 嵌入 ESIR 帧级安全插桩。
///
/// 当前做的事：
///   1. 验证源码强度区间合法
///   2. 根据强度选择平滑策略
///   3. 验证策略有效性
///
///   （不生成 ESIR——Pass 8 负责）
pub fn inject(source: &FeelingSource) -> Result<OiSmoothing, AnimiError> {
    // 强度 ≤ 0 拒绝——没有信号需要平滑过渡
    if source.intensity.max == 0 && source.intensity.min == 0 {
        return Err(AnimiError::StaticSafetyError {
            file_name: crate::error::current_file(),
            rule: "oi 帧平滑".into(),
            detail: "强度为零——不需要平滑过渡。这种源码不应到达 Pass 4。".into(),
        });
    }

    let smoothing = smoothing_for_intensity(source.intensity.min, source.intensity.max);

    // 验证策略有效性
    smoothing
        .validate()
        .map_err(|msg| AnimiError::InternalError {
            file_name: crate::error::current_file(),
            msg: format!("OiSmoothing 验证失败: {}", msg),
        })?;

    Ok(smoothing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::typeck::TypeChecker;

    fn inject_src(src: &str) -> Result<OiSmoothing, AnimiError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let reg = crate::registry::Registry::default();
        let checker = TypeChecker::new(&reg);
        checker.check(&ast)?;
        inject(&ast)
    }

    // ── inject 测试 ──────────────────────────────────────

    #[test]
    fn inject_low_intensity_returns_hard_cut() {
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
        let smoothing = inject_src(src).unwrap();
        assert!(smoothing.is_hard_cut());
        assert_eq!(smoothing.frame_count(), 1);
        assert_eq!(smoothing.duration_ms(), 1);
    }

    #[test]
    fn inject_high_intensity_returns_4_frame_decay() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [15, 45]
}
"#;
        let smoothing = inject_src(src).unwrap();
        assert!(!smoothing.is_hard_cut());
        assert_eq!(smoothing.frame_count(), 5);
        assert_eq!(smoothing.duration_ms(), 5);
    }

    #[test]
    fn inject_boundary_intensity_21_is_decay() {
        // 强度 > 20 → 衰减。边界值 21 应走衰减路径。
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: []
    }
    shape: steady
    intensity: [21, 21]
}
"#;
        let smoothing = inject_src(src).unwrap();
        assert!(!smoothing.is_hard_cut());
        assert_eq!(smoothing.frame_count(), 5);
    }

    // ── OiSmoothing::validate 测试 ───────────────────────────

    #[test]
    fn validate_hard_cut_passes() {
        let s = OiSmoothing {
            steps: vec![DecayStep { multiplier: 0.0 }],
        };
        assert!(s.validate().is_ok());
    }

    #[test]
    fn validate_4_frame_decay_passes() {
        let s = smoothing_for_intensity(15, 45);
        assert!(s.validate().is_ok());
    }

    #[test]
    fn validate_non_monotonic_rejected() {
        let s = OiSmoothing {
            steps: vec![
                DecayStep { multiplier: 0.6 },
                DecayStep { multiplier: 0.8 }, // 上升——非法
                DecayStep { multiplier: 0.0 },
            ],
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn validate_last_step_nonzero_rejected() {
        let s = OiSmoothing {
            steps: vec![
                DecayStep { multiplier: 1.0 },
                DecayStep { multiplier: 0.5 }, // 最后一步不是 0.0
            ],
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn validate_empty_steps_rejected() {
        let s = OiSmoothing { steps: vec![] };
        assert!(s.validate().is_err());
    }

    #[test]
    fn validate_too_many_steps_rejected() {
        let mut steps = Vec::new();
        for i in 0..9 {
            steps.push(DecayStep {
                multiplier: 1.0 - (i as f64) * 0.1,
            });
        }
        let s = OiSmoothing { steps };
        assert!(s.validate().is_err());
    }

    // ── smooth_for_intensity 测试 ─────────────────────────

    #[test]
    fn smoothing_for_peak_20_is_hard_cut() {
        let s = smoothing_for_intensity(5, 20);
        assert!(s.is_hard_cut());
    }

    #[test]
    fn smoothing_for_peak_21_is_decay() {
        let s = smoothing_for_intensity(10, 21);
        assert!(!s.is_hard_cut());
        assert_eq!(s.frame_count(), 5);
    }

    #[test]
    fn smoothing_for_peak_100_is_decay() {
        let s = smoothing_for_intensity(0, 100);
        assert!(!s.is_hard_cut());
        // 按 ADR 004：Frame_N → ×0.6 → ×0.3 → ×0.1 → 保底包
        let expected: Vec<f64> = vec![1.0, 0.6, 0.3, 0.1, 0.0];
        for (step, exp) in s.steps.iter().zip(expected.iter()) {
            assert_eq!(step.multiplier, *exp);
        }
    }

    #[test]
    fn decay_step_new_rejects_invalid() {
        assert!(DecayStep::new(1.5).is_err());
        assert!(DecayStep::new(-0.1).is_err());
        assert!(DecayStep::new(f64::NAN).is_err());
        assert!(DecayStep::new(0.0).is_ok());
        assert!(DecayStep::new(1.0).is_ok());
    }
}
