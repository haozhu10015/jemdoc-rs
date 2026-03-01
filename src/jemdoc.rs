use std::collections::HashMap;
use std::fs;
use std::io::Write;

use fancy_regex::Regex;

use crate::text::{
    allreplace, br, hb_format, mathjax_eq_resub, prepend_nbsps,
    re_replace_all, remove_trailing_comment,
};
use crate::highlight::{format_language, format_pyint, get_hl};

/// The main jemdoc parser and processor.
pub struct JemdocParser {
    /// All input lines (each ending with \n).
    pub lines: Vec<String>,
    /// Current position (line index).
    pub pos: usize,
    /// Current line number (for error messages).
    pub linenum: usize,
    /// Output writer.
    pub outf: Box<dyn Write>,
    /// Configuration map (section tag -> HTML content).
    pub conf: HashMap<String, String>,
    /// Input filename.
    pub inname: String,
    /// Whether equations are enabled.
    pub eqs: bool,
    /// Google Analytics tracking ID.
    pub analytics: Option<String>,
    /// Current table row number.
    pub tablerow: usize,
    /// Current table column number.
    pub tablecol: usize,
    /// File inclusion stack: (lines, pos, linenum).
    pub file_stack: Vec<(Vec<String>, usize, usize)>,
}

impl JemdocParser {
    /// Create a new JemdocParser.
    pub fn new(
        inname: String,
        lines: Vec<String>,
        outf: Box<dyn Write>,
        conf: HashMap<String, String>,
    ) -> Self {
        JemdocParser {
            lines,
            pos: 0,
            linenum: 0,
            outf,
            conf,
            inname,
            eqs: true,
            analytics: None,
            tablerow: 0,
            tablecol: 0,
            file_stack: Vec::new(),
        }
    }

    // =========================================================================
    // Output helpers
    // =========================================================================

    /// Write a string to the output.
    pub fn out(&mut self, s: &str) {
        let _ = self.outf.write_all(s.as_bytes());
    }

    /// Write a half-block to the output.
    pub fn hb(&mut self, tag: &str, content1: &str, content2: Option<&str>, content3: Option<&str>) {
        let s = hb_format(tag, content1, content2, content3);
        self.out(&s);
    }

    // =========================================================================
    // File inclusion
    // =========================================================================

    /// Push the current file onto the stack and switch to a new include file.
    pub fn push_file(&mut self, filename: &str) {
        let old_lines = std::mem::take(&mut self.lines);
        let old_pos = self.pos;
        let old_linenum = self.linenum;
        self.file_stack.push((old_lines, old_pos, old_linenum));

        let content = fs::read_to_string(filename).unwrap_or_default();
        self.lines = content.lines().map(|l| format!("{}\n", l)).collect();
        self.pos = 0;
        self.linenum = 0;
    }

    /// Pop back to the previous file from the stack.
    pub fn next_file(&mut self) {
        if let Some((lines, pos, linenum)) = self.file_stack.pop() {
            self.lines = lines;
            self.pos = pos;
            self.linenum = linenum;
        }
    }

    /// Get a config section value, or empty string if not found.
    fn conf(&self, key: &str) -> String {
        self.conf.get(key).cloned().unwrap_or_default()
    }

    /// Process a potential include directive. Returns true if it was an include.
    pub fn do_includes(&mut self, l: &str) -> bool {
        let l = l.trim();
        if l.starts_with("includeraw{") && l.ends_with('}') {
            let filename = &l["includeraw{".len()..l.len() - 1];
            if let Ok(content) = fs::read_to_string(filename) {
                self.out(&content);
            }
            return true;
        } else if l.starts_with("include{") && l.ends_with('}') {
            let filename = &l["include{".len()..l.len() - 1];
            self.push_file(filename);
            return true;
        }
        false
    }

    // =========================================================================
    // Input reading primitives
    // =========================================================================

