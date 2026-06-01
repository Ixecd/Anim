# Anim — 如何让 01 呼吸

> Anim 不是编译器。Anim 是交织器。

把感受结构的描述（`.anim` 源码）织成神经信号序列（ESIR 帧级指令）。不是"把代码翻译成机器码"——是**把多股独立的信息流织成一根能被岛叶读懂的信号绳。**

---

## 是什么

```
.anim 源码（感受结构的声明式描述）
    ↓  animi 交织器
FSIR JSON（感受结构中间表示——与人无关，与设备无关）
    ↓  PBM 左乘（个人基线矩阵——永不离设备）
ESIR 帧（1ms/帧——直接喂给 FPGA）
    ↓  Feelings-OS → 耳后/后颈/腕部/颞部
感受
```

## 不是什么

- 不是编译器——Anim 不生成可执行文件。Anim 生成信号序列。
- 不是 AI——Anim 不预测。Anim 交织规则是确定的。不确定的部分在 PBM 里——PBM 不在 Anim 里。
- 不是 IDE——Anim 的核心是命令行工具。可视化编辑器是 v0.2.5 的事。

## 项目状态

```
v1.0 已打 tag（自举前唯一 tag——自举完成才换 v2.0）
63 commits，51 单元测试
九 Pass 交织管线——Pass 0-5 实现，Pass 6-8 待 v0.3
最小闭环已交付——.anim 源码 → FSIR JSON
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

设计文档在 [docs/design/](docs/design/)——ADR 001-007。

---

*KubePivot 是承诺。Feelings 是立场。Anim 让 01 呼吸。*