use fancy_regex::Regex;
use std::env;

// =============================================================================
// MathJax substitution helpers
// =============================================================================

/// Escape underscores in links to prevent MathJax interference.
pub fn mathjax_us_sub(link: &str) -> String {
    link.replace('_', "UNDERSCORE65358")
}

/// Restore underscores.
pub fn mathjax_us_resub(r: &str) -> String {
    r.replace("UNDERSCORE65358", "_")
}

/// Escape special characters in equation text for MathJax processing.
pub fn mathjax_eq_sub(eqtext: &str) -> String {
    eqtext
        .replace('\\', "BACKSLASH65358")
        .replace('[', "OPENBRACKET65358")
        .replace(']', "CLOSEBRACKET65358")
        .replace('*', "ASTERISK65358")
        .replace('+', "PLUS65358")
        .replace('&', "AMPERSAND65358")
        .replace('<', "LESSTHAN65358")
        .replace('>', "GREATERTHAN65358")
        .replace('_', "UNDERSCORE65358")
        .replace('/', "SLASH65358")
}

/// Restore special characters after MathJax processing.
pub fn mathjax_eq_resub(r: &str) -> String {
    r.replace("BACKSLASH65358", "\\")
        .replace("OPENBRACKET65358", "[")
        .replace("CLOSEBRACKET65358", "]")
        .replace("ASTERISK65358", "*")
        .replace("PLUS65358", "+")
        .replace("AMPERSAND65358", "&")
        .replace("LESSTHAN65358", "<")
        .replace("GREATERTHAN65358", ">")
        .replace("QUOTATION65358", "\"")
        .replace("UNDERSCORE65358", "_")
        .replace("SLASH65358", "/")
}

// =============================================================================
// Core text replacement utilities
// =============================================================================

/// Helper to do regex replace_all using fancy_regex (which supports lookbehinds).
/// This manually iterates through matches since fancy_regex's replace_all
/// may have limitations with complex replacement patterns.
pub fn re_replace_all(pattern: &str, text: &str, replacement: &str) -> String {
    let re = Regex::new(pattern).expect(&format!("Invalid regex: {}", pattern));
    let mut result = String::new();
    let mut last_end = 0;
    let mut search_start = 0;

    loop {
        if search_start > text.len() {
            break;
        }
        match re.captures_from_pos(text, search_start) {
            Ok(Some(caps)) => {
                let m = caps.get(0).unwrap();
                result.push_str(&text[last_end..m.start()]);

                // Process replacement string with group substitution
                let mut rep = replacement.to_string();
                // Replace $0, $1, $2, ... with captured groups
                // Process in reverse order to handle $10 before $1
                for i in (0..caps.len()).rev() {
                    if let Some(g) = caps.get(i) {
                        rep = rep.replace(&format!("${}", i), g.as_str());
                    } else {
                        rep = rep.replace(&format!("${}", i), "");
                    }
                }
                result.push_str(&rep);

                last_end = m.end();
                search_start = if m.start() == m.end() {
                    m.end() + 1
                } else {
                    m.end()
                };
            }
            _ => break,
        }
    }
    result.push_str(&text[last_end..]);
    result
}

/// Replace &, >, < with HTML entities (except when preceded by backslash).
pub fn allreplace(b: &str) -> String {
    let mut s = re_replace_all(r"(?ms)(?<!\\)&", b, "&amp;");
    s = re_replace_all(r"(?ms)(?<!\\)>", &s, "&gt;");
    s = re_replace_all(r"(?ms)(?<!\\)<", &s, "&lt;");
    s
}