    /// Peek at the next meaningful character without consuming non-comment lines.
    /// Comment lines ARE consumed and skipped.
    pub fn pc(&mut self, ditch_comments: bool) -> String {
        loop {
            if self.pos >= self.lines.len() {
                if !self.file_stack.is_empty() {
                    self.next_file();
                    continue;
                }
                return String::new(); // EOF
            }

            let line = self.lines[self.pos].clone();
            let trimmed = line
                .trim_start_matches(|c: char| c == ' ' || c == '\t');

            // Empty line
            if trimmed.is_empty() || trimmed == "\n" {
                return "\n".to_string();
            }

            let first_char = match trimmed.chars().next() {
                Some(c) => c,
                None => return "\n".to_string(),
            };

            // Handle comments
            if ditch_comments && first_char == '#' {
                let comment_content = &trimmed[1..].trim();
                self.pos += 1;
                self.linenum += 1;

                if self.do_includes(comment_content) {
                    return "#".to_string();
                }
                // Skip this comment line
                continue;
            }

            // Handle backslash-prefixed tokens like \( and \)
            if first_char == '\\' && trimmed.len() > 1 {
                let second = trimmed.chars().nth(1).unwrap();
                return format!("\\{}", second);
            }

            return first_char.to_string();
        }
    }

    /// Get the next input line.
    /// If withcount is true, also returns the count of leading identical characters.
    /// If codemode is true, returns the raw line without stripping.
    pub fn nl(&mut self, withcount: bool, codemode: bool) -> Option<(String, usize)> {
        if self.pos >= self.lines.len() {
            if !self.file_stack.is_empty() {
                self.next_file();
                return self.nl(withcount, codemode);
            }
            return None;
        }

        let mut s = self.lines[self.pos].clone();
        self.pos += 1;
        self.linenum += 1;

        if codemode {
            // Return raw line
            return Some((s, 0));
        }

        // Strip leading whitespace
        s = s.trim_start_matches(|c: char| c == ' ' || c == '\t').to_string();

        // Remove trailing comments
        let trimmed = remove_trailing_comment(&s);
        s = format!("{}\n", trimmed);

        let count = if withcount && s.len() > 1 {
            let first_char = s.chars().next().unwrap_or('\n');
            if first_char == '\n' {
                0
            } else {
                s.chars().take_while(|&c| c == first_char).count()
            }
        } else {
            0
        };

        // Strip leading marker characters (-, ., =, :)
        s = s.trim_start_matches(|c: char| c == '-' || c == '.' || c == '=' || c == ':')
            .to_string();

        Some((s, count))
    }

    /// Get the next paragraph from the input file.
    /// Reads lines until a paragraph-break signal is encountered.
    pub fn np(&mut self, withcount: bool, eatblanks: bool) -> Option<(String, usize)> {
        let (mut s, c) = match self.nl(withcount, false) {
            Some(v) => v,
            None => return None,
        };

        // Detect open inline equation blocks
        let dollar_re = Regex::new(r"(?<!\\)\$").unwrap();
        let match_count = dollar_re
            .find_iter(&s)
            .count();
        let mut lm = match_count % 2;
        let mut is_open_eq = lm == 1;

        // Characters that signal a new paragraph
        let nl_signals = ["\n", ".", ":", "", "=", "~", "{", "\\(", "\\)"];

        loop {
            let pcf = self.pc(true);
            if pcf.is_empty() {
                break;
            }

            let is_signal = nl_signals.contains(&pcf.as_str());

            if !is_open_eq {
                // Not in open equation: break on - or nl_signals
                if pcf == "-" || is_signal {
                    break;
                }
            } else {
                // In open equation: break only on nl_signals (allow -)
                if is_signal {
                    break;
                }
                if pcf == "-" {
                    s.push('-');
                }
            }

            let ns = match self.nl(false, false) {
                Some((line, _)) => line,
                None => break,
            };

            let ns_matches = dollar_re.find_iter(&ns).count();
            lm = (lm + ns_matches) % 2;
            is_open_eq = lm == 1;
            s.push_str(&ns);
        }

        // Eat blank lines
        if eatblanks {
            while self.pc(true) == "\n" {
                self.nl(false, false); // burn blank line
            }
        }

        Some((s, c))
    }

