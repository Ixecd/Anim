# Anim-LOCALITY — 时间局部性、空间局部性与超流水线的物理约束

> 性质：架构约束，不是建议
> 范围：Anim 所有 IR 数据结构布局、ESIR 帧缓冲、FPGA DMA、Pass 管线缓存策略
> 核心：1ms 一帧不是软件 deadline——是物理硬上限。在这个时钟域里，一次 L3 cache miss 的代价可能让一帧直接超时。时间局部性和空间局部性不是在优化 Anim——是在决定 Anim 能不能跑。四层 IR 的数据布局、八 Pass 的缓存策略、双流水线的热路径隔离——全部以局部性为第一原则。不是快慢问题。是过不过得去。

---

## 零、1ms 的物理约束

```
1ms 内 FPGA 门级逻辑必须完成：
    读 ESIR 帧 → DAC 输出信号 → ADC 回读生理数据 → 偏差计算 → 下一帧修正

一次 L3 cache miss = ~40ns（最好情况）到 ~100ns+。
一次主内存访问 = ~100ns。
一次磁盘 I/O = ~10ms —— 不在这个时钟域里。

1ms = 1,000,000ns。
一次主内存访问占 0.01%。
看起来很小。

但 1ms 内要做几百次这样的访问——
加上 FPGA 门延迟、DAC 建立时间、ADC 采样周期。
几百次 × ~100ns = 几十微秒。
再加上流水线里的其他操作——Cache miss 累积到 ~100μs 以上——
帧超时。Session 掉帧。

Anim 的整个数据布局以局部性为第一原则。
不是优化。是能不能在 1ms 内完成。
```

---

## 一、时间局部性——同一份数据短期内被反复访问

同一份数据在被访问一次之后，近期内很可能被再次访问。Anim 把"短期内一定会再被访问"的数据全部留在 L1/L2 缓存内。

### 1.1 ESIR 帧——相邻帧只差一个偏差修正量

```
帧 N 的参数：  { ear: 0.4mA, 25Hz, 200μs; neck: 3kPa, 1Hz; wrist: 36.5°C }
帧 N+1 的参数：{ ear: 0.41mA, 25Hz, 200μs; neck: 3kPa, 1Hz; wrist: 36.5°C }

差别只有一个字段的 0.01mA。
但传统做法 → 每帧重新生成整个 ESIR 结构 → 全部字段写一遍 → 全部字段重新 load。

Anim 的做法 → ESIR 帧缓冲区是原地更新的：
    帧 N+1 在帧 N 的同一块内存上直接修改。
    不变的字段（耳后频率、后颈压力、腕部温度）不写、不读、不移动。
    只有耳后电流从 0.4 → 0.41 —— 一个 u8 的增量。

帧缓冲区是环形缓冲区（Ring Buffer）。
FPGA DMA 读指针循环扫描同一块物理内存。
不是每帧新分配 → 每帧的内存地址相同 → L1 cache 永远命中。
```

### 1.2 PBM 系数——一个 Session 内只读一次

```
Pass 6 Personalize 在 Session 启动时读取 PBM。
同一 Session 内——PBM 不变。
所以 PBM 的查询结果缓存在设备的本地 SRAM 或 L2 cache 里。
不是每次 DSIR 生成时重新读 PBM。

后台离线预交织：
    同一个用户的 PBM 在连续几个 Session 之间可能有微小漂移——
    但 Pass 0-2 的结果（FSIR）完全不依赖 PBM。
    FSIR 缓存的时间局部性 = 只要用户不换感受包——同一份 FSIR 被上百个 Session 共用。
```

### 1.3 安全插桩的阈值——帧级稳定

```
Pass 4 预埋的安全插桩在 ESIR 帧级被逐帧检查。
心率阈值、皮电阈值、呼吸阈值——同一个 Session 内不变。

但这些阈值必须在"每 N 帧"被检查一次。
不是在每一帧从主内存重新读阈值——是加载到 FPGA 寄存器后原地循环比较。
一次加载。N 帧复用。寄存器 = 最高时间局部性。
```

---

## 二、空间局部性——相邻数据被一起访问

如果访问了地址 A，那么 A 附近的地址也很快会被访问。Anim 按照"一起被访问的一起存"来布局所有数据结构。

### 2.1 ESIR 帧——按设备分组连续存储

```
错误的布局（字段散列）：
    struct EsirFrame {
        ear_current: u16,      // 地址 0x1000
        neck_pressure: u16,    // 地址 0x2000  ← 不在同一 cache line
        wrist_temp: u16,       // 地址 0x3000
    }
    → FPGA 读一帧 = 三次 cache line 加载 = 三次内存访问。

正确的布局（设备参数连续）：
    耳后设备参数块 [current, freq, pulse_width] 连续 12 bytes → 一个 cache line。
    后颈设备参数块 [pressure, freq, pattern] 连续 12 bytes → 一个 cache line。
    腕部设备参数块 [temp, vibration, duration] 连续 12 bytes → 一个 cache line。

    FPGA 读一个设备 → 一次 cache line 加载 → 全部参数都在里面。
    三个设备 → 三个独立的 cache line → 按设备并行的 DMA 通道分别读。
    不共享 cache line = 无 false sharing。
```

### 2.2 FSIR 混音结构——主旋律 + 点缀连续存储

