fix: sigmoidal公式修正 + 数学公式手册 + Gemini review后续项

- 核心修复：sigmoidal_scale() 公式修成 compression(1 - α × sigmoid)。
  消除了低强度信号被压碎、高强度过冲击的问题。
  low ≈ linear(10→10), mid压缩(50→38), high饱和(100→52)，完全对齐ADR 003。
- 新增 docs/design/009-pbm-math.md——全部公式的统一手冊。
  含 sigmoidal、冷启动四维系数、DampingMatrix、点缀比例帽。
- 新增5个sigmoidal测试（zero/low/mid/high/cap）+重新基于新公式的断言。
- FORGET P1新增#24（Shape校验不对称）、#25（DampingMatrix被架空）。
- 99 tests pass, 0 warnings.