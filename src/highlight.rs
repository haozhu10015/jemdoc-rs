use crate::text::{allreplace, hb_format, re_replace_all};

/// Syntax highlighting definition for a programming language.
#[derive(Clone)]
pub struct HighlightDef {
    pub strings: bool,
    pub statement: Vec<String>,
    pub builtin: Vec<String>,
    pub operator: Vec<String>,
    pub special: Vec<String>,
    pub error: Vec<String>,
    pub commentuntilend: Vec<String>,
}

impl Default for HighlightDef {
    fn default() -> Self {
        HighlightDef {
            strings: false,
            statement: Vec::new(),
            builtin: Vec::new(),
            operator: Vec::new(),
            special: Vec::new(),
            error: Vec::new(),
            commentuntilend: Vec::new(),
        }
    }
}

/// Wrap each item with word boundaries \b...\b for precise matching.
fn put_bsbs(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| format!("\\b{}\\b", s)).collect()
}

/// Get syntax highlighting definitions for a given language.
pub fn get_hl(lang: &str) -> HighlightDef {
    let mut d = HighlightDef::default();

    match lang {
        "py" | "python" => {
            d.statement = put_bsbs(&[
                "break", "continue", "del", "except", "exec", "finally", "pass", "print",
                "raise", "return", "try", "with", "global", "assert", "lambda", "yield", "def",
                "class", "for", "while", "if", "elif", "else", "import", "from", "as", "assert",
            ]);
            d.builtin = put_bsbs(&[
                "True", "False", "set", "open", "frozenset", "enumerate", "object", "hasattr",
                "getattr", "filter", "eval", "zip", "vars", "unicode", "type", "str", "repr",
                "round", "range", "and", "in", "is", "not", "or",
            ]);
            d.special = put_bsbs(&[
                "cols", "optvar", "param", "problem", "norm2", "norm1", "value", "minimize",
                "maximize", "rows", "rand", "randn", "printval", "matrix",
            ]);
            d.error = put_bsbs(&[r"\w*Error"]);
            d.commentuntilend = vec!["#".to_string()];
            d.strings = true;
        }
        "c" | "c++" | "cpp" => {
            d.statement = put_bsbs(&["if", "else", "printf", "return", "for"]);
            d.builtin = put_bsbs(&[
                "static", "typedef", "int", "float", "double", "void", "clock_t", "struct",
                "long", "extern", "char",
            ]);
            d.operator = vec![
                "#include.*".to_string(),
                "#define".to_string(),
                "@pyval\\{".to_string(),
                "\\}@".to_string(),
                "@pyif\\{".to_string(),
                "@py\\{".to_string(),
            ];
            d.error = put_bsbs(&[r"\w*Error"]);
            d.commentuntilend = vec![
                "//".to_string(),
                "/*".to_string(),
                " * ".to_string(),
                "*/".to_string(),
            ];

            if lang == "c++" || lang == "cpp" {
                d.builtin
                    .extend(put_bsbs(&["bool", "virtual"]));
                d.statement
                    .extend(put_bsbs(&["new", "delete"]));
                d.operator
                    .extend(vec!["&lt;&lt;".to_string(), "&gt;&gt;".to_string()]);
                d.special = vec![
                    "public".to_string(),
                    "private".to_string(),
                    "protected".to_string(),
                    "template".to_string(),
                    "ASSERT".to_string(),
                ];
            }
        }
        "rb" | "ruby" => {
            d.statement = put_bsbs(&[
                "while", "until", "unless", "if", "elsif", "when", "then", "else", "end",
                "begin", "rescue", "class", "def",
            ]);
            d.operator = put_bsbs(&["and", "not", "or"]);
            d.builtin = put_bsbs(&["true", "false", "require", "warn"]);
            d.special = put_bsbs(&["IO"]);
            d.error = put_bsbs(&[r"\w*Error"]);
            d.commentuntilend = vec!["#".to_string()];
            d.strings = true;
        }
        "sh" => {
            d.statement = put_bsbs(&[
                "cd", "ls", "sudo", "cat", "alias", "for", "do", "done", "in",
            ]);
            d.operator = vec![
                "&gt;".to_string(),
                r"\\".to_string(),
                r"\|".to_string(),
                ";".to_string(),
                "2&gt;".to_string(),
                "monolith&gt;".to_string(),
                "kiwi&gt;".to_string(),
                "ant&gt;".to_string(),
                "kakapo&gt;".to_string(),
                "client&gt;".to_string(),
            ];
            d.builtin = put_bsbs(&[
                "gem",
                "gcc",
                "python",
                "curl",
                "wget",
                "ssh",
                "latex",
                "find",
                "sed",
                "gs",
                "grep",
                "tee",
                "gzip",
                "killall",
                "echo",
                "touch",
                "ifconfig",
                "git",
                r"(?<!\.)tar(?!\.)",
            ]);
            d.commentuntilend = vec!["#".to_string()];
            d.strings = true;
        }
        "matlab" => {
            d.statement = put_bsbs(&[
                "max", "min", "find", "rand", "cumsum", "randn", "help", "error", "if", "end",
                "for",
            ]);
            d.operator = vec![
                "&gt;".to_string(),
                "ans =".to_string(),
                ">>".to_string(),
                "~".to_string(),
                r"\.\.\.".to_string(),
            ];
            d.builtin = put_bsbs(&["csolve"]);
            d.commentuntilend = vec!["%".to_string()];
            d.strings = true;
        }
        "rs" | "rust" => {
            d.statement = put_bsbs(&[
                "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
                "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
                "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
                "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn",
            ]);
            d.builtin = put_bsbs(&[
                "bool", "u8", "u16", "u32", "u64", "u128", "usize",
                "i8", "i16", "i32", "i64", "i128", "isize",
                "f32", "f64", "char", "str", "String", "Vec", "Option", "Result",
                "Some", "None", "Ok", "Err", "Box", "Rc", "Arc",
            ]);
            d.operator = vec![
                "#\\[.*\\]".to_string(),
                "#!\\[.*\\]".to_string(),
            ];
            d.error = put_bsbs(&[r"\w*Error", r"\w*Err"]);
            d.commentuntilend = vec!["//".to_string()];
            d.strings = true;
        }
        "commented" => {
            d.commentuntilend = vec!["#".to_string()];
        }
        _ => {}
    }

    // Add word boundaries (bsbs) to remaining items that don't already have them
    // The Python code calls putbsbs on statement, builtin, special, error at the end
    // but our items already have \b wrapping from put_bsbs above.

    d
}

