# FORGET.md — 待修复项（P0 + P1 + P2）

> 扫描日期：2026-06-03
> 范围：代码（src/ 14 模块，91 测试）+ 设计文档（8 ADR）
> 原则：P0 = 生产命门。P1 = 功能受限。P2 = 代码质量/可维护性。
> 命名：animi 是交织器（interlinker），不是编译器。

---

## P0 — 生产命门（5/9）

### 安全体系

1. **三层安全防线不全** — rule.rs 已实现基础 3 条（全局强度上限 100、abrupt_stop≤20、沙箱限制）。guard.rs（Pass 4）已实现 OiSmoothing 衰减曲线，不再是空桩。safety.rs（Pass 3）仍为空桩——无创伤分型交叉判定、无未成年标记。FIXME: v0.3。

2. **Session 中用户状态突变安全盲区** — PSIR 只在启动时校验一次。运行中 cap 从 45 降到 30 但旧参数继续输出。需 10Hz 轻量安全看门狗。FIXME: ADR 009。

3. **混合原子包强度叠加绕过沙箱** — 核心+沙箱原子混合时总强度可能超上限。沙箱强度必须独立校验。FIXME: ADR 009。

4. **创伤用户点缀配比的绝对强度** — 禁主不禁点+配比≤0.10，但不考虑绝对强度×创伤敏感系数。需创伤敏感系数+绝对阈值。FIXME: ADR 009。

5. ~~**oi 帧缺失无平滑处理**~~ ✅ — OiSmoothing 衰减曲线已实现。强度≤20 硬截断，>20 四帧非线性衰减 [×1.0,×0.6,×0.3,×0.1,×0.0]。对齐 ADR 004 硬件衰减状态机。Pass 8 接入后嵌入 ESIR 帧级插桩。`src/guard.rs`。

### 架构

6. ~~**异常终止 Session 数据污染**~~ ✅ — `SessionLabel` + `DataConfidence` + `PbmUpdateStrategy` 三级置信度分级已实现。Abnormal→仅安全阈值更新。Contaminated→步长×0.0。`src/pbm.rs`。

7. ~~**新用户预测器冷启动灾难**~~ ✅ — `ColdStartGuard` 已实现。前 10 次 Session 强制关闭预测器，`predictor_enabled()` 返回 false。`src/pbm.rs`。

8. ~~**跨维度阻尼矩阵无具体参数**~~ ✅ — `DampingMatrix` 参数表已实现。情绪梯度>2.0×步长→冻结内脏+触觉，内脏>1.5×→冻结情绪。具体阈值+`should_freeze()`+`apply()`。`src/pbm.rs`。

9. ~~**FSIR 跨语言 ABI 零设计**~~ ✅ — Postcard 二进制格式已实现。`to_binary`/`from_binary`/`from_json` 方法。双格式并存: JSON 调试 + Postcard 设备缓存。ADR 008 已定稿。`src/fsir.rs` + `docs/design/008-fsir-abi.md`。

---

## P1 — 功能受限（3/14）

### 交织管线

10. **四层 IR 仅 FSIR 实现** — PSIR/DSIR/ESIR 仅为规范描述。FIXME: v0.3。

11. **Pass 6-8（Personalize/DeviceMap/CodeGen）零代码** — FIXME: v0.3。

12. **PBM 地基就绪，完整冷启动未实现** — `pbm.rs` 已有 `SessionLabel`/`DataConfidence`/`ColdStartGuard`/`DampingMatrix`。四维差异化冷启动系数（内脏0.75/情绪0.40/触觉0.80/听觉0.85）+ sigmoidal 收敛 + 因子三实时置信度待 v0.3。FIXME: v0.3。

### 设备

13. **设备热插拔安全重校验缺失** — 设备断开重连后 DSIR 不重做安全校验。任何热插拔→触发 DSIR 级重校验。FIXME: ADR 009。

14. **outline 模式安全约束缺失** — 缺设备时降级行为未定义。每个原子需声明设备依赖等级+降级策略。FIXME: ADR 009+。