    // =========================================================================
    // Block replacement (calls text::br with parser state)
    // =========================================================================

    /// Perform block replacements using parser state.
    pub fn br(&mut self, b: &str, tableblock: bool) -> String {
        br(b, self.eqs, tableblock, &mut self.tablerow)
    }

    // =========================================================================
    // List processing
    // =========================================================================

    /// Process a dash-list (unordered) or dot-list (ordered).
    pub fn dashlist(&mut self, ordered: bool) {
        let mut level = 0usize;
        let (char_marker, ul_tag) = if ordered {
            (".", "ol")
        } else {
            ("-", "ul")
        };

        while self.pc(true) == char_marker {
            let (s, newlevel) = match self.np(true, false) {
                Some(v) => v,
                None => break,
            };

            if newlevel > level {
                for _ in 0..(newlevel - level) {
                    if newlevel > 1 {
                        self.out("\n");
                    }
                    self.out(&format!("<{}>\n<li>", ul_tag));
                }
            } else if newlevel < level {
                self.out("\n</li>");
                for _ in 0..(level - newlevel) {
                    self.out(&format!("</{}>\n</li>", ul_tag));
                }
                self.out("\n<li>");
            } else {
                self.out("\n</li>\n<li>");
            }

            let processed = self.br(&s, false);
            let processed = mathjax_eq_resub(&processed);
            self.out(&format!("<p>{}</p>", processed));
            level = newlevel;
        }

        for _ in 0..level {
            self.out(&format!("\n</li>\n</{}>\n", ul_tag));
        }
    }

    /// Process a colon-list (definition list).
    pub fn colonlist(&mut self) {
        let re = Regex::new(r"(?ms)\s*\{(.*?)(?<!\\)\}(.*)").unwrap();
        self.out("<dl>\n");
        while self.pc(true) == ":" {
            let (s, _) = match self.np(false, false) {
                Some(v) => v,
                None => break,
            };
            if let Ok(Some(caps)) = re.captures(&s) {
                let defpart = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let rest = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                let def_html = self.br(defpart, false);
                self.hb("<dt>|</dt>\n", &def_html, None, None);
                let rest_html = self.br(rest, false);
                self.hb("<dd><p>|</p></dd>\n", &rest_html, None, None);
            }
        }
        self.out("</dl>\n");
    }

    // =========================================================================
    // Code block processing
    // =========================================================================

