# HANDOFF — Anim v1.0

> 编写日期：2026-06-04
> Tag: v1.0（已打 tag——自举前唯一 tag。自举完成前不再打任何 tag）
> Total commits: 111
> Co-Authored-By: DeepSeek

---

## 一、项目定位

**Anim** 是 Feelings 生态的交织语言工具链。不是编译器——是交织器（Interlinker）。

把感受结构的声明式描述（`.anim` 源码）织成神经信号序列（FSIR JSON → 最终 ESIR 帧级指令）。

**核心论断**：
- animi = Anim Interlinker。不是"编译"。是"交织"——多股独立信息流在同一个时空坐标相遇。
- 安全在交织期——九 Pass 四层 IR。Pass 0-4 在交织期拒绝不安全信号。不等到运行期。
- 包名 ≠ 主旋律原子。三层结构：基本感受大类 → 细分变体 → 点缀修饰。

**技术选型**：
- **Rust** — 零成本抽象，所有权模型天然契合"丝/经线/纬线"的不可变语义
- **serde + serde_json** — FSIR 序列化。JSON 格式 = 哈希确定性 + Git diff 可读 + 人机边界
- **chrono** — RFC 3339 时间戳
- **sha2 + hex** — 源码 SHA-256 + Registry 哈希。SPL 锚定就绪
- **零 unsafe** — 所有代码 safe Rust

---

## 二、现在能做什么

### 用户视角

```bash
animi calm.anim                    # 交织 → calm.json
animi calm.anim registry.json      # 使用外部 Registry
animi calm.anim --cap 45           # 强度上限 45
animi --help                       # 帮助
animi --version                    # 版本
```

### 开发者视角

```
make dev    # fmt + clippy + test + build
make ci     # fmt-check + clippy + test + build
```

**交织管线**：
```
.anim 源码
  ↓ Pass 0a 词法（lexer.rs）—— ASCII-only，CRLF，BOM skip
  ↓ Pass 0b 语法（parser.rs）—— 递归下降，顺序无关，重复检测
  ↓ Pass 1  类型检查（typeck.rs）—— Registry 校验 + max_ratio + 编辑距离建议
  ↓ Pass 2  静态安全（rule.rs）—— 全局上限 100，abrupt_stop≤20，沙箱限制
  ↓ Pass 3  用户安全（safety.rs）—— 桩 + scale_intensity（v1.2 接入用户档案）
  ↓ Pass 4  运行期插桩（guard.rs）—— 桩（v1.2 接入 ESIR）
  ↓ Pass 5  FSIR 生成（fsir.rs）—— AST → FsirDoc → JSON
  ↓
fsir.json
```

**FSIR JSON 结构**：
```json
{
  "meta": {
    "animi_version": "0.1.0",
    "compiled_at": "2026-06-01T...",
    "source_hash": "sha256...",
    "pattern_registry_hash": "sha256...",
    "safety_rules_version": null
  },
  "name": "calm",
  "mix": { "main": "calm_meditative", "accents": [...] },
  "shape": { "name": "gradual_rise_fall" },
  "intensity": { "min": 15, "max": 45 },
  "smoothing": {
    "steps": [
      { "multiplier": 1.0 },
      { "multiplier": 0.6 },
      { "multiplier": 0.3 },
      { "multiplier": 0.1 },
      { "multiplier": 0.0 }
    ]
  }
}
```

**测试**：91 单测全绿。覆盖 lexer / parser / typeck / rule / safety / guard / fsir / pbm / oi / error / log。

---

## 三、项目结构

```
Anim/
├── src/                    # 14 模块 + build.rs
│   ├── main.rs             CLI 入口（--help/--version/--cap/--log-level）
│   ├── lib.rs              库入口
│   ├── ast.rs              AST 类型（FeelingSource/Mix/Accent/Shape/Intensity）
│   ├── lexer.rs            Pass 0a 词法分析
│   ├── parser.rs           Pass 0b 语法分析
│   ├── typeck.rs           Pass 1 类型检查
│   ├── rule.rs             Pass 2 静态安全规则
│   ├── safety.rs           Pass 3 用户安全 + scale_intensity
│   ├── guard.rs            Pass 4 运行期插桩 + OiSmoothing 衰减曲线
│   ├── fsir.rs             Pass 5 FSIR 生成 + JSON/Postcard 双序列化
│   ├── pbm.rs              PBM 地基——SessionLabel/DataConfidence/ColdStartGuard/DampingMatrix
│   ├── registry.rs         Pattern Registry（外部 JSON + 内建 fallback）
│   ├── oi.rs               oi! + oi_err! 宏
│   ├── log.rs              日志系统——A_debug!/A_info!/A_warn!/A_error! + 四级过滤
│   ├── error.rs            AnimiError 6 变体
│   └── build.rs            错误码文档自动生成
├── eg/                     示例 .anim 文件（4 个）+ registry.json
├── txt/                    源码的 .txt 副本
├── docs/
│   ├── design/             ADR 001-007 + error-codes.md
│   └── meta/               项目管理文档（19 个）
├── commits/                commit 消息存档
├── snapshots/              历史快照
├── Cargo.toml / Makefile
└── README.md
```

---

## 四、关键设计决策

| 决策 | 内容 | 位置 |
|------|------|------|
| 交织器，不是编译器 | animi = Anim Interlinker | GLOSSARY.md |
| 九 Pass 四层 IR | Pass 0-8，FSIR→PSIR→DSIR→ESIR | ADR 002/003 |
| FSIR = JSON | 哈希确定性 + Git diff | ADR 002 |
| 强度 0-100 默认刻度 | u32 底层，整数一拍 FPGA | FORGET v0.1.12 |
| 三层感受结构 | 包名→主旋律变体→点缀修饰 | FORGET D01 |
| ASCII-only 词法器 | is_ascii_* 全系列，BOM skip | lexer.rs |
| Registry 外部化 | JSON 可扩展，硬编码 fallback | registry.rs |
| 错误码自动生成 | build.rs → docs/error-codes.md | ADR 001 |
| 源码 + Registry 双哈希 | SHA-256，SPL 锚定就绪 | main.rs |
| 强度缩放 | ANIMI_USER_CAP 或 --cap 参数 | safety.rs |

---

## 五、FORGET 状态

```
P0: 5/9   — 安全盲区 4（三层防线不全/Session突变/沙箱混合/创伤阈值）+ 架构 5 全清
P1: 3/14  — IR管线3+设备3+语法2+缓存1+可观测1+代码功能3（错误文件名✅/日志✅/错误测试✅）
P2: 0/0   — 全清
```

- v1.0 tag 已打——自举完成前不再打任何 tag
- 下一阶段关键入口：P0 安全盲区补齐（ADR 009）→ Pass 6-8（PSIR/DSIR/ESIR）

---

## 六、下一窗口

- P0 安全盲区补齐（ADR 009）——三层防线/状态突变/沙箱混合/创伤阈值
- Pass 6-8（Personalize/DeviceMap/CodeGen）—— v0.3
- PBM 四维差异化冷启动系数 + sigmoidal 收敛
- 阶段零数据采集（Polar H10 + Empatica E4）——PBM 不能凭空写数值
- 可视化编辑器（ZENO 节点图框架）—— v0.2.5
- 宏系统——ADR 005 已定稿，macro_rules! 解析器+展开器未实现

---

*关联: [Anim-README.md](Anim-README.md) — 项目入口 | [Anim-ROADMAP.md](Anim-ROADMAP.md) — 路线图 | [Anim-FORGET.md](Anim-FORGET.md) — 待办清单*