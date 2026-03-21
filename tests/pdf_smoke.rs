//! PDF smoke tests.
//!
//! These tests invoke the full pipeline: Markdown → Typst source → compiled
//! PDF written to a temp file.  They verify:
//!
//! 1. The conversion succeeds without errors.
//! 2. The output file exists and has non-zero size.
//! 3. The output starts with the PDF magic bytes (`%PDF`).
//! 4. The `RenderSummary` reports a sane page count.
//!
//! Each test uses `--no-remote-images` so they work offline.
//!
//! Note: These tests are inherently slower than unit/integration tests because
//! they spin up the embedded Typst compiler.  They are tagged `#[test]` so
//! they run as part of `cargo test`, but can be excluded with
//! `cargo test --test integration_md_to_typst --test typst_snapshots`.

use md_to_pdf::renderer::{FontSet, RenderConfig, render_markdown_to_pdf};
use tempfile::TempDir;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn smoke_config(dir: &TempDir) -> RenderConfig {
    RenderConfig {
        page_width_mm: 210.0,
        page_height_mm: 297.0,
        margin_mm: 20.0,
        base_font_size_pt: 12.0,
        fonts: FontSet::default(),
        input_path: Some(dir.path().join("test.md")),
        syntax_theme: "InspiredGitHub".to_string(),
        toc: false,
        toc_explicit: false,
        toc_depth: 6,
        no_remote_images: true,
        cache_dir_override: Some(dir.path().join("cache")),
    }
}

/// Assert the file at `path` is a valid PDF (≥ 4 bytes, starts with `%PDF`).
fn assert_is_pdf(path: &std::path::Path) {
    let bytes = std::fs::read(path)
        .unwrap_or_else(|_| panic!("PDF file not found: {}", path.display()));
    assert!(
        bytes.len() > 4,
        "PDF too small ({} bytes): {}",
        bytes.len(),
        path.display()
    );
    assert_eq!(
        &bytes[..4],
        b"%PDF",
        "File does not start with PDF magic bytes: {}",
        path.display()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Smoke tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn smoke_minimal_document() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let summary = render_markdown_to_pdf("Hello, world!\n", &out, smoke_config(&dir))
        .expect("render should succeed");
    assert_is_pdf(&out);
    assert!(summary.pages >= 1, "expected at least 1 page, got {}", summary.pages);
}

#[test]
fn smoke_all_headings() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\n\nSome content.\n";
    let summary = render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("render should succeed");
    assert_is_pdf(&out);
    assert!(summary.pages >= 1);
    // Should have extracted 6 TOC entries
    assert_eq!(summary.toc_entries.len(), 6,
        "expected 6 TOC entries, got {}", summary.toc_entries.len());
}

#[test]
fn smoke_bullet_list() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "- Alpha\n- Beta\n  - Nested\n- Gamma\n";
    let summary = render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("render should succeed");
    assert_is_pdf(&out);
    assert!(summary.pages >= 1);
}

#[test]
fn smoke_ordered_list() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "1. First\n2. Second\n3. Third\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_task_list() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "- [x] Done\n- [ ] Pending\n- [x] Also done\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_table() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "| Name | Score |\n|------|-------|\n| Alice | 95 |\n| Bob | 82 |\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_table_alignment() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "| Left | Center | Right |\n|:-----|:------:|------:|\n| a | b | c |\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_code_block_rust() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "```rust\nfn main() {\n    println!(\"Hello!\");\n}\n```\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_code_block_python() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "```python\ndef greet(name):\n    return f\"Hello, {name}!\"\n```\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_code_block_no_lang() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "```\nPlain text code block.\n```\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_inline_math() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    // Use simple math that translates cleanly to Typst
    let md = "The value is $x = \\frac{a}{b}$ and $y = \\sqrt{z}$.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_display_math() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    // Use well-supported display math
    let md = "$$\\sum_{n=1}^{\\infty} \\frac{1}{n^2} = \\frac{\\pi^2}{6}$$\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_math_matrix() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "$$\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}$$\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_math_cases() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "$$f(x) = \\begin{cases} x^2 & \\text{if } x \\geq 0 \\\\ -x & \\text{otherwise} \\end{cases}$$\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_blockquote() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "> \"The only way to do great work is to love what you do.\"\n>\n> — Steve Jobs\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_footnotes() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "First[^1] and second[^2] references.\n\n[^1]: First footnote.\n\n[^2]: Second footnote.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_with_toc() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Chapter 1\n\n## Section 1.1\n\nContent here.\n\n## Section 1.2\n\nMore content.\n\n# Chapter 2\n\nFinal chapter.\n";
    let mut config = smoke_config(&dir);
    config.toc = true;
    config.toc_explicit = true;
    config.toc_depth = 3;
    let summary = render_markdown_to_pdf(md, &out, config).expect("render should succeed");
    assert_is_pdf(&out);
    // TOC + body = at least 2 pages (TOC page + content page)
    assert!(summary.pages >= 2, "expected at least 2 pages with TOC, got {}", summary.pages);
}