    /// Process a code block (delimited by ~).
    pub fn codeblock(&mut self, title: Option<&str>, lang: &str) {
        if lang == "raw" {
            // Raw mode: output verbatim
            loop {
                let line = match self.nl(false, true) {
                    Some((l, _)) => l,
                    None => break,
                };
                if line.starts_with('~') {
                    break;
                }
                let line = if line.starts_with("\\~") {
                    line[1..].to_string()
                } else {
                    line
                };
                self.out(&line);
            }
            return;
        }

        if title == Some("filter_through") {
            // Filter through external program (simplified - just output raw).
            // In the Python version, lang is the external command name.
            let mut buff = String::new();
            loop {
                let line = match self.nl(false, true) {
                    Some((l, _)) => l,
                    None => break,
                };
                if line.starts_with('~') {
                    break;
                }
                buff.push_str(&line);
            }
            // In the Python version this pipes through an external program.
            // For now we just output the raw buffer.
            self.out(&buff);
            return;
        }

        // Normal code block
        let codeblock_conf = self.conf("codeblock");
        let blocktitle_conf = self.conf("blocktitle");
        let codeblockcontenttt_conf = self.conf("codeblockcontenttt");
        let codeblockcontent_conf = self.conf("codeblockcontent");
        let codeblockendtt_conf = self.conf("codeblockendtt");
        let codeblockend_conf = self.conf("codeblockend");

        self.out(&codeblock_conf);
        if let Some(t) = title {
            if !t.is_empty() {
                self.hb(&blocktitle_conf, t, None, None);
            }
        }
        if lang == "jemdoc" {
            self.out(&codeblockcontenttt_conf);
        } else {
            self.out(&codeblockcontent_conf);
        }

        let mut stringmode = false;

        loop {
            let line = match self.nl(false, true) {
                Some((l, _)) => l,
                None => break,
            };

            if line.starts_with('~') {
                break;
            }

            let line = if line.starts_with("\\~") {
                line[1..].to_string()
            } else if line.starts_with("\\{") {
                line[1..].to_string()
            } else {
                line
            };

            if stringmode {
                if line.trim_end().ends_with("\"\"\"") {
                    self.out(&format!("{}</span>", line));
                    stringmode = false;
                } else {
                    self.out(&line);
                }
                continue;
            }

            if lang == "pyint" {
                let formatted = format_pyint(&line);
                self.out(&formatted);
            } else if lang == "jemdoc" {
                let ltrimmed = line.trim_start();
                let special_starts = ["#", "~", ">>>", "\\~", "{"];
                let mut handled = false;
                for prefix in &special_starts {
                    if ltrimmed.starts_with(prefix) {
                        self.out("</tt><pre class=\"tthl\">");
                        self.out(&line);
                        self.out("</pre><tt class=\"tthl\">");
                        handled = true;
                        break;
                    }
                }
                if !handled {
                    let colon_starts = [":", ".", "-"];
                    let mut handled2 = false;
                    for prefix in &colon_starts {
                        if ltrimmed.starts_with(prefix) {
                            self.out(&format!("<br />{}", prepend_nbsps(&line)));
                            handled2 = true;
                            break;
                        }
                    }
                    if !handled2 {
                        if ltrimmed.starts_with('=') {
                            self.out(&format!("{}<br />", prepend_nbsps(&line)));
                        } else {
                            self.out(&line);
                        }
                    }
                }
            } else {
                // Check for includes in code blocks
                if line.starts_with("\\#include{") || line.starts_with("\\#includeraw{") {
                    self.out(&line[1..]);
                } else if line.starts_with('#') && self.do_includes(&line[1..]) {
                    continue;
                } else if (lang == "python" || lang == "py")
                    && line.trim().starts_with("\"\"\"")
                {
                    self.out(&format!("<span class=\"string\">{}", line));
                    stringmode = true;
                } else {
                    let hl = get_hl(lang);
                    let formatted = format_language(&line, &hl);
                    self.out(&formatted);
                }
            }
        }

        if lang == "jemdoc" {
            self.out(&codeblockendtt_conf);
        } else {
            self.out(&codeblockend_conf);
        }
    }

    // =========================================================================
    // Title insertion
    // =========================================================================

    /// Insert the document title.
    pub fn insert_title(&mut self, title: Option<&str>) {
        if let Some(t) = title {
            let doctitle_conf = self.conf("doctitle");
            self.hb(&doctitle_conf, t, None, None);

            // Look for a subtitle
            if self.pc(true) != "\n" {
                let (subtitle_text, _) = self.np(false, true).unwrap_or_default();
                let processed = self.br(&subtitle_text, false);
                let subtitle_conf = self.conf("subtitle");
                self.hb(&subtitle_conf, &processed, None, None);
            }

            let doctitleend_conf = self.conf("doctitleend");
            self.hb(&doctitleend_conf, t, None, None);
        }
    }

    // =========================================================================
    // Menu insertion
    // =========================================================================

