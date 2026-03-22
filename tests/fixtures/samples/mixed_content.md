# Mixed Content Document

A representative document exercising many features together in realistic prose.

---

## Project Overview

This is a **Rust**-based CLI tool that converts *Markdown* to PDF via the
[Typst](https://typst.app) typesetting engine.  Key features:

- 🦀 Written in **pure Rust** — no external runtime dependencies
- 📄 Outputs press-quality PDFs
- ✅ Supports [GitHub Flavored Markdown](https://github.github.com/gfm/)
- 🔢 Native LaTeX-style math rendering

---

## Installation

```bash
cargo install ditto
```

Or build from source:

```bash
git clone https://github.com/example/ditto
cd ditto
cargo build --release
```

---

## Usage

### Basic conversion

```
ditto input.md output.pdf
```

### With options

| Option | Default | Description |
|:-------|:-------:|:------------|
| `--preset` | `a4` | Page size preset |
| `--toc` | off | Generate table of contents |
| `--margin` | `20` | Margin in mm |
| `--font-size` | `12` | Body font size in pt |

---

## Supported Math

Inline: $m c^2$ and $\alpha + \beta = \gamma$.

Display block:

$$\int_{-\infty}^{\infty} e^{-x^2} = \sqrt{\pi}$$

Matrix example:

$$\begin{pmatrix} 1 & 0 \\ 0 & 1 \end{pmatrix}$$

---

## Changelog

1. Initial release — basic Markdown support
2. Added GFM tables and task lists
3. Added math rendering via Typst native math
4. Added remote image caching
5. Added table of contents generation

---

## Known Limitations

> Some LaTeX math constructs may not translate perfectly.  When in doubt,
> test with `--debug` to inspect the generated Typst source.

---

## Footnotes

The Typst engine[^typst] is used for compilation.

The PDF standard[^pdf] has been around since 1993.

[^typst]: Typst is available at https://typst.app — a modern typesetting system.
[^pdf]: PDF stands for Portable Document Format, developed by Adobe.
