# TODO

## Current status

Implemented foundation:

- headings (ATX and setext)
- paragraphs
- lists (bullet and ordered)
- nested lists
- ordered list numbering with non-1 starts
- task lists
- blockquotes
- inline code
- code blocks with language tags
- syntax highlighting for fenced code blocks (100+ languages via Syntect)
- links, autolinks, reference-style links
- local raster images (PNG, JPEG, GIF, WebP, BMP, TIFF, AVIF)
- local SVG images
- remote images with HTTP caching (ETag / Last-Modified)
- data-URI images (`data:image/png;base64,…`)
- tables with per-column alignment
- horizontal rules
- strikethrough
- footnotes
- definition / description lists
- GitHub Alerts (`> [!NOTE]`, `> [!TIP]`, `> [!IMPORTANT]`, `> [!WARNING]`, `> [!CAUTION]`)
- superscript (`^text^`) and subscript (`~text~`)
- underline (`__text__`)
- hard line breaks (two trailing spaces → `\\\n`)
- loose list vertical spacing (`#v(0.5em)` between loose items)
- pagination
- table of contents generation (via Typst `#outline()`)
- page-numbered TOC with real Typst page numbers
- clickable TOC links (via heading `<label>` anchors)
- TOC enable/disable support
- TOC depth control
- TOC title from front matter
- duplicate heading label disambiguation
- improved math / LaTeX support
- raw HTML-in-Markdown translation (inline and block)
- CLI font configuration
- CLI theme support
- remote image controls (`--no-remote-images`, `--cache-dir`)
- doctor / self-check support (`--doctor`)
- presets for page / layout sizes (a4, letter, a5, legal, slides)
- YAML front-matter parsing (toc, toc_depth, toc_title, no_toc)
- stdin input (`-`)
- broad automated test coverage

Current automated validation:

- unit tests (renderer helpers, HTML translator, math converter)
- integration tests (Markdown → Typst round-trips)
- Typst snapshot tests (golden .typ files)
- PDF smoke tests (end-to-end Typst compile)
- image pipeline tests (format detection, caching, fallbacks)
- regression fixtures

Latest known status:

- **725 tests passing**
- **0 failures**

This is a strong Rust-native Markdown-to-PDF CLI with broad feature support.
Known incompatibilities are documented in [docs/known-limitations.md](known-limitations.md).

---

## Remaining work

### 1. HTML fidelity improvements

Current state:
- Raw HTML-in-Markdown has a real handling strategy with broad tag support.
- Known gaps are documented in `docs/known-limitations.md` §1.
- `<table>` in HTML blocks uses a fixed single-column layout.
- `<ol start="…">` in HTML blocks does not honour the `start` attribute.
- Only `color` and `font-size` CSS properties are translated on `<span>`.

TODO:
- [ ] Fix `<table>` column counting in HTML blocks to match actual cell count
- [ ] Honour `<ol start="…">` in HTML blocks
- [ ] Expand `<span>` CSS support to include `background-color` and `font-weight`
- [ ] Add support for more named CSS colours beyond the current 10

### 2. Math fidelity improvements

Current state:
- Math support covers the most common LaTeX constructs.
- Known unsupported commands are documented in `docs/known-limitations.md` §2.
- Unknown commands are passed through as `\cmd`, which may produce Typst
  errors visible in stderr.

TODO:
- [ ] Add `\not` (negation prefix) support
- [ ] Add `\mathcal`, `\mathfrak`, `\mathscr` font commands
- [ ] Add `\cancel`, `\boxed`, `\overset`, `\underset`
- [ ] Add `\xleftarrow`, `\xrightarrow`
- [ ] Add `\bigoplus`, `\bigotimes`, `\bigsqcup` and other large operators
- [ ] Add support for `array` environment with column format specifier
- [ ] Add more regression fixtures for edge-case math input

### 3. Front-matter improvements

Current state:
- Only `toc`, `toc_depth`, `toc_title`, and `no_toc` are parsed.
- All other YAML keys are silently ignored.

TODO:
- [ ] Support `title`, `author`, `date` front-matter keys for PDF metadata
- [ ] Support `lang` key for Typst `#set text(lang: …)`
- [ ] Consider full YAML parsing instead of the current hand-rolled parser

### 4. Image and asset polish

Current state:
- Local and remote images work with caching and fallbacks.
- Known limitations are documented in `docs/known-limitations.md` §5.

TODO:
- [ ] Add Markdown-extension image sizing syntax support (`{width=50%}`)
- [ ] Improve remote image cache invalidation strategy
- [ ] Support `<img>` remote downloads inside raw HTML blocks
- [ ] Improve fallback handling for corrupted or partially-downloaded images

### 5. Typography and visual polish

Current state:
- Typography controls exist.
- Layout presets exist.
- Output quality is good.

TODO:
- [ ] Add more theme / preset variants if desired
- [ ] Improve spacing / pagination heuristics for especially dense documents
- [ ] Add `dir: rtl` / bidirectional text support
- [ ] Add page-number offset control

### 6. Packaging and productization

Current state:
- CLI works and is fully tested.
- Known limitations are documented.

TODO:
- [ ] Add end-user documentation for themes, fonts, TOC, remote images, and
  doctor mode (possibly a `docs/user-guide.md`)
- [ ] Add release / build instructions for distribution
- [ ] Consider CI automation for tests and release artifacts
- [ ] Add `--watch` flag for live re-render during editing
- [ ] Add PDF metadata flags (`--title`, `--author`, `--date`)

---

## Priority order

Recommended next order:

1. [ ] Fix HTML block `<table>` column counting
2. [ ] Honour `<ol start="…">` in HTML blocks
3. [ ] Add key missing LaTeX commands (`\not`, `\mathcal`, `\cancel`, `\boxed`)
4. [ ] Front-matter: `title` / `author` / `date` → PDF metadata
5. [ ] Image sizing syntax (`{width=…}`)
6. [ ] Documentation / productization

---

## Goal reminder

Original target:

- support any type of markdown
- support every feature of markdown
- support GitHub Flavored Markdown
- support images
- produce PDF from a Rust CLI

Current project is **close to complete** for practical use. Remaining gaps are
catalogued in [`docs/known-limitations.md`](known-limitations.md) so users know
exactly what to expect. No known correctness regressions.