    /// Insert menu items from a MENU file.
    pub fn insert_menu_items(&mut self, mname: &str, current: &str, prefix: &str) {
        // Resolve the MENU file path relative to the input file's directory.
        let menu_path = if std::path::Path::new(mname).is_absolute() {
            std::path::PathBuf::from(mname)
        } else {
            let indir = std::path::Path::new(&self.inname)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            indir.join(mname)
        };
        let menu_content = match fs::read_to_string(&menu_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let re = Regex::new(r"\s*(.*?)\s*\[(.*)\]").unwrap();
        let currentmenuitem_conf = self.conf("currentmenuitem");
        let menuitem_conf = self.conf("menuitem");
        let menucategory_conf = self.conf("menucategory");

        for line in menu_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Ok(Some(caps)) = re.captures(line) {
                // Menu item
                let mut link = caps.get(2).unwrap().as_str().to_string();
                let menu_text = caps.get(1).unwrap().as_str();

                let option = if link.starts_with('\\') {
                    link = link[1..].to_string();
                    " target=\"blank\""
                } else {
                    ""
                };

                // Don't use prefix for absolute links
                if !link.contains("://") {
                    link = format!("{}{}", prefix, allreplace(&link));
                }

                // Replace spaces with &nbsp; (except in {{ }} blocks)
                let mut menuitem = String::new();
                for group in menu_text.split("{{").flat_map(|s| {
                    let mut v: Vec<(&str, bool)> = Vec::new();
                    let parts: Vec<&str> = s.splitn(2, "}}").collect();
                    if parts.len() == 2 {
                        v.push((parts[0], true)); // inside {{ }}
                        v.push((parts[1], false));
                    } else {
                        v.push((parts[0], false));
                    }
                    v
                }) {
                    if group.1 {
                        menuitem.push_str(group.0);
                    } else {
                        // Collapse one or more spaces into a single ~ (→ &nbsp;),
                        // matching the Python original's re.sub(r' +', '~', ...).
                        let replaced = re_replace_all(r"(?<!\\n) +", group.0, "~");
                        let processed = self.br(&replaced, false);
                        menuitem.push_str(&processed);
                    }
                }

                if link.ends_with(current) {
                    self.hb(
                        &currentmenuitem_conf,
                        &link,
                        Some(&menuitem),
                        Some(option),
                    );
                } else {
                    self.hb(
                        &menuitem_conf,
                        &link,
                        Some(&menuitem),
                        Some(option),
                    );
                }
            } else {
                // Menu category
                let processed = self.br(line, false);
                self.hb(&menucategory_conf, &processed, None, None);
            }
        }
    }

    // =========================================================================
    // Main file processing
    // =========================================================================

