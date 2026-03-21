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
- [ ] Add true language-aware syntax highlighting
- [ ] Use `syntect` or equivalent to tokenize code blocks
- [ ] Map token styles into Typst-renderable styled spans
- [ ] Add fallback behavior for unknown languages
- [ ] Support inline code styling consistency with fenced blocks

### 2. Math / LaTeX support

Current state:
- Basic passthrough-ish handling only
- Complex LaTeX is not robustly supported

TODO:
- [ ] Convert inline math `$...$` into Typst-native math where possible
- [ ] Convert display math `$$...$$` into Typst display math blocks
- [ ] Handle escaping and nested delimiters correctly
- [ ] Document unsupported LaTeX constructs
- [ ] Add tests for common math expressions

### 3. Raw HTML in Markdown

Current state:
- Raw HTML blocks are not faithfully rendered like a browser
- HTML support is incomplete

TODO:
- [ ] Decide on HTML strategy: ignore, sanitize, partial map, or render subset
- [ ] Support common inline HTML tags (`span`, `br`, `sub`, `sup`, etc.)
- [ ] Support common block HTML tags (`div`, `img`, `table`, `details`, etc.)
- [ ] Sanitize unsafe/unsupported HTML
- [ ] Add tests for mixed Markdown + HTML documents

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
3. [ ] Raw HTML rendering strategy
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
