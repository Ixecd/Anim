# Anim

> **从 01 到感受的交织语言** — 不是编译器，是编织者。

---

## 这是什么

Anim 是 Feelings 生态的交织语言。它的工具叫 `animi`——Anim Interlinker（交织器）。

传统编译器做的事：源码 → 词法 → 语法 → IR → 目标码。单向。一次性。

animi 做的事：多股独立流——感受语义、个人基线、设备约束、生理反馈、安全边界——织成一条连续的信号绳。每一层都在把一股新的东西织进去。

```
.anim 源码（感受结构的声明式描述）
    → animi（交织器）
    → FSIR → PSIR → DSIR → ESIR（四层 IR）
    → 帧级指令 → FPGA/设备 → 神经通路 → 感受
```

验证标准：传统编译器 = 程序不 crash。animi = 身体信了，深睡时长涨了。

---

## 和编译器的区别

| 编译器 | 交织器 |
|--------|--------|
| 把 A 翻译成 B | 把多股线织成一股绳 |
| 单向，一次性 | 闭环，每帧微调 |
| 目标：机器码 | 目标：神经信号序列 |
| 验证：正确性 | 验证：身体信了 |

---

## 核心设计

### 八 Pass 三层安全

```
Pass 0: LexParse        词法语法解析
Pass 1: TypeCheck        感受类型校验
Pass 2: StaticSafety     静态安全规则（不可跳过）
Pass 3: UserStateSafety  用户状态安全（创伤分型交叉判定）
Pass 4: RuntimeGuard     运行期插桩生成（按感受类型差异化阈值）
Pass 5: FSIRGen          感受结构 IR 生成
Pass 6: Personalize      个人基线适配（四维差异化冷启动）
Pass 7: DeviceMap        设备信号分配（算力感知编译）
Pass 8: CodeGen          帧级执行指令生成
```

### 四层 IR

```
FSIR    感受结构中间表示        感受的抽象结构图（与人无关，与设备无关）
PSIR    个人适配信号中间表示    个人基线矩阵左乘后的信号参数
DSIR    设备信号分配中间表示    多设备协同信号序列
ESIR    执行信号中间表示        帧级指令（1ms/帧），带闭环反馈指针
```

### 双流水线

```
前台实时浅流水线    Session 运行中，FPGA 硬实时，DSIR→ESIR→闭环（1ms/帧）
后台离线预编译流水线  Session 不运行时，Server 端全量编译并缓存 FSIR
```

---

## 自举三步

```
v0.x    Rust 寄居——用 Rust 写第一个 animi，借壳跑魂
v1.0    自举——用 v0.x 的 animi 编译 Anim 写的 animi 源码
        向下兼容 v0.x 生成的 FSIR 产物
v2.0    从 01 裸奔——Anim 直接管理自己的内存、调度、I/O
        运行在 Feelings 设备上，不经过 OS
```

---

## 类型系统

双层感受原子体系：核心原子（全开放，Pattern Registry 验证）+ 沙盒原子（≤30 强度，创作者本人使用）。

声明式动态注解（不内嵌 if/for）：`@auto_reduce_on` / `@auto_hold_on` / `@auto_release_on` / `@intensity_ceiling` / `@recovery_required`。

---

## License

MIT License.

> Made by qc (Ixecd), for Feelings.
