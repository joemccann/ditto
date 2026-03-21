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

// ─────────────────────────────────────────────────────────────────────────────
// New fixture smoke tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn smoke_fixture_mixed_content() {
    smoke_fixture("mixed_content.md");
}

#[test]
fn smoke_fixture_edge_cases() {
    smoke_fixture("edge_cases.md");
}

#[test]
fn smoke_fixture_comprehensive_gfm() {
    let md_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/samples/comprehensive_gfm.md");
    let md = std::fs::read_to_string(md_path).expect("comprehensive_gfm.md not found");
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    // Use frontmatter-controlled TOC (toc_explicit = false)
    let mut config = smoke_config(&dir);
    config.toc = false;
    config.toc_explicit = false;
    render_markdown_to_pdf(&md, &out, config)
        .expect("comprehensive_gfm smoke should succeed");
    assert_is_pdf(&out);
}

#[test]
fn smoke_fixture_regression() {
    smoke_fixture("regression.md");
}

// ─────────────────────────────────────────────────────────────────────────────
// New math smoke tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn smoke_math_greek_alphabet() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Greek Symbols\n\n$\\alpha, \\beta, \\gamma, \\delta, \\epsilon$\n\n\
              $\\zeta, \\eta, \\theta, \\iota, \\kappa, \\lambda$\n\n\
              $\\mu, \\nu, \\xi, \\pi, \\rho, \\sigma, \\tau$\n\n\
              $\\Gamma, \\Delta, \\Theta, \\Lambda, \\Xi, \\Pi, \\Sigma, \\Omega$\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("greek alphabet smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_math_calculus() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    // Use well-supported Typst math — avoid bare `dx` which Typst treats as unknown var
    let md = "# Calculus\n\n\
              $$\\int_0^{\\infty} e^{-x^2} = \\frac{\\sqrt{\\pi}}{2}$$\n\n\
              $$\\frac{d}{d x} \\sin x = \\cos x$$\n\n\
              $$\\lim_{x \\to 0} \\frac{\\sin x}{x} = 1$$\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("calculus smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_math_linear_algebra() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Linear Algebra\n\n\
              $$\\begin{pmatrix} 1 & 2 \\\\ 3 & 4 \\end{pmatrix}$$\n\n\
              $$\\begin{bmatrix} a & b & c \\\\ d & e & f \\end{bmatrix}$$\n\n\
              $$A \\cdot B = C$$\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("linear algebra smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_math_physics() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    // Use single-letter variables to avoid Typst "unknown variable" errors
    let md = "# Physics Equations\n\n\
              $m c^2$ — mass-energy relation.\n\n\
              $F = m a$ — Newton's second law.\n\n\
              $$E = \\hbar \\omega$$\n\n\
              $$\\nabla \\cdot \\mathbf{E} = \\frac{\\rho}{\\epsilon_0}$$\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("physics smoke");
    assert_is_pdf(&out);
}

// ─────────────────────────────────────────────────────────────────────────────
// New GFM feature smoke tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn smoke_nested_lists_deep() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "- Level 1\n  - Level 2\n    - Level 3\n      - Level 4\n        - Level 5\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("deep list smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_mixed_nested_lists() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "- Category Alpha\n  1. Sub one\n  2. Sub two\n- Category Beta\n  1. Sub one\n  2. Sub two\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("mixed list smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_definition_list() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "Markdown\n: A lightweight markup language.\n\nTypst\n: A typesetting engine.\n: Written in Rust.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("def list smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_inline_html_elements() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "Text with <strong>bold</strong>, <em>italic</em>, <u>underline</u>.\n\nLine one<br>Line two.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir)).expect("inline HTML smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_long_document_multi_page() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    // Generate a document long enough to span multiple pages
    let mut md = String::from("# Long Document\n\n");
    for i in 1..=20 {
        md.push_str(&format!("## Section {i}\n\n"));
        md.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit. ");
        md.push_str("Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ");
        md.push_str("Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.\n\n");
        md.push_str(&format!("- Item A in section {i}\n- Item B in section {i}\n- Item C in section {i}\n\n"));
    }
    let summary = render_markdown_to_pdf(&md, &out, smoke_config(&dir))
        .expect("long document smoke");
    assert_is_pdf(&out);
    assert!(summary.pages >= 2, "long doc should span multiple pages: {} pages", summary.pages);
}

#[test]
fn smoke_document_with_all_gfm_features() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# All GFM Features\n\n\
              ## Bold and Italic\n\n**Bold** and *italic* and ***both***.\n\n\
              ## Strikethrough\n\n~~struck~~ text.\n\n\
              ## Lists\n\n- Alpha\n- Beta\n  - Nested\n\n1. One\n2. Two\n\n\
              - [x] Done\n- [ ] Not done\n\n\
              ## Table\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n\
              ## Code\n\n```rust\nfn main() {}\n```\n\n\
              ## Math\n\n$m c^2$ and $$\\int_0^1 x^2 = \\frac{1}{3}$$\n\n\
              ## Footnotes\n\nNote[^n].\n\n[^n]: Footnote body.\n\n\
              ## Definition List\n\nTerm\n: Definition.\n\n\
              ## Autolink\n\nhttps://example.com\n\n\
              ## Blockquote\n\n> A quoted line.\n";
    let summary = render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("all GFM features smoke");
    assert_is_pdf(&out);
    assert!(summary.pages >= 1);
    assert!(summary.toc_entries.len() >= 9, "all section headings extracted");
}

