# ADR 016: DeviceCapability trait — 设备接口抽象与 DSIR 多态路由

> 状态：草案
> 日期：2026-06-19
> 性质：Pass 7 DeviceMap 重构——从硬编码 ear-only 到多设备 trait 派生 + 降级矩阵 + 离线注入模式
> 对应：ADR 015 扩展 + Feelings docs/architecture/device-architecture.md §十二

---

## 动机

ADR 015 的 DSIR 是硬编码 `{ ear }` 单设备集。ear-only 够 v0.3 闭环，但不够 v0.5+。

物理现实：
- 设备形态不只有耳后一种——护齿、颈部环带、腕部、颞部、雷达、摄像头
- 同一信号维度可以通过多条神经通路送达（Visceral 可走耳支迷走神经，也可走颈段迷走神经）
- 设备可能离线（摘了 / 没电 / 对撞碎了），但系统不应拒绝运行——应降级

三个设计目标：
1. **新设备类型 = 实现一个 trait** — DSIR 核心零改动
2. **按通路路由，不按设备名** — 设备声明"我能走哪些神经"，DSIR 匹配通路
3. **降级不拒绝** — 缺设备 ≠ 拒绝编译 → 折损信号强度 + 标注丢失维度

---

## 决策

### 一、核心 trait：DeviceCapability

```rust
/// 设备声明的神经通路。
///
/// 一个设备可能接入多条通路，每条通路对应不同的信号注入质量和物理约束。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeuralPathway {
    /// 迷走神经耳支（Arnold 神经）— 耳后设备
    VnsEarBranch,
    /// 迷走神经颈段 — 颈部环带
    VnsCervicalBranch,
    /// 三叉神经 — 口腔护齿
    Trigeminal,
    /// C 类触觉纤维 — 腕部设备
    CtFiber,
    /// 颞叶皮层 — 太阳穴设备
    CorticalTemporal,
    /// 脊髓本体感传导 — 后颈设备
    SpinalCervical,
    /// 听觉通路（骨传导 / 气传导）
    Cochlear,
    /// 外周混合神经 — wrist/peripheral
    PeripheralMixed,
}

/// 设备物理形态——决定在极端环境下的存续能力。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFactor {
    /// 耳后吸附式 — 无对抗场景标准形态
    EarClip,
    /// 颈部环带 — 柔性 PCB + 硅胶
    Collar,
    /// 口腔护齿 — 三叉神经通路
    Mouthguard,
    /// 腕部表带 — 腕表 / 手环
    WristBand,
    /// 太阳穴贴片 — 颞叶区
    TemplePatch,
    /// 非接触雷达 — 3-5m 范围
    RemoteRadar,
    /// 夹耳式独立摄像头
    ClipCamera,
}

/// 设备对指定信号维度的覆盖能力。
#[derive(Debug, Clone)]
pub struct PathwayQuality {
    pub pathway: NeuralPathway,
    /// 信号注入质量 (0.0-1.0)
    pub inject_quality: f64,
    /// 信号采集质量 (0.0-1.0)
    pub collect_quality: f64,
    /// 该通路在当前设备上的最大支持强度 (硬件上限)
    pub max_intensity: u32,
}

/// 设备能力接口——trait。
///
/// 新设备类型 = `impl DeviceCapability for MyDevice`
/// DSIR 路由逻辑只依赖此 trait，不依赖具体设备类型。
pub trait DeviceCapability {
    /// 设备唯一标识。
    fn device_id(&self) -> &str;

    /// 该设备接入的神经通路及质量参数。
    fn pathways(&self) -> &[PathwayQuality];

    /// 物理形态。
    fn form_factor(&self) -> FormFactor;

    /// 设备当前是否在线。
    fn online(&self) -> bool;

    /// 查询指定通路的注入质量。未接入 → 0.0。
    fn inject_quality(&self, p: NeuralPathway) -> f64 {
        self.pathways()
            .iter()
            .find(|q| q.pathway == p)
            .map(|q| q.inject_quality)
            .unwrap_or(0.0)
    }

    /// 查询指定通路的采集质量。
    fn collect_quality(&self, p: NeuralPathway) -> f64 {
        self.pathways()
            .iter()
            .find(|q| q.pathway == p)
            .map(|q| q.collect_quality)
            .unwrap_or(0.0)
    }
}
```

