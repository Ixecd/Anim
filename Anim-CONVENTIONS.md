# Anim 开发规范

> 性质：强制约定，不是建议
> 范围：Anim 仓库所有 Rust 代码、文档、提交
> 更新：随项目演进持续修订

---

## 一、空值处理

Rust 没有 null。Anim 不使用任何形式的 null 替代品。

```
✅ Option<T>   — 感受原子可能不存在（Pattern Registry 查询）
✅ Result<T,E> — 交织阶段可能失败
❌ "" / -1 / nullptr  — 不存在
```

### 链式优先

```rust
// ✅
let atom = registry
    .find(name)
    .ok_or(AnimiError::AtomNotFound(name.to_string()))?;

// ❌ match 套 match
let atom = match registry.find(name) {
    Some(a) => a,
    None => return Err(...),
};
```

---

## 二、错误处理

```rust
// ✅ thiserror 定义错误类型
#[derive(Error, Debug)]
enum AnimiError {
    #[error("feeling atom '{0}' not found")]
    AtomNotFound(String),
    #[error("intensity {0} exceeds cap {1}")]
    IntensityExceedsCap(u8, u8),
}

// ❌ 不用 String 当错误
fn foo() -> Result<(), String>;
```

---

## 三、命名

```
snake_case     — 文件名、模块名
CamelCase      — struct / enum / trait
snake_case     — fn / let
SCREAMING      — const
```

不缩写。`feeling_atom` 不是 `fa`，`personalize` 不是 `pers`。

---

## 四、术语铁律

Anim 仓库内强制使用以下术语，禁止混用：

```
✅ animi       ❌ animc
✅ 交织器      ❌ 编译器
✅ 交织        ❌ 编译
✅ 感受原子    ❌ 变量
✅ 混音结构    ❌ 函数/方法
✅ 点缀        ❌ 参数
✅ session     ❌ 执行/运行
```

---

## 五、提交格式

```
feat: xxx      — 新功能
fix: xxx       — 修 bug
docs: xxx      — 文档
refactor: xxx  — 重构（行为不变）
chore: xxx     — 杂项（依赖更新、格式化）
test: xxx      — 测试
```

---

## 六、禁止事项

```
❌ unsafe 代码（除非有性能基准证明必要 + 独立审查）
❌ unwrap() 在非测试/非断言代码中
❌ 硬编码生理阈值
❌ 把「编译器」写进代码——写「交织器」
❌ 把 animc 写进代码——写 animi
❌ println! / eprintln!（走 tracing）
❌ .clone() 满天飞（先想借用）
```

---

*规范不是镣铐，是让后续的人不用猜你当时在想什么。*
