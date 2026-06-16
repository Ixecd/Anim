# ADR 015: Pass 7 DeviceMap + Pass 8 CodeGen — FSIR → ESIR 全链路闭合

> 状态：定稿
> 日期：2026-06-15
> 性质：Anim v0.3 核心交付——交织管线从 FSIR 通到 ESIR 帧级输出
> 对应：ADR 002 §DSIR/§ESIR + ADR 003 §Pass 7/§Pass 8 + FORGET P1 #10/#11/#12

---

## 动机

当前管线断在 Pass 6。Pass 6 (Personalize) 产出 PSIR——经过 PBM 个人校准的感受结构。但之后没有任何代码把 PSIR 推入物理设备层。FSIR → ESIR 链路残缺。

v0.3 的核心交付：**FSIR → PSIR → DSIR → ESIR 完整链路跑通。** 不是 FPGA 对接——是 ESIR 帧级二进制输出，一次编译生成一份 `.esir` 文件。FPGA 对接是 v0.5 的事。

---

## 决策

### Pass 7: DeviceMap — PSIR × DeviceSet → DSIR

```
源: PSIR（个人校准后的感受结构）
织入: 设备集合（哪些设备当前已连接）
输出: DSIR（设备感知的感受结构——每个设备该发什么样的信号轮廓）

核心逻辑（v0.3 骨架）：
    1. 读设备清单。当前阶段只有一套硬编码设备集：
       { ear } —— 迷走神经耳支 + 心率采集。
       完整设备集（{ ear, wrist, neck, temple }）留 v0.4+。

    2. 按原子维度分配设备：
       Visceral  → ear（迷走神经 + 心率）
       Emotional → ear（迷走神经 + 心率）
       Tactile   → （腕部，v0.3 暂无此设备——标记为 outline 降级）
       Auditory  → ear（骨传导通路）

    3. 设备缺失降级——outline 模式：
       缺 wrist（触觉）→ 该维度的主旋律和点缀配比减半 + 标记 degraded_by_device。
       缺 neck（本体感受）→ 形状的 peak 段降级为 steady 段。
       缺 temple（EEG）→ 认知维度全禁——不做任何 EEG 相关输出。
       缺 ear（迷走神经）→ **这个是硬线——ear 缺失 = 整个 Session 不可启动。** 迷走神经耳支是唯一的安全锚定通路。
       v0.3：ear 硬线直接拒绝编译。wrist/neck/temple 降级标记但继续。

    4. 设备频率分配：
       ear:   1ms/帧（迷走神经刺激 + 骨传导）
       wrist: 8ms/帧（BLE 抖动上限）
       neck:  2ms/帧
       temple: 4ms/帧
       当前 v0.3 只有 ear → 帧率固定 1ms/帧。
```

### Pass 8: CodeGen — DSIR → ESIR

```
源: DSIR（设备感知的感受结构）
织入: 帧时钟、形状包络展开、oi 安全插桩、闭环预修正（v0.5+）
输出: ESIR 帧序列二进制（.esir 文件）

核心逻辑（v0.3 骨架）：
    1. 形状 → 时间轴展开：
       gradual_rise_fall     → 缓升缓降。rise 段 N 帧，sustain 段 M 帧，fall 段 K 帧。
       sharp_peak            → 尖峰。短 rise + 瞬时 peak + 短 fall。
       steady                → 恒稳。恒定强度，持续 N 帧。
       slow_decay            → 慢降。从 peak 缓降至零。
       wave                  → 波浪。几轮 rise-fall 的叠加。
       abrupt_stop           → 瞬停。强度直达零。仅在强度 ≤ 20 且 oi_smoothing hook 启用时生成。
       展开参数从 config.sigmoidal / config.oi_smoothing 读取。

    2. 帧结构（每帧）：
       struct ESIRFrame {
           frame_id:     u32,        // 帧序号（从 1 开始）
           timestamp_us: u64,        // 微秒时间戳（Session 启动时归零）
           device:       DeviceId,   // 目标设备
           intensity:    u32,        // 当前帧强度（0-100）
           frequency_hz: u16,        // 刺激频率（0-500Hz）
           pulse_width_us: u16,      // 脉宽（0-500μs）
           flags:        u8,         // 位标志：bit0=安全帧 bit1=恢复帧 bit2=硬截断
       }

    3. 帧序列生成：
       遍历 DSIR 的 shape → 按 shape 类型展开为帧序列。
       每帧插入 oi_smoothing 衰减系数。
       首帧 + 尾帧嵌入安全校验标记。
       尾帧强制归零（保底包）。

    4. 输出格式：
       Postcard 二进制序列——和 FSIR 相同的序列化格式。
       每帧 16 字节固定宽度。
       文件后缀 `.esir`。
       验证：反序列化回 struct → 字段完全一致。
```

---

## 不做的事

- **不做 FPGA 协议对接** — v0.5 的事。当前 ESIR 输出是 Postcard 二进制文件，不连接硬件。
- **不做多设备时钟同步** — Feelings-OS timerd 的事。Anim 只输出帧级指令。
- **不做实时 Session 闭环** — v0.5 的事。当前是离线编译：`.anim → .esir`。
- **不做形状的自定义 math 模板** — 当前只支持预定义的 6 种 shape。自定义波形模板是 v0.4+ 的事。

---

## 和已有文档的咬合

```
本文                                          ADR 015 — Pass 7 + Pass 8 骨架设计
docs/design/002-ir-architecture.md             四层 IR — DSIR/ESIR 定义
docs/design/003-pass-pipeline.md               九 Pass 管线 — Pass 7/Pass 8 功能
docs/meta/Anim-FORGET.md                       P1 #10/#11/#12 — 四层 IR / Pass 7-8 / 冷启动
src/psir.rs                                    PSIR — Pass 7 的输入
src/personalize.rs                             Pass 6 — 产出 PSIR
configs/default.yaml                           shape 参数 / oi_smoothing 序列
```