    /// Process the jemdoc input file and generate HTML output.
    pub fn proc_file(&mut self) {
        self.linenum = 0;

        let mut menu: Option<(String, String, String)> = None; // (menufile, current, prefix)
        let mut show_footer = true;
        let mut show_source_link = false;
        let mut show_last_updated = true;
        let mut show_last_updated_time = true;
        let mut no_default_css = false;
        let mut fwtitle = false;
        let mut css: Vec<String> = Vec::new();
        let mut js: Vec<String> = Vec::new();
        let mut title: Option<String> = None;

        // Parse header directives (lines starting with #)
        while self.pc(false) == "#" {
            let line = match self.nl(false, true) {
                Some((l, _)) => l,
                None => break,
            };
            let line = line.trim().to_string();

            // Check for includes
            if line.starts_with('#') && self.do_includes(line[1..].trim()) {
                continue;
            }

            if line.starts_with("# jemdoc:") {
                let directives = &line["# jemdoc:".len()..];
                let re_braces = Regex::new(r"(?ms)(?<!\\)\{(.*?)(?<!\\)\}").unwrap();

                for directive in directives.split(',') {
                    let d = directive.trim();

                    if d.starts_with("menu") {
                        let mut g: Vec<String> = Vec::new();
                        let mut ss = 0;
                        while let Ok(Some(caps)) = re_braces.captures_from_pos(d, ss) {
                            let m = caps.get(0).unwrap();
                            g.push(caps.get(1).unwrap().as_str().to_string());
                            ss = m.end();
                        }
                        if g.len() >= 2 {
                            let prefix = if g.len() >= 3 {
                                g[2].clone()
                            } else {
                                String::new()
                            };
                            menu = Some((g[0].clone(), g[1].clone(), prefix));
                        }
                    } else if d.starts_with("nofooter") {
                        show_footer = false;
                    } else if d.starts_with("nodate") {
                        show_last_updated = false;
                    } else if d.starts_with("notime") {
                        show_last_updated_time = false;
                    } else if d.starts_with("fwtitle") {
                        fwtitle = true;
                    } else if d.starts_with("showsource") {
                        show_source_link = true;
                    } else if d.starts_with("nodefaultcss") {
                        no_default_css = true;
                    } else if d.starts_with("addcss") {
                        let mut ss = 0;
                        while let Ok(Some(caps)) = re_braces.captures_from_pos(d, ss) {
                            let m = caps.get(0).unwrap();
                            css.push(caps.get(1).unwrap().as_str().to_string());
                            ss = m.end();
                        }
                    } else if d.starts_with("addjs") {
                        let mut ss = 0;
                        while let Ok(Some(caps)) = re_braces.captures_from_pos(d, ss) {
                            let m = caps.get(0).unwrap();
                            js.push(caps.get(1).unwrap().as_str().to_string());
                            ss = m.end();
                        }
                    } else if d.starts_with("analytics") {
                        let ss = 0;
                        if let Ok(Some(caps)) = re_braces.captures_from_pos(d, ss) {
                            self.analytics =
                                Some(caps.get(1).unwrap().as_str().to_string());
                        }
                    } else if d.starts_with("title") {
                        let ss = 0;
                        if let Ok(Some(caps)) = re_braces.captures_from_pos(d, ss) {
                            title = Some(caps.get(1).unwrap().as_str().to_string());
                        }
                    } else if d.starts_with("noeqs") {
                        self.eqs = false;
                    }
                }
            }
        }

        // Output the first bit of HTML
        let firstbit = self.conf("firstbit");
        self.out(&firstbit);

        if !no_default_css {
            let defaultcss = self.conf("defaultcss");
            self.out(&defaultcss);
        }

        // Add per-file CSS
        let specificcss = self.conf("specificcss");
        for c in &mut css {
            if !c.contains(".css") {
                c.push_str(".css");
            }
            self.hb(&specificcss, c, None, None);
        }

        // Add per-file JS
        let specificjs = self.conf("specificjs");
        for j in &js {
            self.hb(&specificjs, j, None, None);
        }

        // Look for a title (line starting with =)
        let t = if self.pc(true) == "=" {
            let (title_line, _) = self.nl(false, false).unwrap_or_default();
            let mut processed = self.br(&title_line, false);
            // Remove trailing \n
            if processed.ends_with('\n') {
                processed.pop();
            }
            if title.is_none() {
                title = Some(
                    re_replace_all(r" *(<br />)|(&nbsp;) *", &processed, " ").to_string(),
                );
            }
            Some(processed)
        } else {
            None
        };

        // Window title
        let window_title = title.clone().unwrap_or_default();
        let windowtitle_conf = self.conf("windowtitle");
        self.hb(&windowtitle_conf, &window_title, None, None);

        // MathJax (always injected, independently of bodystart so user conf overrides don't lose it)
        let mathjax = self.conf("mathjax");
        self.out(&mathjax);

        // Body start
        let bodystart = self.conf("bodystart");
        self.out(&bodystart);

        // Analytics
        if let Some(analytics_id) = self.analytics.clone() {
            let analytics_conf = self.conf("analytics");
            self.hb(&analytics_conf, &analytics_id, None, None);
        }

        // Full-width title
        if fwtitle {
            let fwtitlestart = self.conf("fwtitlestart");
            self.out(&fwtitlestart);
            self.insert_title(t.as_deref());
            let fwtitleend = self.conf("fwtitleend");
            self.out(&fwtitleend);
        }

        // Menu
        if let Some((mname, current, prefix)) = &menu {
            let menustart = self.conf("menustart");
            self.out(&menustart);
            self.insert_menu_items(mname, current, prefix);
            let menuend = self.conf("menuend");
            self.out(&menuend);
        } else {
            let nomenu = self.conf("nomenu");
            self.out(&nomenu);
        }

        // Insert title (if not full-width)
        if !fwtitle {
            self.insert_title(t.as_deref());
        }

        // Main content processing loop
        let mut infoblock = false;
        let mut imgblock = false;
        let mut tableblock = false;

        loop {
            let p = self.pc(true);

            if p.is_empty() {
                break;
            }

            if p == "\\(" {
                // Whole-line equation
                if !self.eqs {
                    break;
                }
                let (mut s, _) = self.nl(false, false).unwrap_or_default();

                // Check if equation is single-line
                if !s.trim().ends_with("\\)") {
                    loop {
                        let line = match self.nl(false, true) {
                            Some((l, _)) => l,
                            None => break,
                        };
                        s.push_str(&line);
                        if line.trim() == "\\)" {
                            break;
                        }
                    }
                }

                let r = self.br(s.trim(), false);
                let r = mathjax_eq_resub(&r);
                self.out(&r);
            } else if p == "-" {
                // Unordered list
                self.dashlist(false);
            } else if p == "." {
                // Ordered list
                self.dashlist(true);
            } else if p == ":" {
                // Definition list
                self.colonlist();
            } else if p == "=" {
                // Heading
                let (s, c) = self.nl(true, false).unwrap_or_default();
                let s = s.trim_end_matches('\n');
                let processed = self.br(s, false);
                let heading = format!("<h{}>|</h{}>\n", c, c);
                self.hb(&heading, &processed, None, None);
            } else if p == "#" {
                // Comment (already consumed by pc)
                let _ = self.nl(false, false);
            } else if p == "\n" {
                // Blank line
                let _ = self.nl(false, false);
            } else if p == "~" {
                // Block delimiter
                let _ = self.nl(false, false);

                if infoblock {
                    let infoblockend = self.conf("infoblockend");
                    self.out(&infoblockend);
                    infoblock = false;
                    let _ = self.nl(false, false);
                    continue;
                } else if imgblock {
                    self.out("</td></tr></table>\n");
                    imgblock = false;
                    let _ = self.nl(false, false);
                    continue;
                } else if tableblock {
                    self.out("</td></tr></table>\n");
                    tableblock = false;
                    let _ = self.nl(false, false);
                    continue;
                } else {
                    // Start a new block
                    let mut g: Vec<String> = Vec::new();

                    if self.pc(true) == "{" {
                        let l = match self.nl(false, false) {
                            Some((line, _)) => allreplace(&line),
                            None => String::new(),
                        };
                        let re_braces = Regex::new(r"(?ms)(?<!\\)\{(.*?)(?<!\\)\}").unwrap();
                        let mut ss = 0;
                        while let Ok(Some(caps)) = re_braces.captures_from_pos(&l, ss) {
                            let m = caps.get(0).unwrap();
                            g.push(caps.get(1).unwrap().as_str().to_string());
                            ss = m.end();
                        }
                    }

                    // Process jemdoc markup in first group (title)
                    if !g.is_empty() {
                        let processed = self.br(&g[0], false);
                        g[0] = processed;
                    }

                    if g.is_empty() || g.len() == 1 {
                        // Info block
                        let infoblock_conf = self.conf("infoblock");
                        self.out(&infoblock_conf);
                        infoblock = true;

                        if g.len() == 1 {
                            let blocktitle_conf = self.conf("blocktitle");
                            self.hb(&blocktitle_conf, &g[0], None, None);
                        }

                        let infoblockcontent_conf = self.conf("infoblockcontent");
                        self.out(&infoblockcontent_conf);
                    } else if g.len() >= 2 && g[1] == "table" {
                        // Table block
                        let name = if g.len() >= 3 && !g[2].is_empty() {
                            format!(" id=\"{}\"", g[2])
                        } else {
                            String::new()
                        };
                        self.out(&format!(
                            "<table{}>\n<tr class=\"r1\"><td class=\"c1\">",
                            name
                        ));
                        self.tablerow = 1;
                        self.tablecol = 1;
                        tableblock = true;
                    } else if g.len() == 2 {
                        // Code block
                        let title = if g[0].is_empty() {
                            None
                        } else {
                            Some(g[0].as_str())
                        };
                        let lang = g[1].clone();
                        self.codeblock(title, &lang);
                    } else if g.len() >= 4 && g[1] == "img_left" {
                        // Image block
                        let mut g = g;
                        while g.len() < 7 {
                            g.push(String::new());
                        }

                        if g[4].chars().all(|c| c.is_ascii_digit()) && !g[4].is_empty() {
                            g[4] = format!("{}px", g[4]);
                        }
                        if g[5].chars().all(|c| c.is_ascii_digit()) && !g[5].is_empty() {
                            g[5] = format!("{}px", g[5]);
                        }

                        self.out("<table class=\"imgtable\"><tr><td>\n");
                        if !g[6].is_empty() {
                            self.out(&format!("<a href=\"{}\">", g[6]));
                        }
                        self.out(&format!("<img src=\"{}\"", g[2]));
                        self.out(&format!(" alt=\"{}\"", g[3]));
                        if !g[4].is_empty() {
                            self.out(&format!(" width=\"{}\"", g[4]));
                        }
                        if !g[5].is_empty() {
                            self.out(&format!(" height=\"{}\"", g[5]));
                        }
                        self.out(" />");
                        if !g[6].is_empty() {
                            self.out("</a>");
                        }
                        self.out("&nbsp;</td>\n<td align=\"left\">");
                        imgblock = true;
                    }
                }
            } else {
                // Regular paragraph
                let (s, _) = match self.np(false, true) {
                    Some(v) => v,
                    None => break,
                };

                if !s.is_empty() {
                    let processed = self.br(&s, tableblock);
                    if tableblock {
                        self.hb("|\n", &processed, None, None);
                    } else {
                        self.hb("<p>|</p>\n", &processed, None, None);
                    }
                }
            }
        }

        // Footer
        if show_footer && (show_last_updated || show_source_link) {
            let footerstart = self.conf("footerstart");
            self.out(&footerstart);

            if show_last_updated {
                let s = {
                    let fmt = if show_last_updated_time {
                        "%Y-%m-%d %H:%M:%S %Z\0"
                    } else {
                        "%Y-%m-%d\0"
                    };
                    unsafe {
                        let mut t: libc::time_t = 0;
                        libc::time(&mut t);
                        let tm = libc::localtime(&t);
                        let mut buf = [0u8; 128];
                        let len = libc::strftime(
                            buf.as_mut_ptr() as *mut libc::c_char,
                            buf.len(),
                            fmt.as_ptr() as *const libc::c_char,
                            tm,
                        );
                        if len > 0 {
                            String::from_utf8_lossy(&buf[..len]).to_string()
                        } else {
                            let now = chrono::Local::now();
                            now.format("%Y-%m-%d %H:%M:%S").to_string()
                        }
                    }
                };
                let lastupdated = self.conf("lastupdated");
                self.hb(&lastupdated, &s, None, None);
            }

            if show_source_link {
                let inname = self.inname.clone();
                let sourcelink = self.conf("sourcelink");
                self.hb(&sourcelink, &inname, None, None);
            }

            let footerend = self.conf("footerend");
            self.out(&footerend);
        }

        // Close menu/layout
        if menu.is_some() {
            let menulastbit = self.conf("menulastbit");
            self.out(&menulastbit);
        } else {
            let nomenulastbit = self.conf("nomenulastbit");
            self.out(&nomenulastbit);
        }

        // Body end
        let bodyend = self.conf("bodyend");
        self.out(&bodyend);

        // Flush output
        let _ = self.outf.flush();
    }
}