#[test]
fn smoke_toc_custom_title() {
    // Verify that a custom toc_title from frontmatter compiles without error.
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "---\ntoc: true\ntoc_depth: 2\ntoc_title: My Index\n---\n\
              # Chapter 1\n\nContent.\n\n## Section 1.1\n\nMore.\n\n# Chapter 2\n\nFinal.\n";
    let config = smoke_config(&dir);
    render_markdown_to_pdf(md, &out, config).expect("toc_title smoke should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_toc_no_toc_frontmatter() {
    // `no_toc: true` in frontmatter must suppress the TOC even when the default
    // config would emit one.
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "---\nno_toc: true\n---\n# Hello\n## World\n\nContent.\n";
    let mut config = smoke_config(&dir);
    config.toc = false;
    config.toc_explicit = false;
    render_markdown_to_pdf(md, &out, config).expect("no_toc smoke should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_toc_duplicate_headings() {
    // Verify that a document with repeated heading text compiles cleanly
    // (duplicate label disambiguation must produce valid Typst).
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Overview\n\nFirst.\n\n# Overview\n\nSecond.\n\n## Details\n\nA.\n\n## Details\n\nB.\n";
    let mut config = smoke_config(&dir);
    config.toc = true;
    config.toc_explicit = true;
    render_markdown_to_pdf(md, &out, config).expect("duplicate headings smoke should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_thematic_break() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "Before\n\n---\n\nAfter\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_missing_image_renders_fallback() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    // Local image that does not exist → fallback block
    let md = "![Missing](no-such-file.png)\n\nSome text after.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_inline_html_br() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "Line one<br>Line two<br>Line three\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_block_html_ul() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "<ul><li>Alpha</li><li>Beta</li></ul>\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_letter_preset() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let mut config = smoke_config(&dir);
    config.page_width_mm = 215.9;
    config.page_height_mm = 279.4;
    config.margin_mm = 20.0;
    render_markdown_to_pdf("# US Letter\n\nContent.\n", &out, config)
        .expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_slides_preset() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let mut config = smoke_config(&dir);
    config.page_width_mm = 338.0;
    config.page_height_mm = 190.0;
    config.margin_mm = 12.0;
    config.base_font_size_pt = 20.0;
    render_markdown_to_pdf("# Slide Title\n\n- Bullet one\n- Bullet two\n", &out, config)
        .expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_unicode_content() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Unicode Test\n\nEmoji: 🦀  CJK: 中文  Accents: café résumé naïve\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_description_list() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "Term\n: Definition goes here.\n\nAnother\n: Second definition.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("render should succeed");
    assert_is_pdf(&out);
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixture file smoke tests
// ─────────────────────────────────────────────────────────────────────────────

fn smoke_fixture(filename: &str) {
    let md_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/samples")
        .join(filename);
    let md = std::fs::read_to_string(&md_path)
        .unwrap_or_else(|_| panic!("fixture not found: {}", md_path.display()));
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    render_markdown_to_pdf(&md, &out, smoke_config(&dir))
        .unwrap_or_else(|e| panic!("fixture '{}' failed: {}", filename, e));
    assert_is_pdf(&out);
}

#[test]
fn smoke_fixture_basic() {
    smoke_fixture("basic.md");
}

#[test]
fn smoke_fixture_math() {
    smoke_fixture("math.md");
}

#[test]
fn smoke_fixture_special_chars() {
    smoke_fixture("special_chars.md");
}

#[test]
fn smoke_fixture_footnotes() {
    smoke_fixture("footnotes.md");
}

#[test]
fn smoke_fixture_code_blocks() {
    smoke_fixture("code_blocks.md");
}

#[test]
fn smoke_fixture_tables() {
    smoke_fixture("tables.md");
}

// ── Dollar-sign / special character regression ────────────────────────────────

#[test]
fn smoke_dollar_sign_in_text_does_not_crash() {
    // Regression: a bare `$` in text (e.g. "$9.99") must be escaped as `\$`
    // so Typst doesn't interpret it as an unclosed math delimiter.
    let dir = TempDir::new().unwrap();
    let md = "# Pricing\n\nBuy now for only $9.99 — or two for $18!\n\nAlso: \\$escaped and plain $$.";
    let out = dir.path().join("out.pdf");
    render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("dollar signs in plain text should not crash");
    assert_is_pdf(&out);
}

// ── Syntax-theme smoke (ocean dark) ───────────────────────────────────────────

#[test]
fn smoke_ocean_dark_theme() {
    let dir = TempDir::new().unwrap();
    let md = "# Code\n\n```rust\nfn main() { println!(\"hello\"); }\n```\n";
    let out = dir.path().join("out.pdf");
    let mut config = smoke_config(&dir);
    config.syntax_theme = "base16-ocean.dark".to_string();
    render_markdown_to_pdf(md, &out, config)
        .expect("ocean dark theme should work");
    assert_is_pdf(&out);
}