/// Quote special characters in a string by preceding them with backslash.
pub fn quote(s: &str) -> String {
    let re = Regex::new(r#"[\\*/+"'<>&$%\.~\[\]\-]"#).unwrap();
    let mut result = String::new();
    let mut last_end = 0;
    let mut search_start = 0;

    loop {
        if search_start > s.len() {
            break;
        }
        match re.find_from_pos(s, search_start) {
            Ok(Some(m)) => {
                result.push_str(&s[last_end..m.start()]);
                result.push('\\');
                result.push_str(m.as_str());
                last_end = m.end();
                search_start = m.end();
            }
            _ => break,
        }
    }
    result.push_str(&s[last_end..]);
    result
}

/// Process {{raw html}} sections by quoting their contents.
pub fn replace_quoted(b: &str) -> String {
    let re = Regex::new(r"(?ms)\{\{(.*?)\}\}").unwrap();
    let mut result = b.to_string();
    let mut search_start = 0;

    loop {
        if search_start > result.len() {
            break;
        }
        match re.captures_from_pos(&result, search_start) {
            Ok(Some(caps)) => {
                let m = caps.get(0).unwrap();
                let inner = caps.get(1).unwrap().as_str();
                let quoted = quote(inner);
                let new_result =
                    format!("{}{}{}", &result[..m.start()], quoted, &result[m.end()..]);
                search_start = m.start() + quoted.len();
                result = new_result;
            }
            _ => break,
        }
    }
    result
}

/// Replace %sections% as +{{sections}}+ (monospace with raw HTML protection).
pub fn replace_percents(b: &str) -> String {
    let re = Regex::new(r"(?ms)(?<!\\)%(.*?)(?<!\\)%").unwrap();
    let mut result = b.to_string();
    let mut search_start = 0;

    loop {
        if search_start > result.len() {
            break;
        }
        match re.captures_from_pos(&result, search_start) {
            Ok(Some(caps)) => {
                let m = caps.get(0).unwrap();
                let inner = caps.get(1).unwrap().as_str();
                let a = inner
                    .replace('[', "BSNOTLINKLEFT12039XX")
                    .replace(']', "BSNOTLINKRIGHT12039XX");
                let replacement = format!("+{{{{{}}}}}+", a); // +{{content}}+
                let new_result = format!(
                    "{}{}{}",
                    &result[..m.start()],
                    replacement,
                    &result[m.end()..]
                );
                search_start = m.start() + replacement.len();
                result = new_result;
            }
            _ => break,
        }
    }
    result
}

/// Replace $equations$ and \(equations\) with MathJax delimiters.
pub fn replace_equations(b: &str) -> String {
    let mut result = b.to_string();

    // Pattern 1: $...$ → inline math \(...\)
    let re_inline = Regex::new(r"(?ms)(?<!\\)\$(.*?)(?<!\\)\$").unwrap();
    let mut search_start = 0;
    loop {
        if search_start > result.len() {
            break;
        }
        match re_inline.captures_from_pos(&result, search_start) {
            Ok(Some(caps)) => {
                let m = caps.get(0).unwrap();
                let eq = caps.get(1).unwrap().as_str();
                let mut eqtext = allreplace(eq);
                eqtext = mathjax_eq_sub(&eqtext);
                eqtext = eqtext.replace("{{", "DOUBLEOPENBRACE");
                eqtext = eqtext.replace("}}", "DOUBLECLOSEBRACE");

                let replacement = format!("BACKSLASH65358({eqtext}BACKSLASH65358)");
                let new_result = format!(
                    "{}{}{}",
                    &result[..m.start()],
                    replacement,
                    &result[m.end()..]
                );
                search_start = m.start() + replacement.len();
                result = new_result;
            }
            _ => break,
        }
    }

    // Pattern 2: \(...\) → display math \[...\]
    let re_display = Regex::new(r"(?ms)(?<!\\)\\\((.*?)(?<!\\)\\\)").unwrap();
    search_start = 0;
    loop {
        if search_start > result.len() {
            break;
        }
        match re_display.captures_from_pos(&result, search_start) {
            Ok(Some(caps)) => {
                let m = caps.get(0).unwrap();
                let eq = caps.get(1).unwrap().as_str();
                let mut eqtext = allreplace(eq);
                eqtext = mathjax_eq_sub(&eqtext);
                eqtext = eqtext.replace("{{", "DOUBLEOPENBRACE");
                eqtext = eqtext.replace("}}", "DOUBLECLOSEBRACE");

                let inner = format!(
                    "BACKSLASH65358OPENBRACKET65358{eqtext}BACKSLASH65358CLOSEBRACKET65358"
                );
                let replacement = format!(
                    "<p style=QUOTATION65358text-align:centerQUOTATION65358>\n{inner}\n</p>"
                );
                let replacement = mathjax_eq_sub(&replacement);
                let new_result = format!(
                    "{}{}{}",
                    &result[..m.start()],
                    replacement,
                    &result[m.end()..]
                );
                search_start = m.start() + replacement.len();
                result = new_result;
            }
            _ => break,
        }
    }

    replace_quoted(&result)
}

/// Replace [img{width}{height}{alttext} location caption] with <img> tags.
pub fn replace_images(b: &str) -> String {
    let re = Regex::new(r"(?ms)(?<!\\)\[img((?:\{.*?\}){0,3})\s(.*?)(?:\s(.*?))?(?<!\\)\]")
        .unwrap();
    let re_braces = Regex::new(r"(?ms)\{(.*?)\}").unwrap();
    let mut result = b.to_string();
    let mut search_start = 0;

    loop {
        if search_start > result.len() {
            break;
        }
        match re.captures_from_pos(&result, search_start) {
            Ok(Some(caps)) => {
                let m = caps.get(0).unwrap();
                let attrs_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let location = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                let _caption = caps.get(3).map(|m| m.as_str().trim());

                // Extract {width}, {height}, {alttext}
                let mut attrs: Vec<String> = Vec::new();
                let mut brace_start = 0;
                while let Ok(Some(bc)) = re_braces.captures_from_pos(attrs_str, brace_start) {
                    let bm = bc.get(0).unwrap();
                    attrs.push(bc.get(1).unwrap().as_str().to_string());
                    brace_start = bm.end();
                }
                while attrs.len() < 3 {
                    attrs.push(String::new());
                }

                let mut bits = Vec::new();
                bits.push(format!("src=\\\"{}\\\"" , quote(location)));

                if !attrs[0].is_empty() {
                    let w = if attrs[0].chars().all(|c| c.is_ascii_digit()) {
                        format!("{}px", attrs[0])
                    } else {
                        attrs[0].clone()
                    };
                    bits.push(format!("width=\\\"{}\\\"" , quote(&w)));
                }
                if !attrs[1].is_empty() {
                    let h = if attrs[1].chars().all(|c| c.is_ascii_digit()) {
                        format!("{}px", attrs[1])
                    } else {
                        attrs[1].clone()
                    };
                    bits.push(format!("height=\\\"{}\\\"" , quote(&h)));
                }
                if !attrs[2].is_empty() {
                    bits.push(format!("alt=\\\"{}\\\"" , quote(&attrs[2])));
                } else {
                    bits.push("alt=\\\"\\\"".to_string());
                }

                let replacement = format!("<img {} />", bits.join(" "));
                let new_result = format!(
                    "{}{}{}",
                    &result[..m.start()],
                    replacement,
                    &result[m.end()..]
                );
                search_start = m.start() + replacement.len();
                result = new_result;
            }
            _ => break,
        }
    }
    result
}

/// Replace [link.html text] with <a href="...">text</a>.
pub fn replace_links(b: &str) -> String {
    let re = Regex::new(r"(?ms)(?<!\\)\[(.*?)(?:\s(.*?))?(?<!\\)\]").unwrap();
    let mut result = b.to_string();
    let mut search_start = 0;

    loop {
        if search_start > result.len() {
            break;
        }
        match re.captures_from_pos(&result, search_start) {
            Ok(Some(caps)) => {
                let m = caps.get(0).unwrap();
                let mut m1 = caps.get(1).unwrap().as_str().trim().to_string();

                let option;
                if m1.starts_with('/') {
                    option = "";
                    m1 = m1[1..].to_string();
                } else {
                    option = "TARGETBLANK65358";
                }

                let mut link = if m1.contains('@')
                    && !m1.starts_with("mailto:")
                    && !m1.starts_with("http://")
                {
                    format!("mailto:{}", m1)
                } else {
                    m1.clone()
                };

                // Unquote hashes
                link = link.replace("\\#", "#");
                // Remove +{{ or }}+ markers
                link = link.replace("+{{", "%").replace("}}+", "%");
                link = quote(&link);
                link = mathjax_us_sub(&link);

                let linkname = if let Some(name_match) = caps.get(2) {
                    name_match.as_str().trim().to_string()
                } else {
                    link.replace("mailto:", "").clone()
                };

                let replacement = format!(
                    "<a href=\\\"{}\\\"{}>{}<\\/a>",
                    link, option, linkname
                );
                let new_result = format!(
                    "{}{}{}",
                    &result[..m.start()],
                    replacement,
                    &result[m.end()..]
                );
                search_start = m.start() + replacement.len();
                result = new_result;
            }
            _ => break,
        }
    }
    result
}

// =============================================================================
// Block replacement (br) - the main text processing pipeline
// =============================================================================

/// Remove a trailing comment from a line (unescaped # and everything after).
pub fn remove_trailing_comment(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == '#' && (i == 0 || chars[i - 1] != '\\') {
            let mut end = i;
            while end > 0
                && matches!(
                    chars[end - 1],
                    ' ' | '\t' | '\r' | '\n'
                )
            {
                end -= 1;
            }
            let result: String = chars[..end].iter().collect();
            return result;
        }
    }
    s.trim_end_matches('\n').to_string()
}

