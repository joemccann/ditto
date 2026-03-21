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
- syntax highlighting for fenced code blocks
- links
- local images
- remote images with caching
- tables
- horizontal rules
- pagination
- table of contents generation
- improved math / LaTeX support
- raw HTML-in-Markdown handling strategy

This is now a strong Rust-native foundation, but it still does **not** fully satisfy the original goal of supporting every Markdown feature and full GitHub Flavored Markdown fidelity.

## Remaining work

### 1. GFM fidelity polishing

Current state:
- Core GFM features are implemented.
- Fidelity still needs polishing for edge cases and nesting behavior.

TODO:
- [ ] Improve nested list indentation and spacing fidelity
- [ ] Improve ordered list numbering behavior in complex nesting
- [ ] Polish task list layout and spacing
- [ ] Respect GFM table alignment markers (`:---`, `:---:`, `---:`)
- [ ] Verify autolink behavior against GFM expectations
- [ ] Verify footnote support end-to-end
- [ ] Verify definition list support end-to-end
- [ ] Add representative GFM compatibility fixtures

### 2. Table of contents improvements

Current state:
- TOC is generated
- It is not yet fully document-aware

TODO:
- [ ] Generate page-numbered TOC
- [ ] Add internal clickable navigation
- [ ] Add CLI option to enable/disable TOC
- [ ] Add CLI option for TOC depth (`--toc-depth`)

### 3. Image pipeline polish

Current state:
- Local and remote images work
- Remote images are cached
- Image behavior still needs refinement

TODO:
- [ ] Improve remote content-type / extension detection
- [ ] Improve SVG edge-case handling
- [ ] Improve image sizing heuristics
- [ ] Improve caption and alt-text rendering
- [ ] Add missing-image fallback rendering
- [ ] Add cache invalidation / refresh policy
- [ ] Add image-specific tests

### 4. Typography and layout polish

Current state:
- Good baseline layout exists
- Styling and pagination heuristics need refinement

TODO:
- [ ] Add configurable body font family
- [ ] Add configurable monospace font family
- [ ] Add theme/preset support
- [ ] Improve page-break heuristics
- [ ] Improve spacing around headings, lists, code blocks, and tables
- [ ] Add print presets (A4, Letter, etc.)

### 5. CLI and product polish

Current state:
- Core CLI works
- Several user-facing controls are still missing

TODO:
- [ ] Add `--font-family`
- [ ] Add `--mono-font-family`
- [ ] Add `--theme`
- [ ] Add `--toc` / `--no-toc`
- [ ] Add `--toc-depth`
- [ ] Add `--no-remote-images`
- [ ] Add `--cache-dir`
- [ ] Add `--self-check` or doctor mode
- [ ] Improve help output and examples

### 6. Testing and quality

Current state:
- Manual validation done
- Core automated quality coverage still missing

TODO:
- [ ] Add unit tests for Markdown -> Typst conversion helpers
- [ ] Add integration tests for representative Markdown inputs
- [ ] Add snapshot tests for generated Typst
- [ ] Add PDF smoke tests
- [ ] Add regression fixtures for tricky GFM cases
- [ ] Add representative sample documents with expected outputs

## Priority order

Recommended implementation order:

1. [ ] GFM fidelity polishing
2. [ ] TOC with page numbers and internal links
3. [ ] Image pipeline polish
4. [ ] CLI and typography polish
5. [ ] Test coverage and regression protection

## Goal reminder

Original target:

- support any type of markdown
- support every feature of markdown
- support GitHub Flavored Markdown
- support images
- produce PDF from a Rust CLI

Current project is much closer, but still not yet at full fidelity for that goal.
