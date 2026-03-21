# TODO

## Current status

Implemented foundation:

- headings
- paragraphs
- lists
- task lists
- blockquotes
- inline code
- code blocks
- links
- local images
- remote images
- tables
- horizontal rules
- pagination
- table of contents generation

This is a solid Rust-native foundation, but it does **not** yet fully satisfy the original goal of supporting every Markdown feature, full GitHub Flavored Markdown fidelity, and arbitrary Markdown/HTML content.

## Remaining work

### 1. Syntax highlighting

Current state:
- Code blocks are visually styled.
- Language labels are shown.
- No token-level syntax coloring yet.

TODO:
- [x] Add true language-aware syntax highlighting
- [x] Use `syntect` to tokenize code blocks (SyntaxSet + ThemeSet via OnceLock)
- [x] Map token styles into Typst `#text(fill: rgb(...), weight, style)[...]` spans
- [x] Add fallback behavior for unknown languages (uses `#raw(...)` for safe rendering)
- [ ] Support inline code styling consistency with fenced blocks

### 2. Math / LaTeX support

Current state:
- `$...$` inline and `$$...$$` display math parsing enabled via comrak `math_dollars`
- ` ```math ``` ` fenced code blocks and `$`...`$` backtick syntax supported
- Full `latex_to_typst` translator converts common LaTeX to Typst native math:
  - Fractions: `\frac`, `\dfrac`, `\cfrac`
  - Roots: `\sqrt`, `\sqrt[n]{...}`
  - Greek letters (lower and uppercase): `\alpha`…`\Omega`
  - Operators: `\pm`, `\leq`, `\geq`, `\neq`, `\approx`, `\cdot`, `\times`, `\div`…
  - Trig / log: `\sin`, `\cos`, `\tan`, `\ln`, `\log`, `\lim`, `\sum`, `\int`…
  - Arrows: `\to`, `\rightarrow`, `\Rightarrow`, `\mapsto`…
  - Accents: `\hat`, `\tilde`, `\bar`, `\vec`, `\dot`, `\ddot`…
  - Text in math: `\text{…}`, `\mathrm{…}`, `\mathbf{…}`, `\mathbb{…}`
  - Matrices: `pmatrix`, `bmatrix`, `Bmatrix`, `vmatrix`, `Vmatrix`, `matrix`
  - Piecewise functions: `cases` environment
  - Aligned equations: `align`, `align*`, `aligned`
  - Delimiters: `\left`, `\right`, `\langle`, `\rangle`, `\lfloor`, `\rfloor`…
  - Misc: `\partial`, `\nabla`, `\infty`, `\hbar`, `\forall`, `\exists`…

Known limitations:
- Multi-letter implicit multiplication (`ac`, `dx`) must use spaces in LaTeX source
  (Typst treats multi-letter identifiers as named variables, not implied products)
- Obscure LaTeX packages / environments not yet mapped

TODO:
- [x] Convert inline math `$...$` into Typst-native math where possible
- [x] Convert display math `$$...$$` into Typst display math blocks
- [x] Handle escaping and nested delimiters correctly
- [x] Document unsupported LaTeX constructs
- [ ] Add comprehensive automated tests for math expressions
- [ ] Consider auto-spacing single-letter sequences (e.g. `dx` → `d x`)

### 3. Raw HTML in Markdown

Current state:
- ✅ Inline HTML tags are parsed and mapped to Typst equivalents via stateful stack
- ✅ Block HTML tags are parsed and mapped to Typst block constructs
- ✅ HTML entities are decoded before rendering
- ✅ CSS `color` and `font-size` on `<span>` are converted to Typst text attributes
- ✅ Unsupported/unknown tags are silently stripped (content preserved)
- ✅ `src/html.rs` module with 44 unit tests covering all supported tags

Supported inline tags: `<br>`, `<wbr>`, `<b>`, `<strong>`, `<i>`, `<em>`, `<u>`,
`<s>`, `<del>`, `<ins>`, `<mark>`, `<small>`, `<sub>`, `<sup>`, `<code>`,
`<kbd>`, `<samp>`, `<var>`, `<span style="color:…;font-size:…">`, `<a href="…">`,
`<abbr>`, `<cite>`, `<dfn>`, `<q>`, `<time>`, `<data>`