```
mix {
    main: accomplishment_satisfaction,
    accents: [
        { feeling: exhaustion_relief, ratio: 0.25 },
        { feeling: slight_void,        ratio: 0.15 },
        { feeling: self_assurance,     ratio: 0.10 },
    ]
}

主旋律和点缀在 FSIR 中连续存储——不是指针数组指向零散堆内存。
主旋律头（8 bytes）→ 点缀数组长度（2 bytes）→ 点缀条目连续（每条 16 bytes）。
全部在同一个 cache line 或相邻 cache line 内。
一次加载 → 主旋律 + 全部点缀在 L1 里。
```

### 2.3 Pattern Registry——按感受类别分桶存储

```
人类的 accomplishment_certainty 和犬类的 accomplishment_certainty 是不同的原子。
但它们被查询的模式是——"同一个混音包里的主旋律和点缀几乎总是在一起被查询"。

Pattern Registry 的分桶策略：
    不是按"人类/犬类"分区——虽然逻辑上分物种。
    是按"同一源码包内共现频率"做存储优化。
    accomplishment_satisfaction 和 exhaustion_relief 和 slight_void 和 self_assurance——
    在 FSIR 生成时被一起查询 → 在 Registry 的物理存储上被放在相邻页面。
    一次磁盘页加载 → 全部四个原子的元数据在内存里。
```

---

## 三、CPU 指令流水线——Pass 管线和分支预测的硬件映射

### 3.1 八 Pass 的 CPU 视图

```
传统编译器 Pass：每 Pass 输出到内存 → 下一 Pass 读内存 → 序列化/反序列化。
    → 每个 Pass 边界 = 一次流水线冲刷。
    → 八个 Pass = 八次冲刷 = 分支预测器八次从头来。

Anim 的八 Pass 在 CPU 上不是八个独立进程——是八个函数调用。
Pass 之间的 IR 传递 = 指针传递。不序列化。不写磁盘。
    → CPU 指令流水线不被 Pass 边界打断。
    → 分支预测器在前几个 Pass 训练好的模式——后面几个 Pass 继续用。
```

### 3.2 双流水线的分支预测隔离

```
后台离线预交织（Pass 0-5）：
    源码级。不碰生理数据。不需要实时响应。
    分支预测器在"源码解析"的 branch pattern 上训练。
    → 高度可预测。if 语句几乎全部走同一分支。
    → 分支预测准确率 > 95%。

前台实时交织（Pass 7-8）：
    帧级。生理反馈循环。偏差修正。
    分支预测器在"上一帧偏差方向"上训练。
    → 心率在上升趋势中 → 下一帧大概率继续上升 → 预判。
    → 这和 Anim 的帧级预判器（ADR 004 §分支预测）咬合。
    两个管线用不同的分支预测上下文。互不污染。
```

---

## 四、超流水线——缓存和 DMA 的物理布局

### 4.1 FPGA 一侧——ESIR 帧缓冲的 DMA 布局

```
FPGA 上跑的不是 CPU 指令流水线。是门级逻辑的硬实时并行。

ESIR 帧缓冲区在物理内存上是一块连续的环形缓冲区。
FPGA 的 DMA 引擎按固定步长循环读。
DMA 的读写指针：
    写指针（CPU/交织器写下一帧）和读指针（FPGA 读当前帧）之间至少相差 2 帧。
    距离 = 时间缓冲。突发延迟 ≤ 2ms。
    2ms 内 CPU 必须完成下一帧的计算并写入——否则 DMA 读到旧帧。

环形缓冲区的大小 = 帧大小 × 缓冲深度。
缓冲深度 = 3 帧（当前 + 下一帧 + 安全停止帧预留）。
安全停止帧 = FPGA 在任何时候可以跳到的保底地址。
```

### 4.2 CPU 一侧——后台离线交织的缓存友好布局

```
后台离线预交织在 Feelings-Server 上运行。
Server 不是实时系统——没有 1ms 硬上限。

但 Pass 0-5 的中间数据结构大小可能是 MB 级（完整 AST + FSIR + 元数据）。
不做序列化 → 全部在内存里。
Pass 之间传递智能指针（Rc/Arc）→ 不复制。
FSIR 最终缓存到设备本地磁盘——但 Server 端保留热 FSIR 在内存中。
```

---

## 五、和 Anim 已有架构的精确咬合

```
本文                                Anim-LOCALITY——局部性与超流水线的物理约束
ADR 002: IR 架构                   四层 IR 各自独立——
                                   每层的数据结构按"和谁一起被访问"连续布局。
                                   PSIR 和 FSIR 分离 = 热数据（PSIR）和冷数据（FSIR）不共享 cache line。
ADR 003: Pass 管线                 八个 Pass 不是八个进程——是指针传递的八个函数。
                                   Pass 边界不冲刷流水线。
ADR 004: 双流水线                  热路径（DSIR→ESIR）和冷路径（Pass 0-5）在物理核心上隔离。
                                   不共享 L1/L2。不是逻辑隔离——是物理 cache line 不交叉。
Anim-SAFETY.md                     安全插桩的阈值在 FPGA 寄存器里循环比较。
                                   一次加载。N 帧复用。最高时间局部性 = 最安全 = 零延迟。
```

---

*Cache miss 不是性能下降——是帧超时。0.01mA 的修正量如果在主内存上多拉了一次 fetch——那下一帧就已经来不及了。Anim 的整个数据布局不是为了跑得快——是为了在 1ms 的硬限制里每一帧都在 L1 里完成。时间局部性 = 刚摸过的数据别再从主存拉。空间局部性 = 一起用的东西放在同一行。不是优化哲学。是硬死线。*
