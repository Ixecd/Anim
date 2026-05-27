# FORGET.md — 待修复项（P0 + P1）

> 扫描日期：2026-05-27
> 范围：语言规范（`Feelings-LANGUAGE.md`）+ 设计文档 + 代码（零行）
> 原则：只列 P0（生产命门）和 P1（功能受限），P2 Ops / P3 Polish 不提

---

## P0 — 生产命门（交织器不存在就无法工作）— 0/5

### 交织管线

1. **八 Pass 骨架零代码** — `src/` 目录不存在。Pass 0-8 全部未实现。FIXME: v0.2 milestone 1。依赖：AST 类型定义 + parser + type checker + safety passes + IR 生成。

2. **四层 IR 零代码** — FSIR / PSIR / DSIR / ESIR 仅在规范中定义，无任何 Rust struct 实现。FIXME: v0.2 milestone 1（FSIR），v0.3 milestone 2（PSIR/DSIR/ESIR）。

### 类型系统

3. **双层感受原子体系零代码** — 核心原子（Pattern Registry 查询）和沙盒原子（unverified 标记）的区分逻辑未实现。

### 安全模型

4. **三层安全防线零代码** — StaticSafety / UserStateSafety / RuntimeGuard 均为规范描述，零实现。创伤分型交叉判定矩阵未编码。

### PBM

5. **个人基线矩阵零代码** — 四维差异化冷启动系数（内脏 0.75 / 情绪 0.40 / 触觉 0.80 / 听觉 0.85）仅在规范中，无任何实现。

---

## P1 — 功能受限（规模化前必做）— 0/6

### 交织管线

6. **双流水线调度器零代码** — 设计文档已完成（ADR 004），实现方案已定稿。代码零行。
    FIXME: v0.4 milestone 3。

7. **设备算力感知交织零代码** — 入门/标准/高端三档信号参数密度调整逻辑已在 ADR 003 Pass 7 中定稿。代码零行。

### 语法

8. **Anim 宏系统零代码** — 语法树级展开、三层保证（类型检查 + 强度生命周期 + 创伤作用域）已在 ADR 005 中定稿。`macro_rules!` 解析器 + 展开器 + 递归深度保护零代码。

9. **动态 ratio 变量绑定零实现** — `@bind(skin_conductance_trend, range(min, max))` 语法未实现到 parser/typeck。

10. **自定义 shape 曲线零实现** — `keyframes` 语法 + 插值算法未实现。

11. **声明式注解展开零代码** — 五条 `@` 注解展开为 ESIR 插桩的逻辑已在 ADR 005 中定稿。代码零行。

### 设备对接

12. **FPGA 固件接口协议零设计** — ESIR 帧格式 → 硬件数据包的通信协议未定义。

### 设计文档审计缺陷（2026-05-27 MiniMax + Gemini 审计）

13. **oi 生命周期边界不清** — ADR 006 说编译期带 String、运行期是 bit，但 Pass 4（RuntimeGuard）在离线时跑，其 oi 是 String 还是 bit？边界未画清。FIXME: ADR 006 §oi 的双重生命周期。

14. **PBM 冷启动与收敛关系未说明** — PHILOSOPHY 说冷启动系数硬编码，ADR 003 说非线性 sigmoidal 查表。硬编码的值和可调的曲线之间是什么关系？收敛到哪去？

15. **自适应帧密度 vs 安全校验帧频率冲突** — ADR 003 Pass 8 说 safe frame 每 N 帧一个（N 固定），但自适应帧密度让 plateau 段 10ms/帧。如果 plateau 段只有 100ms → 只产 10 帧 → N=20 的 safe frame 约束会崩。N 是多少？冲突时谁优先？

16. **分支预测器 FPGA 实现细节缺失** — ADR 004 说预测器跑在 FPGA 上+"查寄存器+简单线性外推"，但 FPGA 的门级逻辑是静态的。外推逻辑怎么烧进去？训练数据存在哪？

17. **后台预交织触发条件未量化** — ADR 004 说"设备在充电。用户未佩戴"，两个条件是 AND 还是 OR？预交织源码从哪来？闲置久了的 PBM 漂移缓存新鲜度怎么保证？

18. **Span 元数据存储成本未估算** — ADR 005 每个宏展开节点带 Span（source_file + macro_name 两个 String）。不用 interning → 200 节点 = 几十 KB 冗余。用了 interning → 成本是多少？

19. **#[allow_hedge] 语法位置未定** — 写在宏调用方还是 mix 声明处？两个语义的优先级？

20. **Pass 命名不一致** — Pass 5 叫 FSIRGen，Pass 6-8 不叫 PSIRGen/DSIRGen/ESIRGen。应统一。

21. **ADR 002 五层方案反驳不够有力** — TSIR 和 ESIR 在语义上等价但没有说清楚。

22. **ADR 003 Pass 5 为什么不放在 Pass 1 之后** — FSIR 生成理论上在 TypeCheck 之后就可以做。放在安全层（Pass 2-4）之后的理由需要写清楚。

23. **验证标准无工程化路径** — "身体信了，深睡时长涨了"是哲学描述。需要量化：涨多少？测多久？基线是什么？

---

## 编辑记录

```
2026-05-20  v0.1 初始扫描
            - 语言规范 100%
            - 代码 0%
            - P0 5 项：八Pass / 四层IR / 原子体系 / 安全三层 / PBM
            - P1 6 项：双流水线调度 / 算力感知 / 动态ratio / 自定义shape /
                       注解展开 / FPGA协议
            - 全部 open，符合 v0.1 早期阶段预期

2026-05-26  v0.1.1 设计文档补全
            - 新增 ADR 002-006（IR架构 / Pass管线 / 双流水线 / 宏系统 / oi错误处理）
            - 新增 Anim-SAFETY.md、Anim-LOCALITY.md、Anim-CROSSOVER.md
            - P1 项更新：双流水线调度→设计已定稿代码零行；算力感知→设计已定稿；
              宏系统新增 P1；注解展开→设计已定稿代码零行
            - oi 作为错误处理约定已定稿——不是 Err 不是 Error，是 oi
            - P0 不变：代码仍零行，v0.2 milestone 1 开始

2026-05-27  v0.1.2 Gemini + MiniMax 审计
            - Gemini 四刺已修：oi硬件Trap / 控制回路+BRAM / Emergency Bypass+衰减状态机 / Span溯源
            - MiniMax 已修：Pass数量八→九 / SAFETY章节跳号 / Pattern Registry + 术语表
            - MiniMax 待修：11项新增 P1（见 §设计文档审计缺陷 #13-23）
            - 新增 Anim-FEELINGS.md / Anim-PATTERN-REGISTRY.md / GLOSSARY.md
            - P0 不变：代码仍零行
```