15. **多设备时钟同步与无线抖动零设计** — 耳后有线 1ms 帧，腕部 BLE 天然 2-3ms 抖动。迟到帧丢弃还是缓存。需 Jitter Buffer 规约。FIXME: ADR 009。

### 语法

16. **Anim 宏系统零代码** — ADR 005 已定稿。`macro_rules!` 解析器+展开器未实现。

17. **宏递归组合风险绕过** — 组合风险必须在完全展开后的 AST 上计算，非展开前。FIXME: 宏展开阶段实现时。

21. **科学计数法浮点字面量不支持** — `1e-5` / `2.5E-3` 等科学计数法格式词法分析器未识别，对极小配比场景（敏感用户）不友好。需要 lexer.rs 的 `number()` 分支增加 `e`/`E` + 可选 `+`/`-` 解析。FIXME: ADR 009。

### 缓存

18. **后台预编译 FSIR 缓存失效策略缺失** — 安全规则更新后本地缓存仍用旧参数。FSIR 需携带规则版本号+Registry 哈希。FIXME: FSIR meta 字段扩展。

### 可观测性

19. **运行期 oi 可观测性为零** — 硬件 bit 不携带上下文。FPGA 需 oi 原因寄存器（4 位编码 16 种原因）。FIXME: v0.5。

### 代码功能

~~20. **错误信息无文件名**~~ ✅ — thread-local `CURRENT_FILE` + `current_file()` 辅助函数，oi! 宏 + 手动构造全带文件名。

---

## P2 — 代码质量 / 可维护性（0/0）

~~25. **无日志系统**~~ ✅ — `A_info!`/`A_warn!`/`A_error!` 宏，时间戳+ANSI颜色。KubePivot用P，Anim用A。

~~26. **错误路径测试覆盖不足**~~ ✅ — 空源/纯空白/纯注释/空文件/缺少main/缺少mix，60 测试全绿。

~~27. **无版本兼容性检查**~~ — 不需要。和 KubePivot 一样——Anim 不检查版本，通过 Registry 哈希保证兼容。FSIR 是 JSON——天然向前兼容。

---

## 已完成（v1.1）

- ✅ Pass 0a 词法分析（lexer.rs）——关键字/标识符/数字/标点/注释/行号
- ✅ Pass 0b 语法分析（parser.rs）——递归下降，顺序无关，重复检测
- ✅ Pass 1 类型检查（typeck.rs）——内建 Registry 8 原子+5 shape，ratio/max_ratio
- ✅ Pass 2 静态安全（rule.rs）——全局强度上限 100，abrupt_stop≤20，沙箱限制
- ✅ Pass 5 FSIR 生成（fsir.rs）——AST→FsirDoc→JSON，RFC 3339 时间戳
- ✅ CLI 入口（main.rs）——animi <file.anim> → file.json
- ✅ oi! 宏 + oi_err! 宏——轻量拒绝
- ✅ AnimiError 6 变体——Lex/Parse/TypeCheck/StaticSafety/UserStateSafety/Internal
- ✅ 91 单元测试全绿
- ✅ CRLF 行尾处理
- ✅ ratio [0.0,1.0] 基础校验 + per-atom max_ratio
- ✅ 强度零值拒绝 + 10000 上限
- ✅ shape 建议算法（前缀+子串匹配）
- ✅ 术语纠正——animi 是交织器，不是编译器
- ✅ Registry 外部化——JSON 文件加载 + 硬编码 fallback（P1 #22）
- ✅ 错误码自动生成——build.rs → docs/error-codes.md（P1 #23）
- ✅ 源码 SHA-256 哈希——SPL 锚定就绪
- ✅ scale_intensity 接入 main——`--cap` 参数 + `check_with_scale`
- ✅ abrupt_stop 加入 shapes——rule.rs 拦截生效
- ✅ Registry from_file 返回 Result——不再静默回退
- ✅ 四示例覆盖四大类（calm/focus/rest/post_achievement）
- ✅ 魔法数字提取——`MAX_GLOBAL_INTENSITY`、ratio 范围（P2 #24）
- ✅ 错误信息加文件名——thread-local `CURRENT_FILE`，91 单元测试全绿（P1 #20）
- ✅ 日志系统——`A.debug/info/warn/error` + 四级过滤 + UTC ISO8601 + `--log-level` 参数（P2 #25）
- ✅ 错误路径测试——空源/纯空白/纯注释/空文件/缺少main/缺少mix（P2 #26）
- ✅ FSIR Postcard 二进制 ABI——`to_binary`/`from_binary`，双格式并存，ADR 008 定稿（P0 #9）
- ✅ oi 帧平滑过渡——OiSmoothing 衰减曲线，强度≤20 硬截断，>20 四帧衰减，对齐 ADR 004（P0 #5）
- ✅ PBM 地基——SessionLabel + DataConfidence 数据置信度分级 + ColdStartGuard 冷启动守护 + DampingMatrix 跨维度阻尼矩阵（P0 #6/#7/#8）