### 二、PbmDimension → NeuralPathway 映射

每个感受维度有首选通路和备用通路：

```rust
fn primary_pathway(dim: PbmDimension) -> NeuralPathway {
    match dim {
        PbmDimension::Visceral   => NeuralPathway::VnsEarBranch,   // 迷走神经分支
        PbmDimension::Emotional  => NeuralPathway::VnsEarBranch,   // 同通路，岛叶投射
        PbmDimension::Tactile    => NeuralPathway::CtFiber,        // CT 纤维
        PbmDimension::Auditory   => NeuralPathway::Cochlear,       // 听觉通路
    }
}

fn fallback_pathways(dim: PbmDimension) -> &'static [NeuralPathway] {
    match dim {
        PbmDimension::Visceral  => &[NeuralPathway::VnsCervicalBranch, NeuralPathway::Trigeminal],
        PbmDimension::Emotional => &[NeuralPathway::CorticalTemporal],
        PbmDimension::Tactile   => &[NeuralPathway::PeripheralMixed],
        PbmDimension::Auditory  => &[], // 听觉通路无备用——缺 ear 则静默
    }
}
```

### 三、DSIR 多态路由

```rust
/// 将 PSIR 分配到在线设备集。
///
/// 对每个感受维度，遍历在线设备，按通路匹配分配。
/// 首选通路不可用 → 尝试备用通路 → 信号质量折损。
pub fn device_map(
    psir: &PsirDoc,
    devices: &[Box<dyn DeviceCapability>],
) -> Result<DsirDoc, AnimiError> {
    let online: Vec<&Box<dyn DeviceCapability>> = devices.iter().filter(|d| d.online()).collect();

    if online.is_empty() {
        // 零设备——检查是否有注入模式 (InjectedMode)
        return injected_mode_route(psir);
    }

    let mut assignments = Vec::new();
    let mut missed_dimensions = Vec::new();
    let mut degradation = Vec::new();

    for dim in DIM_ORDER {
        let primary = primary_pathway(dim);
        let fallbacks = fallback_pathways(dim);

        if let Some(dev) = find_device(&online, primary) {
            let quality = dev.inject_quality(primary);
            assignments.push(DeviceAssignment {
                device_id: dev.device_id().into(),
                dimension: dim,
                pathway: primary,
                quality,
                intensity_scale: quality, // 信号强度按质量折损
            });
        } else if let Some(dev) = find_first_device(&online, fallbacks) {
            let pathway = fallbacks.iter().find(|p| dev.inject_quality(**p) > 0.0).copied().unwrap();
            let quality = dev.inject_quality(pathway) * 0.8; // 备用通路额外折损 20%
            degradation.push(format!("{}: {} -> {} (q={:.2})", dim, primary, pathway, quality));
            assignments.push(DeviceAssignment {
                device_id: dev.device_id().into(),
                dimension: dim,
                pathway,
                quality,
                intensity_scale: quality,
            });
        } else {
            missed_dimensions.push(dim); // 该维度无在线设备覆盖
        }
    }

    Ok(DsirDoc {
        assignments,
        missed_dimensions,
        degradation,
        device_count: online.len(),
        injected_mode: false,
    })
}

fn find_device<'a>(
    devices: &[&'a Box<dyn DeviceCapability>],
    pathway: NeuralPathway,
) -> Option<&'a Box<dyn DeviceCapability>> {
    devices.iter().find(|d| d.inject_quality(pathway) > 0.0).copied()
}

fn find_first_device<'a>(
    devices: &[&'a Box<dyn DeviceCapability>],
    pathways: &[NeuralPathway],
) -> Option<&'a Box<dyn DeviceCapability>> {
    for p in pathways {
        if let Some(d) = find_device(devices, *p) {
            return Some(d);
        }
    }
    None
}
```

### 四、离线注入模式 (InjectedMode)

对抗性运动场景——赛前注入，赛中不戴设备。

