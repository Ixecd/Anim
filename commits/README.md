## 提交格式

^((Merge (branch|pull request).*?)|((revert: )?(feat|fix|perf|style|build|refactor|test|ci|docs|chore)(([a-zA-Z0-9-]+))?!?: .+[^.]))$

## 安全模板
<type>(<scope>): <description>

### type（必须是这 10 个之一）
- `feat`     新功能
- `fix`      修 bug
- `perf`     性能优化
- `style`    代码格式
- `build`    构建系统（Cargo.toml、Makefile）
- `refactor` 重构
- `test`     测试
- `ci`       CI/CD
- `docs`     文档（含 ADR、FORGET、ROADMAP 更新）
- `chore`    杂项

### scope（可选，[a-zA-Z0-9-] only）
- 例：`(lexer)`、`(parser)`、`(typeck)`、`(fsir)`、`(pass4)`

### description（必填）
- **不能以 `.` 结尾**
- subject 总长度建议 < 80 字符
- **强烈建议全 ASCII**（避免 hook 编码问题）
- 简洁、动词开头（如 "add..." / "fix..." / "remove..."）

## 推荐模板（Anim 项目实际格式）

feat: add tokenizer for .anim source

- Pass 0a 词法分析入口
- 支持关键字、感受原子名、强度区间、shape 名称
- 非法字符 → oi! 编译期拒绝

fix: resolve mix block indent edge case

- 嵌套 mix 块的缩进误差从 2 空格放宽到不限制
- 缩进计算改用相对偏移而非绝对列号

docs: update FSIR binary layout spec

- ADR 008 定稿——FlatBuffers 裁剪版
- Go 侧输出 → Rust 侧 mmap 零拷贝验证方案

test: cover unregistered feeling atom error

- typeck 拒绝未注册感受原子
- 错误信息包含候选项（编辑距离 < 3 的原子名）

## body 规则
- 每行 ≤ 1000 字符
- 中文 OK
- 不限制内容格式

---

## Anim 项目 scope 速览

| scope | 对应源文件 | 说明 |
|-------|----------|------|
| lexer | src/lexer.rs | Pass 0a 词法分析 |
| parser | src/parser.rs | Pass 0b 语法分析 |
| typeck | src/typeck.rs | Pass 1 类型检查 |
| rule | src/rule.rs | Pass 2 通用安全规则 |
| safety | src/safety.rs | Pass 3 用户安全 |
| guard | src/guard.rs | Pass 4 运行期插桩 |
| fsir | src/fsir.rs | Pass 5 FSIR 生成 |
| pbm | src/pbm.rs | Pass 6 PBM 左乘 |
| device-map | src/device_map.rs | Pass 7 设备映射 |
| codegen | src/codegen.rs | Pass 8 ESIR 生成 |
| error | src/error.rs | oi! 宏 + AnimiError |
| ast | src/ast.rs | AST 类型定义 |
| docs | docs/ | ADR/FORGET/ROADMAP 更新 |
| build | Cargo.toml/Makefile | 构建系统 |

---

## 发布节奏（tag → 实现 → tag）

和 KubePivot 一致——tag 先行。tag 是起跑线，不是终点线。

```
1. 打起点 tag（锁定当前功能基线）
   git tag v0.2

2. 实现新功能
   - cargo test → cargo clippy 全绿
   - 每批功能单独 commit（commit 消息用 -F commits/<file>）

3. 实现满意后打终点 tag
   git tag v0.3
```

**为什么 tag 先行**：
- 用 tag 切分"已完成"和"施工中"，回滚有锚点
- 不在功能写到一半时打 tag（tag 代表稳定基线，不绑定半成品）
- tag 是起跑线——不是终点线