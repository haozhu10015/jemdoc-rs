# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## About

`jemdoc-rs` is a Rust rewrite of [jemdoc+MathJax](https://github.com/wsshin/jemdoc_mathjax). It converts `.jemdoc` markup files into HTML5 pages with built-in MathJax 4 support.

## Commands

```sh
# Build
cargo build
cargo build --release          # binary at target/release/jemdoc-rs

# Run
cargo run -- index             # converts index.jemdoc -> index.html
cargo run -- -c site.conf index.jemdoc

# Test
cargo test
cargo test test_name           # run a single test by name
cargo test -- --nocapture      # show stdout from tests

# Check / Lint
cargo check
cargo clippy
```

## Architecture

All source is in `src/` across five files:

### `main.rs`
CLI entrypoint. Parses flags (`-c`, `-o`), loads config via `parse_conf()`, reads each `.jemdoc` input file, and drives `JemdocParser::proc_file()`.

### `config.rs`
Defines the default HTML template as `standard_conf()` — a multi-section INI-like format where each `[sectionname]` block is a snippet of HTML. `parse_conf()` merges standard config with any user-supplied `.conf` files, returning a `HashMap<String, String>`. Sections are retrieved at render time and used as templates with `|` placeholders.

### `jemdoc.rs`
The core `JemdocParser` struct and its state machine. `proc_file()` is the main loop:
1. Reads header directives (`# jemdoc: menu{...}, addcss{...}`, etc.)
2. Emits HTML boilerplate from config sections
3. Dispatches each block type: headings (`=`), lists (`-`, `.`, `:`), code/info/table/image blocks (`~~~`), display equations (`\(...\)`), and regular paragraphs
4. Emits footer and closing HTML

File inclusion (`include{file}` / `includeraw{file}`) is handled via a `file_stack` — the parser pushes its current position and switches to processing the included file, then pops back.

### `text.rs`
The `br()` function is the inline markup pipeline. It processes a text block in sequence:
1. Environment variable substitution (`!$VAR$!`)
2. Equation protection: `$...$` → `\(...\)`, `\(...\)` → `\[...\]` (with placeholder escaping)
3. HTML entity escaping (`&`, `<`, `>`)
4. Image replacement (`[img{w}{h}{alt} path]`)
5. `%code%` → monospace
6. Link replacement (`[url text]`)
7. Inline formatting: `/italic/`, `*bold*`, `_underline_`, `+mono+`
8. Typography: `---`, `--`, `...`, `~` (non-breaking space)
9. Table cell splitting (`|`, `||`)

**MathJax placeholder pattern**: Special characters inside equations are temporarily replaced with named tokens (e.g., `BACKSLASH65358`, `UNDERSCORE65358`) before HTML processing so they survive entity escaping, then restored via `mathjax_eq_resub()`.

**`hb_format()`**: Template substitution helper. Replaces `|` with content1, or `|1`/`|2`/`|3` with multiple content arguments.

`fancy_regex` is used instead of the standard `regex` crate for lookbehind support — backslash-escaped markup (e.g., `\*`, `\_`) is skipped using `(?<!\\)` lookbehinds.

### `highlight.rs`
Syntax highlighting for code blocks. `get_hl(lang)` returns a `HighlightDef` with keyword lists per category (`statement`, `builtin`, `operator`, `special`, `error`, `commentuntilend`). `format_language()` applies each category as a regex replacement wrapping matched tokens in `<span class="...">`.

## Config Template System

The config system uses `[sectionname]` blocks. To override a section in a `.conf` file:

```ini
[bodystart]
</head>
<link rel="icon" href="img/icon.png" type="image/x-icon" />
<body>
```

Run `cargo run -- --show-config` to see all available sections with their defaults.