Supported block tags: `<p>`, `<div>`, `<section>`, `<article>`, `<main>`,
`<header>`, `<footer>`, `<nav>`, `<aside>`, `<blockquote>`, `<hr>`, `<pre>`,
`<ul>`, `<ol>`, `<li>`, `<dl>`, `<dt>`, `<dd>`, `<img>`, `<figure>`,
`<figcaption>`, `<table>`, `<thead>`, `<tbody>`, `<tfoot>`, `<tr>`, `<th>`,
`<td>`, `<details>`, `<summary>`, `<h1>`–`<h6>`

TODO (future improvements):
- [ ] Inline style `background-color` → highlight
- [ ] `<iframe>`, `<video>`, `<audio>` — fallback placeholder text
- [ ] Full table alignment from `align=` or CSS `text-align`
- [ ] `<ruby>` / `<rt>` phonetic annotation

### 4. GitHub Flavored Markdown fidelity

Current state:
- Many GFM features work
- Fidelity is not yet complete

TODO:
- [ ] Improve nested list rendering
- [ ] Improve task list spacing and indentation
- [ ] Respect GFM table alignment markers
- [ ] Support autolinks more faithfully
- [ ] Verify footnote support end-to-end
- [ ] Verify definition list support end-to-end
- [ ] Add compatibility tests against representative GFM samples

### 5. Images

Current state:
- Local images work
- Remote images download and cache
- Image sizing/caption behavior is basic

TODO:
- [ ] Improve remote image content-type / extension detection
- [ ] Handle SVG and edge-case image formats more robustly
- [ ] Improve image scaling heuristics
- [ ] Improve caption and alt-text rendering
- [ ] Add missing-image fallback rendering
- [ ] Add cache invalidation / refresh policy
- [ ] Add tests for local, remote, raster, and SVG images

### 6. Table of contents

Current state:
- TOC is generated
- No page numbers
- No internal navigation support

TODO:
- [ ] Generate page-numbered TOC
- [ ] Add internal clickable navigation
- [ ] Optionally allow TOC enable/disable via CLI flag
- [ ] Optionally support depth limits (`--toc-depth`)

### 7. Typography and layout polish

Current state:
- Basic page and text configuration exists
- Limited styling control

TODO:
- [ ] Add configurable body font family
- [ ] Add configurable monospace font family
- [ ] Add theme/preset support
- [ ] Improve page-break heuristics
- [ ] Add widow/orphan control where possible
- [ ] Improve spacing around headings, tables, lists, and code blocks
- [ ] Add support for print presets (A4, Letter, etc.)

### 8. CLI and product polish

Current state:
- Core CLI exists
- Limited user-facing controls

TODO:
- [ ] Add `--font-family`
- [ ] Add `--mono-font-family`
- [ ] Add `--theme`
- [ ] Add `--toc` / `--no-toc`
- [ ] Add `--toc-depth`
- [ ] Add `--no-remote-images`
- [ ] Add `--cache-dir`
- [ ] Add `--self-check` or `doctor` mode
- [ ] Improve help output and examples

### 9. Testing and quality

Current state:
- Manual testing done
- No comprehensive automated test coverage yet

TODO:
- [ ] Add unit tests for Markdown -> Typst conversion helpers
- [ ] Add integration tests for representative Markdown inputs
- [ ] Add snapshot tests for generated Typst
- [ ] Add PDF smoke tests
- [ ] Add regression fixtures for tricky GFM cases
- [ ] Add sample documents with expected outputs

## Priority order

Recommended implementation order:

1. [ ] Real syntax highlighting
2. [ ] Better math support
3. [x] Raw HTML rendering strategy — done (`src/html.rs`, 44 tests)
4. [ ] Better GFM fidelity for edge cases
5. [ ] TOC with page numbers and internal links
6. [ ] CLI polish and test coverage

## Goal reminder

Original target:

- support any type of markdown
- support every feature of markdown
- support GitHub Flavored Markdown
- support images
- produce PDF from a Rust CLI

Current project is **on the way**, but not yet at full fidelity for that goal.
