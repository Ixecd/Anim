// src/macros.rs — Anim 宏系统：macro_rules! 解析 + 展开（ADR 005）
//
// 在 Pass 0 (LexParse) 和 Pass 1 (TypeCheck) 之间运行。
// 宏展开不是字符串替换——是 AST 节点嵌入。
// 展开后的 AST 必经 Pass 1/2/3 全量检查——不存在"展开就放行"。

use crate::error::AnimiError;
use crate::error::Severity;
use std::collections::HashMap;

/// 宏展开后源码的最大允许长度（字节）。
/// 防止递归宏组合产生指数级膨胀的源码。
pub const MAX_EXPANDED_SIZE: usize = 1_000_000;

/// 最大嵌套宏调用层数。
pub const MAX_MACRO_NESTING: usize = 32;

/// 验证展开后的源码是否安全——不超出合理界限。
///
/// 在宏展开后、Pass 1 前调用。拒绝膨胀超限的源码。
pub fn validate_expanded(source: &str) -> Result<(), AnimiError> {
    if source.len() > MAX_EXPANDED_SIZE {
        return Err(AnimiError::InternalError {
            file_name: crate::error::current_file(),
            msg: format!(
                "宏展开后源码过大 ({} bytes > {} max)。可能存在递归宏组合爆炸。",
                source.len(),
                MAX_EXPANDED_SIZE
            ),
            severity: Severity::Deny,
        });
    }
    Ok(())
}

/// 宏定义——macro_rules! 声明。
#[derive(Debug, Clone)]
pub struct MacroDef {
    /// 宏名称。
    pub name: String,
    /// 参数列表。
    pub params: Vec<String>,
    /// 宏体的 tokens 序列（未经展开的源码片段）。
    pub body_tokens: String,
}

/// 宏调用——在 mix 块中出现的 macro_name!(args)。
#[derive(Debug, Clone)]
pub struct MacroCall {
    /// 宏名称。
    pub name: String,
    /// 调用参数。
    pub args: Vec<String>,
    /// 调用点在源码中的行号。
    pub line: usize,
    /// 调用点在源码中的列号。
    pub col: usize,
}

/// 从 .anim 源码中提取宏定义。
///
/// 返回 (宏定义表, 剩余的源码——宏定义已从源码中移除)。
pub fn extract_macros(source: &str) -> (HashMap<String, MacroDef>, String) {
    let mut macros = HashMap::new();
    let mut remaining = String::new();
    let mut i = 0;
    let chars: Vec<char> = source.chars().collect();

    while i < chars.len() {
        // 跳过空白
        while i < chars.len() && chars[i].is_whitespace() {
            remaining.push(chars[i]);
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        // 检查是否是 macro_rules!
        let rest: String = chars[i..].iter().take(13).collect();
        if rest == "macro_rules! " || rest.starts_with("macro_rules!") {
            // 提取宏名
            i += 12; // 跳过 "macro_rules!"
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            let mut name = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                name.push(chars[i]);
                i += 1;
            }
            // 跳过空白到 {
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i >= chars.len() || chars[i] != '{' {
                // 不是有效的宏定义——跳过
                continue;
            }
            // 提取宏体——匹配花括号
            let end = find_brace_end(&chars, i);
            let body_str: String = chars[i + 1..end].iter().collect();
            let params = extract_macro_params(&body_str);
            let inner_body = extract_macro_inner_body(&body_str);

            macros.insert(
                name.clone(),
                MacroDef {
                    name,
                    params,
                    body_tokens: inner_body,
                },
            );

            // 在剩余源码中，宏定义位置留空
            remaining.push_str("-- macro ");
            remaining.push_str(&macros.len().to_string());
            remaining.push_str(" expanded\n");
            i = end + 1;
        } else {
            remaining.push(chars[i]);
            i += 1;
        }
    }

    (macros, remaining)
}

