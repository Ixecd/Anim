// build.rs — 错误码文档自动生成（ADR 001）
//
// 从 src/error.rs 提取 AnimiError 枚举变体和 /// 注释，
// 生成 docs/error-codes.md 编目。

use std::fs;

fn main() {
    let src = fs::read_to_string("src/error.rs").expect("无法读取 src/error.rs");
    let lines: Vec<&str> = src.lines().collect();

    let valid = [
        "LexError", "ParseError", "TypeCheckError",
        "StaticSafetyError", "UserStateSafetyError", "InternalError",
    ];

    let mut errors: Vec<(String, String)> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // 收集连续的 /// 注释
        if line.starts_with("///") {
            let mut doc = String::new();
            while i < lines.len() && lines[i].trim().starts_with("///") {
                let d = lines[i].trim().strip_prefix("///").unwrap_or("").trim();
                if !d.is_empty() {
                    doc.push_str(d);
                    doc.push(' ');
                }
                i += 1;
            }

            // 跳过注释后的空行
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }

            // 检查下一行是否是枚举变体
            if i < lines.len() {
                let next = lines[i].trim();
                for name in &valid {
                    if next.starts_with(name) {
                        errors.push((name.to_string(), doc.trim().to_string()));
                        break;
                    }
                }
            }
        } else {
            i += 1;
        }
    }

    let mut out = String::from(
        "# Anim 错误码\n\n\
         > 由 `build.rs` 自动生成——源在 `src/error.rs`。不要手改此文件。\n\n\
         | 错误变体 | 说明 |\n\
         | -------- | ---- |\n",
    );

    for (name, doc) in &errors {
        out.push_str(&format!("| `{}` | {} |\n", name, doc));
    }

    if errors.len() < 4 {
        panic!(
            "error codegen: 只检测到 {} 个错误变体，预期 ≥ 4。检查 src/error.rs 的注释格式。",
            errors.len()
        );
    }

    fs::create_dir_all("docs").ok();
    fs::write("docs/error-codes.md", &out).expect("无法写入 error-codes.md");

    println!("cargo:warning=错误码文档已生成: docs/error-codes.md");
    println!("cargo:rerun-if-changed=src/error.rs");
}
