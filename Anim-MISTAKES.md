# Anim 工程错误日志

> 哲学：**犯了两次及以上的错误都应该记录下来。单次错误是偶然，重复错误是认知盲区——必须显式化才能消除。**
>
> 规则：重复次数 ≥ 2 才入册。首次犯→观察；再次犯→记录根因+解法，全团队显式共享。

---

## 目录
- [Rust 语言陷阱](#rust-语言陷阱)
- [Anim 特有陷阱](#anim-特有陷阱)
- [工程流程](#工程流程)

---

## Rust 语言陷阱

> 暂无记录。代码零行，尚未触达 Rust 编码阶段。预埋以下来自 Axon/KubePivot 项目的已知陷阱。

### R01: tokio::Mutex guard 跨 `.await` 持锁死锁

- **来源**: Axon MISTAKES.md R01
- **症状**: `let guard = self.sm.lock().await;` 后在 `.await` 点处编译报错 `Future is not Send`
- **解法**: `enum StateMachine` + `std::mem::replace`
- **重复次数**: 0（预埋）

### R02: anyhow::bail! 未导入直接调用

- **来源**: Axon MISTAKES.md R02
- **症状**: 调用 `bail!(...)` 编译失败
- **解法**: 文件顶部加 `use anyhow::bail;`
- **重复次数**: 0（预埋）

### R03: tokio::spawn JoinHandle 未 await 导致 panic 静默吞

- **来源**: Axon MISTAKES.md R03
- **解法**: `JoinHandle` 必须被 await 或 abort
- **重复次数**: 0（预埋）

### R04: 测试中全局函数变量注入失败

- **来源**: Axon MISTAKES.md R04
- **解法**: trait + `Box<dyn Fn>` 注入
- **重复次数**: 0（预埋）

---

## Anim 特有陷阱

> 暂无记录。代码零行。预埋已知领域知识。

### A01: 把 animi 叫成 animc 或 compiler

- **症状**: 代码、注释、文档中写 `animc` / `compiler` / `编译`
- **根因**: 惯性——传统工具链都是 compiler
- **解法**: animi = Anim Interlinker（交织器）。不是编译器，是编织者。
- **重复次数**: 0（预埋）

### A02: 安全校验放在运行期

- **症状**: 生理阈值检查写在了 ESIR 帧级执行时，而非编译期插桩
- **根因**: 习惯「运行时检查」的通用编程模式
- **解法**: Pass 4（RuntimeGuard）在交织期预埋插桩。运行期只做寄存器值的比较——不做判断。
- **重复次数**: 0（预埋）

---

## 工程流程

> 暂无记录。仓库未初始化。

---

*最后更新: 2026-05-20 | 维护者: Anim 团队*
