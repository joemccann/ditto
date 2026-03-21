# md-to-pdf

A fast, pure-Rust CLI that converts Markdown (CommonMark + GitHub Flavored Markdown) to PDF via the [Typst](https://typst.app) engine.

## Features

- ✅ Full CommonMark + GFM support (tables, strikethrough, task lists, autolinks, footnotes)
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

```bash
cargo install --path .
```

## Usage

```
md-to-pdf [OPTIONS] <INPUT> <OUTPUT>
md-to-pdf --doctor
```

### Quick examples

```bash
# Basic conversion
md-to-pdf README.md README.pdf

# US Letter, 14 pt, custom fonts
md-to-pdf --preset letter --font-size 14 \
          --font-family "EB Garamond" --mono-font-family "Fira Code" \
          report.md report.pdf

# Slide deck (16:9)
md-to-pdf --preset slides --no-toc slides.md slides.pdf

# TOC — H1 and H2 only
md-to-pdf --toc --toc-depth 2 big-doc.md big-doc.pdf

# Dark code theme, skip remote images
md-to-pdf --syntax-theme "base16-ocean.dark" --no-remote-images doc.md doc.pdf

# Self-check
md-to-pdf --doctor
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
| `--cache-dir DIR` | `.md-to-pdf-cache/` | Remote-image cache location |

### Doctor / self-check

```bash
md-to-pdf --doctor
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
cargo build --release   # release binary → target/release/md-to-pdf
cargo test              # run 44 unit tests
cargo run -- test.md output.pdf
```

## License

MIT OR Apache-2.0
