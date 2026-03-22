# Architecture

## Overview

ditto converts Markdown to PDF in two stages:

1. **Markdown → Typst source** — walk the comrak AST and emit Typst markup
2. **Typst source → PDF** — compile in-process via the `typst` crate and write the PDF bytes

There is no intermediate HTML, no external process, and no runtime dependency on Python or a browser.

## Data flow

```
Input Markdown
      │
      ▼
  comrak parser
  (CommonMark + GFM extensions)
      │
      ▼
  TypstRenderer  ──────────────────────────────┐
  (src/renderer.rs)                            │
      │                                        │
      ├── Text / inline nodes                  │
      ├── Headings (label + equals markers)    │
      ├── Lists (bullet / ordered / task)      │
      ├── Code blocks ──→ highlighter.rs       │
      ├── Math (LaTeX → Typst)                 │
      ├── Tables (with alignment)              │
      ├── Images ──→ image resolution          │
      │              local / remote / data-URI │
      ├── Raw HTML ──→ html.rs                 │
      └── TOC (Typst #outline())               │
                                               │
      ▼                                        │
  Typst source string ◄────────────────────────┘
      │
      ▼
  TypstWorld (World impl)
  Font discovery via typst-kit
      │
      ▼
  typst::compile::<PagedDocument>()
      │
      ▼
  typst_pdf::pdf()
      │
      ▼
  PDF bytes written to output file
```

## Module responsibilities

### `src/main.rs`

CLI entry point. Parses arguments via Clap, dispatches to `--doctor` or the render pipeline. No rendering logic.

### `src/cli.rs`

All Clap argument definitions: `Cli` struct, `Preset` enum, flag documentation, and examples. Preset resolution (mm values) lives here.

### `src/lib.rs`

Thin module re-export surface used by integration tests. Only exposes what tests need.

### `src/renderer.rs`

The core conversion module. Walks the comrak `AstNode` tree and emits Typst markup.

Key types:

- `RenderConfig` — all conversion settings derived from CLI args
- `TypstRenderer` — stateful walker; maintains list nesting, footnote accumulation, heading label dedup
- `TypstWorld` — implements `typst::World`; serves source files and font data to the compiler
- `Fonts` — loaded once via `typst-kit`; reused for every compile

Key behaviours:

- Headings emit `<label>` anchors for TOC clickability
- Duplicate heading text gets deduplicated labels (`<overview>`, `<overview-2>`, ...)
- Footnotes are accumulated during the walk and emitted as a block at the end
- Code blocks are delegated to `highlighter.rs`
- Raw HTML nodes are delegated to `html.rs`
- Remote images are downloaded to a cache dir; local and data-URI images are resolved directly

### `src/highlighter.rs`

Converts a fenced code block into Typst styled content using Syntect.

- `SyntaxSet` and `ThemeSet` are loaded once via `OnceLock` and reused for every block
- Each token becomes a `#text(fill: rgb("…"), weight: …, style: …)[…]` span
- Unknown/empty languages fall back to plain `#raw(...)` which is safe for all characters
- Near-white theme backgrounds are nudged to `#f6f8fa` for visibility on white paper

### `src/html.rs`

Translates raw HTML embedded in Markdown into Typst markup.

Two strategies:

- **Inline HTML** (`HtmlInline` AST nodes): stateful tag stack on `TypstRenderer`. Open tags push a prefix/suffix pair; close tags pop and emit the suffix.
- **Block HTML** (`HtmlBlock` AST nodes): full recursive tokeniser and renderer in `html.rs`.

Supported tags and CSS are documented in the README.

### `src/doctor.rs`

Implements `--doctor` mode. Runs a series of named checks and prints pass/fail/warning for each:

- Typst engine (compiles a trivial document)
- Font availability (body + mono)
- Cache directory writability
- Network reachability

## Testing

```
tests/
  unit_renderer.rs           Pure unit tests for helper functions
  integration_md_to_typst.rs Round-trip Markdown → Typst string assertions
  typst_snapshots.rs         Golden file snapshots for Typst output
  pdf_smoke.rs               End-to-end PDF compilation (uses real Typst compile)
  image_pipeline.rs          Image resolution, format detection, caching, fallbacks
  fixtures/
    samples/                 Representative Markdown input files
    snapshots/               Golden .typ snapshot files
```

Tests do not depend on external services. Remote image tests use stubs or are skipped with `--no-remote-images`.

## Dependencies

| Crate | Role |
|---|---|
| `comrak` | CommonMark + GFM parsing |
| `typst` | Typst language, layout engine |
| `typst-pdf` | PDF exporter |
| `typst-kit` | Font discovery |
| `typst-syntax` | `FileId`, `Source`, `VirtualPath` types |
| `syntect` | Syntax highlighting |
| `ureq` | HTTP client for remote image downloads |
| `clap` | CLI argument parsing |
| `anyhow` | Error handling |
