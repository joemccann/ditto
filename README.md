# Ditto

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/dark.svg">
    <source media="(prefers-color-scheme: light)" srcset=".github/light.svg">
    <img alt="Ditto Logo" src=".github/light.svg" width="100%">
  </picture>
</p>

<p align="center" width="80%">A fast, pure-Rust CLI that converts Markdown (CommonMark + GitHub Flavored Markdown) to PDF via the <a href="https://typst.app">Typst</a> engine.</p>

## Features

- ✅ CommonMark + GFM (tables, strikethrough, task lists, autolinks, footnotes, definition lists, GitHub Alerts, superscript, subscript, underline)
- ✅ Fenced code blocks with Syntect syntax highlighting
- ✅ LaTeX math — `$…$` inline and `$$…$$` display, plus ` ```math ``` ` blocks
- ✅ Remote image caching (http/https → local cache)
- ✅ Data-URI images (`data:image/png;base64,…`)
- ✅ Auto-generated Table of Contents (via Typst's `#outline()` — real page numbers)
- ✅ Named page presets: a4, letter, a5, legal, slides
- ✅ Custom body and monospace font families
- ✅ Doctor / self-check mode (`--doctor`)
- ✅ Read from stdin (`-`)

## Installation

### Quick install

```bash
./install.sh
```

Builds the release binary and symlinks it to `/usr/local/bin/ditto` so it's available globally.

### Manual install

```bash
cargo build --release
# Binary is at target/release/ditto
```

### Via cargo

```bash
cargo install --path .
```

## Usage

```
ditto [OPTIONS] <INPUT> <OUTPUT>
ditto --doctor
```

### Quick examples

```bash
# Basic conversion
ditto README.md README.pdf

# US Letter, 14 pt, custom fonts
ditto --preset letter --font-size 14 \
          --font-family "EB Garamond" --mono-font-family "Fira Code" \
          report.md report.pdf

# Slide deck (16:9)
ditto --preset slides --no-toc slides.md slides.pdf

# TOC — H1 and H2 only
ditto --toc --toc-depth 2 big-doc.md big-doc.pdf

# Dark code theme, skip remote images
ditto --syntax-theme "base16-ocean.dark" --no-remote-images doc.md doc.pdf

# Self-check
ditto --doctor
```

## Options

### Page layout

| Flag | Default | Description |
|------|---------|-------------|
| `--preset` | `a4` | Named layout preset (see table below) |
| `-w, --page-width MM` | preset | Page width in mm (overrides preset) |
| `--page-height MM` | preset | Page height in mm (overrides preset) |
| `-m, --margin MM` | preset | All-sides margin in mm (overrides preset) |

#### Presets

| Preset | Width × Height | Margin | Notes |
|--------|----------------|--------|-------|
| `a4` | 210 × 297 mm | 20 mm | ISO A4 portrait (default) |
| `letter` | 216 × 279 mm | 20 mm | US Letter portrait |
| `a5` | 148 × 210 mm | 15 mm | A5 pocket |
| `legal` | 216 × 356 mm | 20 mm | US Legal |
| `slides` | 338 × 190 mm | 12 mm | 16 : 9 presentation deck |

### Typography

| Flag | Default | Description |
|------|---------|-------------|
| `-f, --font-size PT` | `12.0` | Base body font size in points |
| `--font-family FAMILY` | `"Libertinus Serif"` | Body font family |
| `--mono-font-family FAMILY` | `"DejaVu Sans Mono"` | Monospace font (code blocks) |
| `--syntax-theme THEME` | `InspiredGitHub` | Syntect code-highlighting theme |

**Built-in syntax themes:** `InspiredGitHub`, `base16-ocean.dark`, `base16-ocean.light`,
`base16-eighties.dark`, `base16-mocha.dark`, `Solarized (dark)`, `Solarized (light)`

### Table of contents

| Flag | Default | Description |
|------|---------|-------------|
| `--toc` | off | Prepend a TOC page |
| `--no-toc` | — | Suppress TOC (explicit override) |
| `--toc-depth DEPTH` | `6` | Maximum heading depth in TOC (1–6) |

TOC entries are real (Typst `#outline()` with page numbers and clickable links).  
You can also enable/override the TOC from YAML front matter:

```yaml
---
toc: true
toc_depth: 3
---
```

### Image handling

| Flag | Default | Description |
|------|---------|-------------|
| `--no-remote-images` | off | Skip http/https image downloads |
| `--cache-dir DIR` | `.ditto-cache/` | Remote-image cache location |

### Doctor / self-check

```bash
ditto --doctor
```

Runs diagnostics:
- ✅ Typst engine round-trip compile
- ✅/⚠️ Default body and mono font availability
- ✅ Cache directory writability
- ✅/⚠️ Network reachability (for remote images)
- ✅ Rust toolchain info

Exits with code `0` on pass, `1` if any check fails.

## Frontmatter overrides

YAML front matter (delimited by `---`) can override `toc` and `toc_depth`:

```markdown
---
toc: true
toc_depth: 2
---
# My Document
```

## Development

```bash
cargo build --release   # release binary → target/release/ditto
cargo test              # full test suite (725+ tests)
cargo clippy            # lint
cargo fmt               # format
cargo run -- test.md output.pdf
```

## Known limitations

Some HTML tags, LaTeX commands, and edge-case Markdown constructs are not fully
supported. See **[docs/known-limitations.md](docs/known-limitations.md)** for a
comprehensive list, including:

- Which HTML tags are translated and which are stripped
- Which CSS properties on `<span>` are honoured (`color` and `font-size` only)
- Which LaTeX math commands have no Typst equivalent
- GFM features that are unsupported (wikilinks, custom heading IDs, etc.)
- Front-matter keys that are recognised (`toc`, `toc_depth`, `toc_title`, `no_toc` only)
- Image and typography constraints

## License

MIT OR Apache-2.0