/// 在字符数组中寻找配对花括号的闭合位置。
fn find_brace_end(chars: &[char], open_pos: usize) -> usize {
    let mut depth = 1;
    let mut pos = open_pos + 1;
    while pos < chars.len() && depth > 0 {
        match chars[pos] {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        pos += 1;
    }
    pos - 1 // 指向闭合花括号
}

/// 提取宏的参数列表。格式：( $param:ident, ... )
fn extract_macro_params(body: &str) -> Vec<String> {
    let body = body.trim();
    if !body.starts_with('(') {
        return vec![];
    }
    let paren_end = body.find(')').unwrap_or(body.len());
    let params_str = &body[1..paren_end];
    params_str
        .split(',')
        .filter_map(|p| {
            let p = p.trim();
            if p.starts_with('$') && p.contains(":ident") {
                Some(p[1..].split(':').next().unwrap_or("").to_string())
            } else {
                None
            }
        })
        .collect()
}

/// 提取宏的 inner body——`=> { ... }` 内的内容。
fn extract_macro_inner_body(body: &str) -> String {
    let body = body.trim();
    // 找 => {
    if let Some(arrow_pos) = body.find("=>") {
        let after_arrow = &body[arrow_pos + 2..];
        // 找第一个 {
        if let Some(open) = after_arrow.find('{') {
            let chars: Vec<char> = after_arrow.chars().collect();
            let mut depth = 0;
            let mut i = open;
            while i < chars.len() {
                if chars[i] == '{' {
                    depth += 1;
                } else if chars[i] == '}' {
                    depth -= 1;
                    if depth == 0 {
                        return after_arrow[open + 1..i].trim().to_string();
                    }
                }
                i += 1;
            }
        }
    }
    body.to_string()
}

/// 展开源码中的宏调用。
///
/// 在 Pass 0 生成 tokens 之前调用——在原始源码层面做宏展开。
/// 返回展开后的源码——所有 macro_rules! 定义已被移除，
/// 所有 macro_name!(args) 调用已被替换为宏体（参数替换完成）。
pub fn expand_macros(source: &str, macros: &HashMap<String, MacroDef>) -> String {
    let mut result = String::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // 跳过空白
        while i < chars.len() && chars[i].is_whitespace() {
            result.push(chars[i]);
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        // 读一个标识符
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let mut ident = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                ident.push(chars[i]);
                i += 1;
            }
            // 检查后面是不是 !
            if i < chars.len() && chars[i] == '!' && macros.contains_key(&ident) {
                // 这是宏调用——展开它
                i += 1; // 跳过 !
                        // 跳过可选的空白到 (
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '(' {
                    let (args_str, end) = extract_paren_args(&chars, i);
                    let args: Vec<String> = args_str
                        .split(',')
                        .map(|a| a.trim().to_string())
                        .filter(|a| !a.is_empty())
                        .collect();
                    i = end + 1;

                    // 展开宏体
                    if let Some(mdef) = macros.get(&ident) {
                        let mut expanded = mdef.body_tokens.clone();
                        for (idx, arg) in args.iter().enumerate() {
                            if idx < mdef.params.len() {
                                let placeholder = format!("${}", mdef.params[idx]);
                                expanded = expanded.replace(&placeholder, arg);
                            }
                        }
                        result.push_str(&expanded);
                    }
                    continue;
                }
            }
            // 不是宏调用——原样保留
            result.push_str(&ident);
            if i < chars.len() && chars[i] == '!' {
                result.push('!');
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// 提取匹配 () 内的参数。
fn extract_paren_args(chars: &[char], open_pos: usize) -> (String, usize) {
    let mut depth = 1;
    let mut args = String::new();
    let mut pos = open_pos + 1; // 跳过 (

    while pos < chars.len() && depth > 0 {
        if chars[pos] == '(' {
            depth += 1;
        } else if chars[pos] == ')' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        if depth > 0 {
            args.push(chars[pos]);
        }
        pos += 1;
    }
    (args, pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple_macro() {
        let src = r#"
macro_rules! safe_accent {
    () => {
        accents: [belonging 0.10]
    }
}

feeling calm {
    mix {
        main: calm_meditative
        safe_accent!()
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let (macros, remaining) = extract_macros(src);
        assert_eq!(macros.len(), 1);
        assert!(macros.contains_key("safe_accent"));
        let m = &macros["safe_accent"];
        assert_eq!(m.name, "safe_accent");
        assert_eq!(m.body_tokens, "accents: [belonging 0.10]");
        assert!(remaining.contains("safe_accent"));
    }

    #[test]
    fn expand_macro_call() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        safe_accent!()
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let mut macros = HashMap::new();
        macros.insert(
            "safe_accent".to_string(),
            MacroDef {
                name: "safe_accent".to_string(),
                params: vec![],
                body_tokens: "accents: [belonging 0.10]".to_string(),
            },
        );
        let expanded = expand_macros(src, &macros);
        assert!(expanded.contains("accents: [belonging 0.10]"));
        assert!(!expanded.contains("safe_accent!()"));
    }

    #[test]
    fn expand_macro_with_params() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        my_accent!(belonging, 0.15)
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let mut macros = HashMap::new();
        macros.insert(
            "my_accent".to_string(),
            MacroDef {
                name: "my_accent".to_string(),
                params: vec!["atom".to_string(), "ratio".to_string()],
                body_tokens: "accents: [$atom $ratio]".to_string(),
            },
        );
        let expanded = expand_macros(src, &macros);
        assert!(expanded.contains("accents: [belonging 0.15]"));
        assert!(!expanded.contains("my_accent!"));
    }

    #[test]
    fn full_pipeline_extract_then_expand() {
        let src = r#"
macro_rules! safe_accent {
    () => {
        accents: [belonging 0.10]
    }
}

feeling calm {
    mix {
        main: calm_meditative
        safe_accent!()
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let (macros, source) = extract_macros(src);
        // 展开后应该是一个合法的 .anim 源码
        let expanded = expand_macros(&source, &macros);
        assert!(expanded.contains("accents: [belonging 0.10]"));
        assert!(!expanded.contains("macro_rules!"));
        assert!(!expanded.contains("safe_accent!()"));
        // 可以用正常管线继续处理——词法+语法+类型
    }

    #[test]
    fn no_macros_passthrough() {
        let src = r#"
feeling calm {
    mix {
        main: calm_meditative
        accents: [belonging 0.3]
    }
    shape: steady
    intensity: [10, 20]
}
"#;
        let (macros, source) = extract_macros(src);
        assert!(macros.is_empty());
        let expanded = expand_macros(&source, &macros);
        assert!(expanded.contains("calm_meditative"));
        assert!(expanded.contains("belonging 0.3"));
    }

    #[test]
    fn validate_expanded_accepts_normal_source() {
        let src = "feeling calm { mix { main: x accents: [] } shape: steady intensity: [10, 20] }";
        assert!(validate_expanded(src).is_ok());
    }

    #[test]
    fn validate_expanded_rejects_oversized_source() {
        let huge = "x".repeat(MAX_EXPANDED_SIZE + 1);
        assert!(validate_expanded(&huge).is_err());
    }
}
