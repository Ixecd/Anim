# Anim — 如何让 01 呼吸

> Anim 不是编译器。Anim 是交织器。

把感受结构的描述（`.anim` 源码）织成神经信号序列（ESIR 帧级指令）。不是"把代码翻译成机器码"——是**把多股独立的信息流织成一根能被岛叶读懂的信号绳。**

---

## 是什么

Feelings 既可以刺激神经——也可以安抚神经——也可以麻痹神经。Anim 就是这条三重管线的编译器。同一条 ESIR 帧序列——根据 Core 的强度调度——进入等控器的激活/安抚/麻痹方向。

```
.anim 源码（感受结构的声明式描述）
    ↓  animi 交织器
FSIR JSON（感受结构中间表示——与人无关，与设备无关）
    ↓  PBM 左乘（个人基线矩阵——永不离设备）
ESIR 帧（1ms/帧——直接喂给 FPGA）
    ↓  Feelings-OS → 耳后/后颈/腕部/颞部
感受
```

## 为什么

Anim 的设计动机不是"控制神经"。是——长期专注一件事——第 1 天充实——第 100 天麻木——等控器在满足太久之后——阈值上移——忘了什么是满足。Anim 存在的意义——不是给你更强的信号——是帮你找回第 1 天的那个强度——通过节律——通过恢复——通过麻痹让该关的关——通过刺激让该开的开。

## 不是什么

- 不是编译器——Anim 不生成可执行文件。Anim 生成信号序列。
- 不是 AI——Anim 不预测。Anim 交织规则是确定的。不确定的部分在 PBM 里——PBM 不在 Anim 里。
- 不是 IDE——Anim 的核心是命令行工具。可视化编辑器是 v0.2.5 的事。

### Anim 的边界——只管编译

```
Anim 做的事                              Anim 不做的事
──────────                              ───────────

.anim 源码 → ESIR 帧                         不管硬件——不驱动设备、不管 PLL 时钟、不管总线。
  九 Pass 管线——前端+安全+交织                    Feelings-OS timerd/busd 管。
  强度上限 / 漏桶 / 恢复帧 / 冷启动 / 阻尼矩阵
                                              不管 PBM——不计算个人基线、不维护 anchors、
安全防线——Pass 2/3/3b/4                      不追踪 R（锚点置信度）、不管理 sessions。
  单帧 cap + 时域积分 + 信号变异度标记              Feelings-Core PBM 管。
  参数由 Core 提供——Anim 只做硬上限
                                              不管调度——不仲裁 P0/P1/P1b 优先级。
                                               Feelings-OS schedulerd 管。
```

## 项目状态

```
v1.0 已打 tag（自举前唯一 tag——自举完成才换 v2.0）
108 单元测试，make dev 全绿
九 Pass 交织管线——Pass 0-6 实现，Pass 7-8 待 v0.3
P0 8/12（3 项 ADR 已定稿 / 4 项设计未完）
P1 4/20  P2 2/2
12 ADR（001-012）
```

## 快速开始

```bash
make dev                    # fmt + clippy + test + build
cargo run -- eg/calm.anim  # 产出第一份 FSIR JSON
```

## 入口文档

按顺序读：

1. [README.md](docs/meta/Anim-README.md) — 项目介绍
2. [PHILOSOPHY.md](docs/meta/Anim-PHILOSOPHY.md) — 产品哲学
3. [HANDOFF.md](docs/meta/Anim-HANDOFF.md) — 技术全景
4. [ROADMAP.md](docs/meta/Anim-ROADMAP.md) — 路线图
5. [FORGET.md](docs/meta/Anim-FORGET.md) — 待办清单（P0/P1/P2）
6. [GLOSSARY.md](docs/meta/Anim-GLOSSARY.md) — 术语速查

设计文档在 [docs/design/](docs/design/)——ADR 001-012。

---

*KubePivot 是承诺。Feelings 是立场。Anim 让 01 呼吸。*