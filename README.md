# jemdoc-rs

A Rust rewrite of [jemdoc+MathJax](https://github.com/wsshin/jemdoc_mathjax), a light text-based markup language for creating static websites.

`jemdoc-rs` converts `.jemdoc` source files into clean, standards-compliant HTML5 pages with built-in support for MathJax 4.

## Installation

### Using `cargo install`

Requires [Rust](https://www.rust-lang.org/tools/install) 1.85 or later.

```sh
cargo install --git https://github.com/haozhu10015/jemdoc-rs.git
```

This installs `jemdoc-rs` to `~/.cargo/bin/`, which is typically already on your `PATH`. You only need to do this once (or re-run to update).

### Building from Source

```sh
git clone https://github.com/haozhu10015/jemdoc-rs.git
cd jemdoc-rs
cargo build --release
```

The binary will be at `target/release/jemdoc-rs`. You can copy it to any directory on your `PATH`.

## Quick Start

```sh
# Convert a single file (produces index.html from index.jemdoc)
jemdoc-rs index

# With a configuration file
jemdoc-rs -c mysite.conf index.jemdoc

# Multiple files
jemdoc-rs -c mysite.conf *.jemdoc
```

## Usage

```
jemdoc-rs [OPTIONS] [SOURCEFILE...]
```

| Option | Description |
|---|---|
| `-c <file>` | Use a configuration file (can be specified multiple times) |
| `-o <file>` | Write output to a specific file or directory |
| `--show-config` | Print the default configuration template |
| `--version` | Show version information |
| `--help`, `-h` | Show help message |

## Example

The `example/` directory contains a set of `.jemdoc` files demonstrating the full feature set. To generate and preview them:

```sh
cd example

# Generate all HTML pages
make jemdoc

# Preview locally at http://127.0.0.1:8000
make preview
```

`make jemdoc` runs `jemdoc-rs -c mysite.conf *.jemdoc` and writes the HTML output in place. `make preview` starts a local HTTP server using [`simple-http-server`](https://github.com/TheWaWaR/simple-http-server) (installed automatically via Cargo if not already present). Run `make help` to see all available targets.

## Markup Reference

### Titles and Headings

```
= Page Title
== Section
=== Subsection
```

### Text Formatting

| Markup | Result |
|---|---|
| `*bold*` | **bold** |
| `/italic/` | *italic* |
| `_underline_` | <u>underline</u> |
| `+monospace+` | `monospace` |
| `~` | non-breaking space |
| `---` | em dash |
| `--` | en dash |
| `...` | ellipsis |

### Equations (MathJax)

MathJax 4 is always included. Inline math uses `$...$` and display math uses `\(...\)`:

```
Inline equation: $E = mc^2$.

Display equation:
\(
\nabla \times \mathbf{E} = -\frac{\partial\mathbf{B}}{\partial t}
\)
```

Numbered equations work with LaTeX `equation`, `align`, and `\label`/`\eqref` commands.

### Links

```
[http://example.com]                  # URL as link text, opens in new tab
[http://example.com Example Site]     # Custom link text, opens in new tab
[/localpage.html Local Page]          # Leading / opens in the same tab
[user@example.com]                    # Mailto link
```

### Images

```
[img{width}{height}{alt text} path/to/image.png Caption]
```

### Lists

```
- Unordered item 1
- Unordered item 2

. Ordered item 1
. Ordered item 2

: {Term} Definition
: {Another term} Another definition
```

Nested lists use repeated markers (`--`, `...`).

### Code Blocks

````
~~~
{Title}{language}
def hello():
    print("world")
~~~
````

Supported languages for syntax highlighting: `python`/`py`, `c`, `c++`/`cpp`, `ruby`/`rb`, `sh`, `matlab`, `commented`, `jemdoc`, `pyint`.

Use `{}{raw}` to output content verbatim (no highlighting or wrapping).

### Info Blocks

```
~~~
{Optional Title}
Block content here.
~~~
```

### Tables

````
~~~
{}{table}{optional-id}
Cell 1 | Cell 2 | Cell 3
Row 2  | Cell 5 | Cell 6
~~~
````

Use `||` for row breaks within a cell.

### Image Blocks

````
~~~
{}{img_left}{photo.jpg}{alt text}{width}{height}{link}
Description text goes here.
~~~
````

### Raw HTML

Embed raw HTML with double braces: `{{<span style="color:red">red</span>}}`.

### Comments

Lines starting with `#` are comments and are not included in the output.

```
# This is a comment
```

## Header Directives

Directives are placed at the top of a `.jemdoc` file as special comments:

```
# jemdoc: menu{MENU}{thispage.html}
# jemdoc: addcss{custom.css}
# jemdoc: title{Custom Window Title}
```

| Directive | Description |
|---|---|
| `menu{MENUFILE}{current.html}{prefix}` | Add a navigation menu (prefix is optional) |
| `addcss{file.css}` | Include an additional CSS file |
| `addjs{file}` | Include an additional JS file |
| `analytics{G-XXXXXXXXXX}` | Add Google Analytics (GA4) tracking |
| `title{Window Title}` | Override the browser window title |
| `nodefaultcss` | Don't include the default CSS |
| `nofooter` | Hide the page footer |
| `nodate` | Hide the "last updated" date |
| `notime` | Show date only (no time) in footer |
| `noeqs` | Disable equation processing and MathJax loading |
| `fwtitle` | Use full-width title (above the menu) |
| `showsource` | Add a link to the `.jemdoc` source file |

## Menu Files

A `MENU` file defines the navigation sidebar:

```
Menu Category
    Page One        [page1.html]
    Page Two        [page2.html]

Another Category
    External Link   [\http://example.com]
```

- Indented lines with `[link]` are menu items.
- Non-indented lines are category headers.
- Prefix a link with `\` to open it in a new tab.

## Configuration

jemdoc-rs uses a default HTML template that can be customized. Run `jemdoc-rs --show-config` to see all configurable sections. Override any section in a `.conf` file:

```ini
# mysite.conf — override the body start to add a favicon
[bodystart]
</head>
<link rel="icon" href="img/icon.png" type="image/x-icon" />
<body>
```

Use `-c mysite.conf` to apply the overrides. Multiple `-c` flags can be used; later files take precedence.