#[test]
fn smoke_repeated_heading_text() {
    // Duplicate headings must produce valid Typst (label disambiguation)
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Introduction\n\nFirst.\n\n# Introduction\n\nSecond.\n\n\
              ## Details\n\nA.\n\n## Details\n\nB.\n\n## Details\n\nC.\n";
    let mut config = smoke_config(&dir);
    config.toc = true;
    config.toc_explicit = true;
    render_markdown_to_pdf(md, &out, config)
        .expect("duplicate headings must compile without error");
    assert_is_pdf(&out);
}

#[test]
fn smoke_special_chars_throughout() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Prices and Tags\n\nCost: $9.99 per unit.\n\nTag: #1 on the charts.\n\n\
              ## Code\n\n```python\nprice = \"$9.99\"  # USD\ncount = 42\n```\n\n\
              ## Backslashes\n\nPath: `C:\\Users\\name`.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("special chars smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_toc_with_comprehensive_sections() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Part I\n\n## Chapter 1\n\n### 1.1 Introduction\n\nContent.\n\n\
              ### 1.2 Background\n\nContext.\n\n## Chapter 2\n\n### 2.1 Method\n\nApproach.\n\n\
              # Part II\n\n## Chapter 3\n\nMore content.\n\n### 3.1 Results\n\nFindings.\n";
    let mut config = smoke_config(&dir);
    config.toc = true;
    config.toc_explicit = true;
    config.toc_depth = 3;
    let summary = render_markdown_to_pdf(md, &out, config)
        .expect("TOC comprehensive smoke");
    assert_is_pdf(&out);
    assert!(summary.pages >= 2, "TOC + content: {} pages", summary.pages);
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional preset / layout smoke tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn smoke_a3_preset() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let mut config = smoke_config(&dir);
    config.page_width_mm = 297.0;
    config.page_height_mm = 420.0;
    render_markdown_to_pdf("# A3 Document\n\nContent.\n", &out, config)
        .expect("A3 smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_large_font_size() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let mut config = smoke_config(&dir);
    config.base_font_size_pt = 24.0;
    render_markdown_to_pdf("# Big Text\n\nLarge body text.\n", &out, config)
        .expect("large font smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_minimal_margin() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let mut config = smoke_config(&dir);
    config.margin_mm = 5.0;
    render_markdown_to_pdf("# Minimal Margin\n\nContent.\n", &out, config)
        .expect("minimal margin smoke");
    assert_is_pdf(&out);
}

// ─────────────────────────────────────────────────────────────────────────────
// Syntax theme smoke tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn smoke_solarized_dark_theme() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let mut config = smoke_config(&dir);
    config.syntax_theme = "Solarized (dark)".to_string();
    let md = "```python\ndef hello(): print('world')\n```\n";
    render_markdown_to_pdf(md, &out, config).expect("solarized dark smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_inspired_github_theme() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let mut config = smoke_config(&dir);
    config.syntax_theme = "InspiredGitHub".to_string();
    let md = "```rust\nfn main() { println!(\"hello\"); }\n```\n";
    render_markdown_to_pdf(md, &out, config).expect("InspiredGitHub smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_base16_eighties_theme() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let mut config = smoke_config(&dir);
    config.syntax_theme = "base16-eighties.dark".to_string();
    let md = "```javascript\nconst x = 42;\nconsole.log(x);\n```\n";
    render_markdown_to_pdf(md, &out, config).expect("base16-eighties smoke");
    assert_is_pdf(&out);
}

// ─────────────────────────────────────────────────────────────────────────────
// Regression smoke tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn smoke_regression_at_sign_in_text() {
    // @ in plain text (not a link) must not crash Typst
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "Contact user@example.com or admin@test.org.\n\n\
              Also: <user@example.com> as an autolink.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("@ in text smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_regression_empty_code_block() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "```\n```\n\nText after.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("empty code block smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_regression_consecutive_headings() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\n# H1 Again\n## H2 Again\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("consecutive headings smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_regression_no_content_footnote() {
    // A footnote definition with only inline content — no extra paragraphs
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "Word[^f].\n\n[^f]: Short.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("short footnote smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_regression_fenced_math_block() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Fenced Math\n\n```math\nm c^2\n```\n\n```math\n\\int_0^1 x^2 = \\frac{1}{3}\n```\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("fenced math smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_regression_table_single_row() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "| Name | Value |\n|------|-------|\n| Only | row |\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("single-row table smoke");
    assert_is_pdf(&out);
}

#[test]
fn smoke_regression_image_missing_renders_pdf() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");
    let md = "# Document with Missing Images\n\n\
              ![First missing](a.png)\n\n\
              Some text between images.\n\n\
              ![Second missing](b.png)\n\n\
              More text after.\n";
    render_markdown_to_pdf(md, &out, smoke_config(&dir))
        .expect("multiple missing images smoke");
    assert_is_pdf(&out);
}