---

## 设计决策

### D01: 包名与感受原子解耦——三层结构

```
feeling <基本感受包名> {
    mix {
        main: <基本感受的某种细分变体>
        accents: [<点缀1> <配比>, <点缀2> <配比>, ...]
    }
    shape: <时间形状>
    intensity: [<min>, <max>]
}
```

- **包名**（如 `calm`、`focus`、`rest`）——基本感受大类。是给创作者的人类标签。日常语言里"冷静"是一个笼统的词。
- **主旋律**（如 `calm_meditative`）——基本感受的某种细分变体。是给岛叶的信号。同一个包名下可以有多种主旋律——"冷静是一族状态。"
- **点缀**（如 `belonging 0.3`）——对主旋律的修饰。不是附属——是共同织成感受绳的另一股丝。

创作者先选基本感受——再选具体变体——再加点缀和形状。Anim 不替创作者决定——只保证类型正确+安全。

---

## 编辑记录

```
2026-06-03  v0.1.15 P0 第1轮——5/9
            - P0 #9: FSIR Postcard 二进制 ABI + ADR 008
            - P0 #5: oi 帧平滑过渡 OiSmoothing + guard.rs 不再是空桩
            - P0 #6: SessionLabel + DataConfidence 数据置信度分级
            - P0 #7: ColdStartGuard 冷启动守护
            - P0 #8: DampingMatrix 跨维度阻尼矩阵参数表
            - 新增 pbm.rs 模块，91 测试全绿

2026-06-01  v0.1.14 P0/P1/P2 重构
            - 全部条目按 P0(生产命门)/P1(功能受限)/P2(代码质量)重组
            - P0 9项：安全盲区5+架构4
            - P1 14项：FSIR跨语言ABI/设备热插拔/oi可观测性/宏系统/缓存失效/科学计数法
            - P2 4项：魔法数字/日志/错误测试/版本兼容
            - 新增"已完成"章节——v1.1 交付清单
            - 12个架构盲区全部归入对应优先级
            - 14个代码审计问题全部归入对应优先级

2026-06-01  v0.1.13 深度架构审计——12个设计盲区
            - animi 不是编译器——是交织器（interlinker）
            - #1-#12 全部入 FORGET

2026-06-01  v0.1.12 强度刻度设计决策
            - 0-100 默认刻度，u32 底层，FP 整数一拍

2026-06-01  v0.1.11 感受的全面性——外部锚点感受也是感受
            - 碾压→被接住，暴富→够了。四层防火墙。

2026-06-01  v0.1.10 Stream First 架构对齐
            - Anim=五股流的编译期预编织

2026-06-01  v0.1.9 设计审计
            - ADR003 八→九 Pass，ADR004 管线图补 Pass 4

2026-05-30  v0.1.8 xattr 文件元数据

2026-05-30  v0.1.7 FPGA 为什么无处不在但你看不见

2026-05-30  v0.1.6 FPGA 计算存储一体

2026-05-30  v0.1.5 第二纪元模拟层——ZENO 生物模拟+物理求解器

2026-05-30  v0.1.4 实现路径更新——可视化编辑器 ZENO 节点图

2026-05-30  v0.1.3 技术审计——智能指针/指令集/FSIR JSON/驱动边界

2026-05-27  v0.1.2 MiniMax + Gemini 审计（11项）

2026-05-27  v0.1.1 初始扫描——架构规范 100%，代码 0%
```