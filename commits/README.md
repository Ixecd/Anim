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

## 推荐模板

feat(lexer): add tokenizer for .anim source
fix(parser): resolve mix block indent edge case
docs(fsir): update FSIR binary layout spec
test(typeck): cover unregistered feeling atom error
refactor(pass4): split RuntimeGuard into pre-check and inject

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
| static-safety | src/static_safety.rs | Pass 2 静态安全 |
| user-safety | src/user_safety.rs | Pass 3 用户状态安全 |
| runtime-guard | src/runtime_guard.rs | Pass 4 运行期插桩 |
| fsir | src/fsir.rs | Pass 5 FSIR 生成 |
| pbm | src/pbm.rs | Pass 6 PBM 左乘 |
| device-map | src/device_map.rs | Pass 7 设备映射 |
| codegen | src/codegen.rs | Pass 8 ESIR 生成 |
| error | src/error.rs | oi! 宏 + AnimiError |
| ast | src/ast.rs | AST 类型定义 |
| docs | docs/ | ADR/FORGET/ROADMAP 更新 |
| build | Cargo.toml/Makefile | 构建系统 |

---

## 发布节奏

Anim 走 Rust 标准发布流程——Cargo.toml 版本号 + git tag。不打半成品 tag。

```
1. 实现功能 → cargo test → cargo clippy 全绿
2. 每批功能单独 commit（commit 消息用 `-F commits/<file>`）
3. 功能完整后 bump Cargo.toml version + git tag v0.x
```