/// Main text replacement pipeline. Converts jemdoc markup to HTML.
///
/// Parameters:
/// - `b`: the text block to process
/// - `eqs`: whether equations are enabled
/// - `tableblock`: whether we're inside a table
/// - `tablerow`: mutable reference to table row counter
pub fn br(b: &str, eqs: bool, tableblock: bool, tablerow: &mut usize) -> String {
    let mut b = b.to_string();

    // Deal with environment variables
    let re_env = Regex::new(r"(?ms)!\$(\w{2,})\$!").unwrap();
    let mut search_start = 0;
    loop {
        if search_start > b.len() {
            break;
        }
        let caps = re_env.captures_from_pos(&b, search_start);
        match caps {
            Ok(Some(caps)) => {
                let m_start = caps.get(0).unwrap().start();
                let m_end = caps.get(0).unwrap().end();
                let var_name = caps.get(1).unwrap().as_str().to_string();
                let replacement = match env::var(&var_name) {
                    Ok(val) => val,
                    Err(_) => format!("FAILED_MATCH_{}", var_name),
                };
                b = format!("{}{}{}", &b[..m_start], replacement, &b[m_end..]);
                search_start = m_start + replacement.len();
            }
            _ => break,
        }
    }

    // Deal with equations
    if eqs {
        b = replace_equations(&b);
    }

    // Deal with literal backslashes
    b = b.replace("\\\\", "jemLITerl33talBS");

    // Deal with {{html embedding}}
    b = replace_quoted(&b);

    // HTML entity escaping
    b = allreplace(&b);

    // Remove leading spaces, tabs, dashes, dots
    b = b.trim_start_matches(|c: char| c == '-' || c == '.' || c == ' ' || c == '\t').to_string();

    // Replace images
    b = replace_images(&b);

    // Replace percents
    b = replace_percents(&b);

    // Replace links
    b = replace_links(&b);

    // Restore bracket markers
    b = b.replace("BSNOTLINKLEFT12039XX", "[");
    b = b.replace("BSNOTLINKRIGHT12039XX", "]");

    // Quote remaining raw HTML sections
    b = replace_quoted(&b);

    // Italics: /text/
    b = re_replace_all(r"(?ms)(?<!\\)/(.*?)(?<!\\)/", &b, "<i>$1</i>");

    // Bold: *text*
    b = re_replace_all(r"(?ms)(?<!\\)\*(.*?)(?<!\\)\*", &b, "<b>$1</b>");

    // Underline: _text_
    b = re_replace_all(r"(?ms)(?<!\\)_(.*?)(?<!\\)_", &b, "<u>$1</u>");
    b = mathjax_us_resub(&b);

    // Monospace: +text+
    b = re_replace_all(r"(?ms)(?<!\\)\+(.*?)(?<!\\)\+", &b, "<tt>$1</tt>");

    // Double quotes: "text"
    b = re_replace_all(
        r#"(?ms)(?<!\\)"(.*?)(?<!\\)""#,
        &b,
        "&ldquo;$1&rdquo;",
    );

    // Left quote: `
    b = re_replace_all(r"(?ms)(?<!\\)`", &b, "&lsquo;");

    // Apostrophe: ' (not followed by a letter)
    b = re_replace_all(r"(?ms)(?<!\\)'(?![a-zA-Z])", &b, "&rsquo;");

    // Em dash: ---
    b = re_replace_all(r"(?ms)(?<!\\)---", &b, "&#8201;&mdash;&#8201;");

    // En dash: --
    b = re_replace_all(r"(?ms)(?<!\\)--", &b, "&ndash;");

    // Ellipsis: ...
    b = re_replace_all(r"(?ms)(?<!\\)\.\.\.", &b, "&hellip;");

    // Non-breaking space: ~
    b = re_replace_all(r"(?ms)(?<!\\)~", &b, "&nbsp;");
    b = b.replace("TILDE", "~");

    // Registered trademark: \R
    b = re_replace_all(r"(?ms)(?<!\\)\\R", &b, "&reg;");

    // Copyright: \C
    b = re_replace_all(r"(?ms)(?<!\\)\\C", &b, "&copy;");

    // Middot: \M
    b = re_replace_all(r"(?ms)(?<!\\)\\M", &b, "&middot;");

    // Line break: \n
    b = re_replace_all(r"(?ms)(?<!\\)\\n", &b, "<br />");

    // Paragraph break: \p
    b = re_replace_all(r"(?ms)(?<!\\)\\p", &b, "</p><p>");

    // Table processing
    if tableblock {
        let re_rowbreak = Regex::new(r"(?ms)(?<!\\)\|\|").unwrap();
        let re_colbreak = Regex::new(r"(?ms)(?<!\\)\|").unwrap();

        let bcopy = b.clone();
        b = String::new();

        for line in bcopy.split('\n') {
            if line.is_empty() {
                continue;
            }
            *tablerow += 1;

            // Replace || with row breaks
            let mut processed_line = String::new();
            let mut last_end = 0;
            let mut ss = 0;
            loop {
                if ss > line.len() {
                    break;
                }
                match re_rowbreak.find_from_pos(line, ss) {
                    Ok(Some(m)) => {
                        processed_line.push_str(&line[last_end..m.start()]);
                        processed_line.push_str(&format!(
                            "</td></tr>\n<tr class=\"r{}\"><td class=\"c1\">",
                            tablerow
                        ));
                        last_end = m.end();
                        ss = m.end();
                    }
                    _ => break,
                }
            }
            processed_line.push_str(&line[last_end..]);

            // Replace | with column breaks
            let mut final_line = String::new();
            let mut col = 2;
            let parts: Vec<&str> = {
                let mut parts = Vec::new();
                let mut last = 0;
                let mut ss2 = 0;
                loop {
                    if ss2 > processed_line.len() {
                        break;
                    }
                    match re_colbreak.find_from_pos(&processed_line, ss2) {
                        Ok(Some(m)) => {
                            parts.push(&processed_line[last..m.start()]);
                            last = m.end();
                            ss2 = m.end();
                        }
                        _ => break,
                    }
                }
                parts.push(&processed_line[last..]);
                parts
            };

            for (i, part) in parts.iter().enumerate() {
                final_line.push_str(part);
                if i < parts.len() - 1 {
                    final_line.push_str(&format!("</td><td class=\"c{}\">", col));
                    col += 1;
                }
            }

            b.push_str(&final_line);
        }
    }

    // Remove remaining quoting backslashes (not followed by another backslash)
    b = re_replace_all(r"\\(?!\\)", &b, "");

    // Restore literal backslashes
    b = b.replace("jemLITerl33talBS", "\\");

    // Restore double braces
    b = b.replace("DOUBLEOPENBRACE", "{{");
    b = b.replace("DOUBLECLOSEBRACE", "}}");

    // Restore target="_blank" attribute
    b = b.replace("TARGETBLANK65358", r#" target="_blank""#);

    b
}

// =============================================================================
// Half-block (hb) formatting
// =============================================================================

/// Format a half-block template by replacing | placeholders with content.
/// If content2 is None, replaces all | with content1.
/// If content2 is Some, replaces |1, |2, |3 with content1, content2, content3.
pub fn hb_format(
    tag: &str,
    content1: &str,
    content2: Option<&str>,
    content3: Option<&str>,
) -> String {
    let c3 = content3.unwrap_or("");

    // Protect literal "||" (e.g. JavaScript OR) from being split by | replacement
    let tag = tag.replace("||", "\x00DOUBLEPIPE\x00");

    let r = if let Some(c2) = content2 {
        let r = tag.replace("|1", content1);
        let r = r.replace("|3", c3);
        r.replace("|2", c2)
    } else {
        let r = tag.replace('|', content1);
        r.replace("|3", c3)
    };

    let r = r.replace("\x00DOUBLEPIPE\x00", "||");
    mathjax_eq_resub(&r)
}

/// Prepend non-breaking spaces for leading spaces in a line.
pub fn prepend_nbsps(l: &str) -> String {
    let leading_spaces = l.len() - l.trim_start_matches(' ').len();
    let rest = &l[leading_spaces..];
    format!("{}{}", "&nbsp;".repeat(leading_spaces), rest)
}