```rust
/// 零在线设备但 PSIR 已预设 → 注入模式路由
///
/// 原理：DSIR 记录注入时间戳 + 每种激素的半衰期窗口。
/// ESIR 据此判断"当前帧是否仍在覆盖窗口内"。
fn injected_mode_route(psir: &PsirDoc) -> Result<DsirDoc, AnimiError> {
    if psir.injected_at.is_none() {
        return Err(AnimiError::InternalError { /* 无设备 + 无注入 = 拒绝 */ });
    }

    // 注入模式下：所有维度都有虚拟分配，质量=1.0
    // 实际效果由激素半衰期窗口在 ESIR 阶段判断
    let assignments = DIM_ORDER.iter().map(|dim| {
        DeviceAssignment {
            device_id: "injected".into(),
            dimension: *dim,
            pathway: primary_pathway(*dim),
            quality: 1.0,
            intensity_scale: 1.0,
        }
    }).collect();

    Ok(DsirDoc {
        assignments,
        missed_dimensions: vec![],
        degradation: vec![],
        device_count: 0,
        injected_mode: true,
    })
}
```

到了 ESIR 阶段（Pass 8），`injected_mode=true` 的 DSIR 会按照注入时间戳 + 激素半衰期表生成帧级别窗口标记：

```
帧 0-1800000 (0-30min):  注入窗口内——全量信号输出
帧 1800000-5400000 (30-90min): 皮质醇半衰期尾部——信号逐渐衰减
帧 > 5400000: 超窗——降级为仅有硬件支持的维度（若有在线设备）/ 或仅留监控
```

### 五、设备实现示例

```rust
pub struct EarDevice {
    id: String,
    is_online: bool,
}

impl DeviceCapability for EarDevice {
    fn device_id(&self) -> &str { &self.id }
    fn form_factor(&self) -> FormFactor { FormFactor::EarClip }
    fn online(&self) -> bool { self.is_online }

    fn pathways(&self) -> &[PathwayQuality] {
        static P: &[PathwayQuality] = &[
            PathwayQuality { pathway: NeuralPathway::VnsEarBranch,  inject_quality: 0.9, collect_quality: 0.85, max_intensity: 100 },
            PathwayQuality { pathway: NeuralPathway::Cochlear,      inject_quality: 0.95, collect_quality: 0.0, max_intensity: 100 },
        ];
        P
    }
}

pub struct MouthguardDevice {
    id: String,
    is_online: bool,
}

impl DeviceCapability for MouthguardDevice {
    fn device_id(&self) -> &str { &self.id }
    fn form_factor(&self) -> FormFactor { FormFactor::Mouthguard }
    fn online(&self) -> bool { self.is_online }

    fn pathways(&self) -> &[PathwayQuality] {
        static P: &[PathwayQuality] = &[
            PathwayQuality { pathway: NeuralPathway::Trigeminal, inject_quality: 0.7, collect_quality: 0.5, max_intensity: 80 },
        ];
        P
    }
}

// Collar, WristBand, TemplePatch, RemoteRadar, ClipCamera — 等同模式
// 每个 impl DeviceCapability + 返回自己的 pathways() 表
```

---

## 和已有文档的咬合

```
本文                                          ADR 016 — DeviceCapability trait + DSIR 多态路由
docs/design/015-device-map-codegen.md         原 ADR 015 — v0.3 ear-only 骨架，本文为扩展
docs/design/002-ir-architecture.md            四层 IR — DSIR/ESIR 定义，需新增注入模式字段
docs/design/009-math-and-constraints.md        §十.9 神经内分泌约束——注入模式依据
Feelings/docs/architecture/device-architecture.md  §十二 对抗性运动设备适配——方案三 赛前注入
src/dsir.rs + src/device_map.rs               当前硬编码实现——本文为重构目标
```

---

## 不做的事

- **不做自定义信号调制函数** — 设备只声明通路质量，不定义"怎么调信号形状"。形状是 CodeGen 的事。
- **不做设备间总线同步** — Feelings-OS busd 的事。Anim 只分配信号到设备，不管理时钟。
- **不做运行时热插拔** — v0.7+ 的事。当前假设 Session 启动时设备集固定不变。

---

## 版本管理

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1 | 2026-06-19 | 初始草案——DeviceCapability trait + NeuralPathway 枚举 + 多态路由 + InjectedMode |
