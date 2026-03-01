use std::collections::HashMap;
use std::fs;

/// Returns the default jemdoc HTML template configuration.
pub fn standard_conf() -> String {
    let raw = r#"[firstbit]
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<meta name="generator" content="jemdoc-rs, see https://github.com/haozhu10015/jemdoc-rs" />

[defaultcss]
<link rel="stylesheet" href="jemdoc.css" type="text/css" />

[windowtitle]
<title>|</title>

[fwtitlestart]
<div id="fwtitle">

[fwtitleend]
</div>

[doctitle]
<div id="toptitle">
<h1>|</h1>

[subtitle]
<div id="subtitle">|</div>

[doctitleend]
</div>

[mathjax]
<!-- MathJax -->
<script>
MathJax = {
  tex: {
    inlineMath: [['\\(','\\)']],
    displayMath: [['\\[','\\]']],
    tags: 'ams'
  }
};
</script>
<script id="MathJax-script" async src="https://cdn.jsdelivr.net/npm/mathjax@4/tex-mml-chtml.js">
</script>
<!-- End MathJax -->

[bodystart]
</head>
<body>

[analytics]
<!-- Google tag (gtag.js) -->
<script async src="https://www.googletagmanager.com/gtag/js?id=|"></script>
<script>
window.dataLayer = window.dataLayer || [];
function gtag(){dataLayer.push(arguments);}
gtag('js', new Date());
gtag('config', '|');
</script>
<!-- End Google tag (gtag.js) -->

[menustart]
<table summary="Table for page layout." id="tlayout">
<tr valign="top">
<td id="layout-menu">

[menuend]
</td>
<td id="layout-content">

[menucategory]
<div class="menu-category">|</div>

[menuitem]
<div class="menu-item"><a href="|1"|3>|2</a></div>

[specificcss]
<link rel="stylesheet" href="|" type="text/css" />

[specificjs]
<script src="|.js" type="text/javascript"></script>

[currentmenuitem]
<div class="menu-item"><a href="|1" class="current"|3>|2</a></div>

[nomenu]
<div id="layout-content">

[menulastbit]
</td>
</tr>
</table>

[nomenulastbit]
</div>

[bodyend]
</body>
</html>

[infoblock]
<div class="infoblock">

[codeblock]
<div class="codeblock">

[blocktitle]
<div class="blocktitle">|</div>

[infoblockcontent]
<div class="blockcontent">

[codeblockcontent]
<div class="blockcontent"><pre>

[codeblockend]
</pre></div></div>

[codeblockcontenttt]
<div class="blockcontent"><tt class="tthl">

[codeblockendtt]
</tt></div></div>

[infoblockend]
</div></div>

[footerstart]
<div id="footer">
<div id="footer-text">

[footerend]
</div>
</div>

[lastupdated]
Page generated |, by <a href="https://github.com/haozhu10015/jemdoc-rs" target="blank">jemdoc-rs</a>.

[sourcelink]
(<a href="|">source</a>)

"#;
    raw.to_string()
}

/// Read a line from content lines, skipping comment lines (lines starting with #).
fn read_noncomment(lines: &[&str], pos: &mut usize) -> Option<String> {
    while *pos < lines.len() {
        let line = lines[*pos];
        *pos += 1;
        if line.starts_with('#') {
            continue;
        }
        // Return the line trimmed with just one \n
        return Some(format!("{}\n", line.trim_end()));
    }
    None
}

/// Parse configuration from the standard config + any user-provided config files.
/// Returns a HashMap mapping section tags to their content.
pub fn parse_conf(confnames: &[String]) -> HashMap<String, String> {
    let mut syntax = HashMap::new();

    // Parse standard config first
    parse_conf_content(&standard_conf(), &mut syntax);

    // Parse user config files
    for name in confnames {
        if let Ok(content) = fs::read_to_string(name) {
            parse_conf_content(&content, &mut syntax);
        }
    }

    syntax
}

/// Parse configuration content string into the syntax HashMap.
fn parse_conf_content(content: &str, syntax: &mut HashMap<String, String>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut pos = 0;

    while pos < lines.len() {
        let line = match read_noncomment(&lines, &mut pos) {
            Some(l) => l,
            None => break,
        };

        // Look for [tag] header
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.contains(']') {
            if let Some(end) = trimmed.find(']') {
                let tag = &trimmed[1..end];
                let mut section_content = String::new();

                // Read content lines until empty line or EOF
                loop {
                    let cline = match read_noncomment(&lines, &mut pos) {
                        Some(l) => l,
                        None => break,
                    };
                    if cline.trim().is_empty() {
                        break;
                    }
                    section_content.push_str(&cline);
                }

                syntax.insert(tag.to_string(), section_content);
            }
        }
    }
}

/// Print the standard config (for --show-config).
pub fn show_config() {
    println!("{}", standard_conf());
}
