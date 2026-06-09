docs: 重写数学与约束规范—全源码公式收拢至统一文档 (2026-06-09)

009-math-and-constraints.md 完整重写为11章:
- §一 Sigmoidal 个人强度缩放—compression factor 公式+数值表
- §二 强度等比缩放—scale_intensity() 公式+边界条件
- §三 PBM冷启动四维系数表
- §四 DampingMatrix—完整冻结判定表+各维度梯度threshold
- §五 SessionLabel→PbmUpdateStrategy 映射表
- §六 DataConfidence—step_multiplier 三级乘数表
- §七 ColdStartGuard 默认参数+规则
- §八 OiSmoothing 衰减曲线—≤20硬截断/ >20五帧衰减
- §九 点缀比例帽—Core/Sandbox + Damping态调整公式
- §十 静态安全约束速查表
- §十一 Trauma路径重定向规则(未实现)
覆盖面: personalize.rs / safety.rs / guard.rs / pbm.rs / rule.rs / ast.rs
全部公式与此文件单一定义—代码读取、测试验证、benchmark 校准从此统一入口