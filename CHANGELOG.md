# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] — 2025-03-21

### Added

- Pure Rust Markdown-to-PDF CLI via the Typst engine
- CommonMark support via comrak
- GitHub Flavored Markdown extensions:
  - Tables with alignment markers
  - Strikethrough
  - Task lists
  - Autolinks
  - Footnotes
  - Definition lists
  - GitHub Alert blocks (Note, Tip, Important, Warning, Caution)
  - Superscript, subscript, underline
- Syntax highlighting for fenced code blocks via syntect (100+ languages, 7 themes)
- LaTeX math support (`$...$` inline, `$$...$$` display, ` ```math ``` ` blocks)
- LaTeX-to-Typst math translation (Greek letters, fractions, roots, matrices, operators, accents, environments)
- Local and remote image embedding with automatic caching
- Data URI image support (`data:image/png;base64,...`)
- SVG image support
- Auto-generated Table of Contents via Typst `#outline()` with real page numbers and clickable links
- Named page presets: a4, letter, a5, legal, slides
- Custom body and monospace font families
- Syntax theme selection
- Raw HTML-in-Markdown handling (25+ inline tags, 30+ block tags)
- YAML front matter support (`toc`, `toc_depth`, `toc_title`, `no_toc`)
- Doctor / self-check mode (`--doctor`)
- Read from stdin (`-`)
- 725+ automated tests (unit, integration, snapshot, PDF smoke)
