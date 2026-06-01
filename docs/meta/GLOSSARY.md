# Anim 术语表

> Anim 特有的概念速查。按字母序排列。每个术语 = 英文名 + 中文名 + 一句话定义。

---

## A

- **animi** — Anim Interlinker。交织器。不叫编译器。把多股独立流织成信号绳的工具。

## C

- **cap** — 承载上限。用户当前被验证的强度上限。cap 35 = 不能交织强度 > 35 的感受。
- **cross checker** — 交叉检查器。Anim 的安全检查——不检查内存。检查交叉资格。替代 borrow checker。
- **cross_over** — 交叉。Anim 的一等操作。两根丝在同一个时空坐标相遇。共同产生一个值。不是借用。
- **核心原子 (Core Atom)** — 经过完整安全性验证的感受原子。全体用户。全强度区间。

## D

- **DSIR (Device Signal IR)** — 设备信号分配中间表示。PSIR 被分解为多设备协同信号序列。

## E

- **Emergency Bypass** — 硬件直连中断总线。后台检测到安全滑坡 → 硬件拉高 IRQ → FPGA 触发衰减状态机。不经过软件栈。

- **ESIR (Execution Signal IR)** — 执行信号中间表示。帧级指令（1ms/帧）。带闭环反馈指针。直接喂给 FPGA。

## F

- **FSIR (Feeling Structure IR)** — 感受结构中间表示。与人无关。与设备无关。纯粹的感受语义声明。

## H

- **硬件衰减状态机 (Hardware Decay Fader)** — FPGA 门级逻辑。Emergency Bypass 触发后——在 4-8ms 内将信号输出按固定非线性递减系数归零。不是硬截断。

## I

- **交织器 (Interlinker)** — 见 animi。

## K

- **绳结 (Knot)** — 一根丝的"地址"。不是内存地址——是这根丝和哪些丝交叉过的全部历史。上次交叉的位置 = 绳结。

## O

- **oi** — 交叉拒绝。不是错误。不是异常。是安全层在挡。"这帧交叉没发生。下一帧继续。"编译期带 String（人类可读）。运行期是硬件控制总线上的一个 Trap bit（0ns 延迟）。

## P

- **Pattern Registry** — 感受原子注册表。每个感受原子 → 一组神经通路映射 + 安全参数。Anim 的类型系统底座。

- **PBM (Personal Baseline Matrix)** — 个人基线矩阵。四维差异化冷启动（内脏/情绪/触觉/听觉）。永不离设备。

- **Pass** — 交织阶段。从 Pass 0（LexParse）到 Pass 8（CodeGen）。共九 Pass。每个 Pass 只做一件事——织入一股新信息流或施加一层新安全约束。

- **PSIR (Personal Signal IR)** — 个人适配信号中间表示。FSIR × PBM。此人的平静 = 迷走神经 0.38mA（不是通用的 0.5mA）。

## R

- **沙盒原子 (Sandbox Atom)** — 未经验证的感受原子。≤30 强度。创作者本人使用。不给创伤协议用户。不给未成年人。

## S

- **丝 (Thread)** — Anim 的基本数据单元。一股独立的信号流。有自己的纺锤（Spindle）——产生它的源。

- **纺锤 (Spindle)** — 丝的源头。FSIR 的纺锤 = .anim 源码。PSIR 的纺锤 = PBM。DSIR 的纺锤 = 设备集合。ESIR 的纺锤 = 实时回读环。

## T

- **创伤作用域 (Trauma Scope)** — Anim 宏的 `#[trauma_scope(v1, v2)]` 注解。声明哪些创伤阶段的用户可以安全使用此宏。v3 不在作用域 → Pass 3 拒绝。

## U

- **回退 (Uncross)** — 交叉点被撤销。不是 memory deallocation。是"这一帧的交叉不再有效"。安全停止帧触发 → 当期帧交叉回退。经线不断。

## W

- **经线 (Warp)** — 纵向基轴。感受结构（FSIR）、个人基线（PBM）、安全阈值。不动才会有稳定的交叉点。

- **织入 (Weave)** — 一根纬线穿过一整排经线。一路产生交叉点。交叉点连成线 = 织入完成。

- **纬线 (Weft)** — 横向穿过。每帧在变。上一帧的心率。当前帧的偏差修正量。纬线每织一次换一根。不保留。