/// Apply syntax highlighting to a line of code.
/// Returns the highlighted line as HTML.
pub fn format_language(l: &str, hl: &HighlightDef) -> String {
    let mut line = l.trim_end().to_string();
    line = allreplace(&line);

    // Handle strings
    if hl.strings {
        line = re_replace_all(
            r#"(".*?")"#,
            &line,
            "<span CLCLclass=\"string\">$1</span>",
        );
        line = re_replace_all(
            r"('.*?')",
            &line,
            "<span CLCLclass=\"string\">$1</span>",
        );
    }

    // Apply keyword highlighting
    if !hl.statement.is_empty() {
        let pattern = format!("({})", hl.statement.join("|"));
        line = re_replace_all(&pattern, &line, "<span class=\"statement\">$1</span>");
    }

    if !hl.operator.is_empty() {
        let pattern = format!("({})", hl.operator.join("|"));
        line = re_replace_all(&pattern, &line, "<span class=\"operator\">$1</span>");
    }

    if !hl.builtin.is_empty() {
        let pattern = format!("({})", hl.builtin.join("|"));
        line = re_replace_all(&pattern, &line, "<span class=\"builtin\">$1</span>");
    }

    if !hl.special.is_empty() {
        let pattern = format!("({})", hl.special.join("|"));
        line = re_replace_all(&pattern, &line, "<span class=\"special\">$1</span>");
    }

    if !hl.error.is_empty() {
        let pattern = format!("({})", hl.error.join("|"));
        line = re_replace_all(&pattern, &line, "<span class=\"error\">$1</span>");
    }

    // Fix CLCLclass back to class
    line = line.replace("CLCLclass", "class");

    // Handle comment-until-end-of-line
    if !hl.commentuntilend.is_empty() {
        if hl.commentuntilend.len() > 1 {
            // Multiple comment styles (e.g., C/C++)
            for cue in &hl.commentuntilend {
                if line.trim().starts_with(cue.as_str()) {
                    return hb_format(
                        "<span class=\"comment\">|</span>\n",
                        &allreplace(&line),
                        None,
                        None,
                    );
                }
            }
            if hl.commentuntilend.contains(&"//".to_string()) {
                line = re_replace_all(
                    r"//.*",
                    &line,
                    "<span class=\"comment\">$0</span>",
                );
            }
        } else {
            let cue = &hl.commentuntilend[0];
            match cue.as_str() {
                "#" => {
                    line = re_replace_all(
                        r"#.*",
                        &line,
                        "<span class=\"comment\">$0</span>",
                    );
                }
                "%" => {
                    line = re_replace_all(
                        r"%.*",
                        &line,
                        "<span class=\"comment\">$0</span>",
                    );
                }
                _ => {
                    if line.trim().starts_with(cue.as_str()) {
                        return hb_format(
                            "<span class=\"comment\">|</span>\n",
                            &allreplace(&line),
                            None,
                            None,
                        );
                    }
                }
            }
        }
    }

    format!("{}\n", line)
}

/// Python interactive mode highlighting.
pub fn format_pyint(l: &str) -> String {
    let mut line = l.trim_end().to_string();
    line = allreplace(&line);

    line = re_replace_all(r"(#.*)", &line, "<span class = \"comment\">$1</span>");

    if line.starts_with("&gt;&gt;&gt;") {
        hb_format("<span class=\"pycommand\">|</span>\n", &line, None, None)
    } else {
        format!("{}\n", line)
    }
}
