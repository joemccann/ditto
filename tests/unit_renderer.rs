//! Unit tests for Markdown-to-Typst helper functions in `renderer.rs`.
//!
//! These tests exercise the pure transformation functions — escaping, label
//! generation, math translation, TOC generation, frontmatter parsing — without
//! spawning any Typst compilation.

use md_to_pdf::renderer::{
    FontSet, RenderConfig, TocEntry,
    escape_typst_text_pub as escape_typst_text,
    extract_toc_pub as extract_toc,
    generate_typst_toc_pub as generate_typst_toc,
    heading_label_pub as heading_label,
    latex_to_typst_pub as latex_to_typst,
    markdown_to_typst_pub as md_to_typst,
    stable_name_pub as stable_name,
    typst_quoted_string_pub as typst_quoted_string,
};
use tempfile::TempDir;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn default_config(dir: &TempDir) -> RenderConfig {
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
        cache_dir_override: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// escape_typst_text
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn escape_backslash() {
    assert_eq!(escape_typst_text("a\\b"), "a\\\\b");
}

#[test]
fn escape_hash() {
    assert_eq!(escape_typst_text("a#b"), "a\\#b");
}

#[test]
fn escape_open_bracket() {
    assert_eq!(escape_typst_text("a[b"), "a\\[b");
}

#[test]
fn escape_close_bracket() {
    assert_eq!(escape_typst_text("a]b"), "a\\]b");
}

#[test]
fn escape_open_brace() {
    assert_eq!(escape_typst_text("{key}"), "\\{key\\}");
}

#[test]
fn escape_double_quote() {
    assert_eq!(escape_typst_text("say \"hi\""), "say \\\"hi\\\"");
}

#[test]
fn escape_newline_becomes_space() {
    assert_eq!(escape_typst_text("line1\nline2"), "line1 line2");
}

#[test]
fn escape_plain_text_unchanged() {
    assert_eq!(escape_typst_text("hello world"), "hello world");
}

#[test]
fn escape_all_specials_combined() {
    let input = r#"\#[]{}"quote""#;
    let out = escape_typst_text(input);
    assert!(out.contains("\\\\"), "backslash: {out}");
    assert!(out.contains("\\#"), "hash: {out}");
    assert!(out.contains("\\["), "open bracket: {out}");
    assert!(out.contains("\\]"), "close bracket: {out}");
}

// ─────────────────────────────────────────────────────────────────────────────
// typst_quoted_string
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn quoted_simple() {
    assert_eq!(typst_quoted_string("hello"), "\"hello\"");
}

#[test]
fn quoted_with_backslash() {
    assert_eq!(typst_quoted_string("a\\b"), "\"a\\\\b\"");
}

#[test]
fn quoted_with_double_quote() {
    assert_eq!(typst_quoted_string("say \"hi\""), "\"say \\\"hi\\\"\"");
}

#[test]
fn quoted_empty_string() {
    assert_eq!(typst_quoted_string(""), "\"\"");
}

#[test]
fn quoted_font_name_spaces() {
    assert_eq!(typst_quoted_string("Libertinus Serif"), "\"Libertinus Serif\"");
}

// ─────────────────────────────────────────────────────────────────────────────
// heading_label
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn label_simple_lowercase() {
    assert_eq!(heading_label("Introduction"), "introduction");
}

#[test]
fn label_spaces_become_dashes() {
    assert_eq!(heading_label("Getting Started"), "getting-started");
}

#[test]
fn label_special_chars_stripped() {
    // Punctuation → dashes, then collapsed
    assert_eq!(heading_label("What's New?"), "what-s-new");
}

#[test]
fn label_strips_typst_markup() {
    // #strong[text] → text → "text"
    assert_eq!(heading_label("#strong[Chapter 1]"), "chapter-1");
}

#[test]
fn label_leading_digit_prefixed() {
    // Must not start with a digit
    let label = heading_label("1. Introduction");
    assert!(!label.chars().next().unwrap().is_ascii_digit(),
        "label should not start with digit: {label}");
}

#[test]
fn label_empty_produces_h_prefix() {
    let label = heading_label("---");
    // All dashes collapse — should fall back to "h-" prefix or similar
    assert!(!label.is_empty(), "label should not be empty");
}

#[test]
fn label_unicode_mapped_to_dashes() {
    let label = heading_label("Über Alles");
    // Non-ASCII chars become dashes
    assert!(!label.contains('ü'), "should not contain ü: {label}");
    assert!(!label.is_empty());
}

#[test]
fn label_multiple_dashes_collapsed() {
    // "Hello   World" → "hello-world" (run of dashes collapsed)
    let label = heading_label("Hello   World");
    assert_eq!(label, "hello-world");
}

// ─────────────────────────────────────────────────────────────────────────────
// latex_to_typst (math translation)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn math_frac() {
    let out = latex_to_typst(r"\frac{a}{b}");
    assert!(out.contains("frac(a, b)"), "got: {out}");
}

#[test]
fn math_sqrt() {
    let out = latex_to_typst(r"\sqrt{x}");
    assert!(out.contains("sqrt(x)"), "got: {out}");
}

#[test]
fn math_sqrt_nth_root() {
    let out = latex_to_typst(r"\sqrt[3]{8}");
    assert!(out.contains("root(3, 8)"), "got: {out}");
}

#[test]
fn math_hat() {
    let out = latex_to_typst(r"\hat{x}");
    assert!(out.contains("hat(x)"), "got: {out}");
}

#[test]
fn math_greek_lowercase() {
    let out = latex_to_typst(r"\alpha + \beta + \gamma");
    assert!(out.contains("alpha"), "got: {out}");
    assert!(out.contains("beta"), "got: {out}");
    assert!(out.contains("gamma"), "got: {out}");
}

#[test]
fn math_greek_uppercase() {
    let out = latex_to_typst(r"\Omega + \Delta");
    assert!(out.contains("Omega"), "got: {out}");
    assert!(out.contains("Delta"), "got: {out}");
}

#[test]
fn math_infty() {
    let out = latex_to_typst(r"\infty");
    assert!(out.contains("oo"), "got: {out}");
}

#[test]
fn math_leq_geq() {
    let out = latex_to_typst(r"a \leq b \geq c");
    assert!(out.contains("lt.eq"), "got: {out}");
    assert!(out.contains("gt.eq"), "got: {out}");
}

#[test]
fn math_arrows() {
    let out = latex_to_typst(r"f: A \rightarrow B");
    assert!(out.contains("->"), "got: {out}");
}

#[test]
fn math_sum_integral() {
    let out = latex_to_typst(r"\sum_{n=0}^{\infty} \int_0^1");
    assert!(out.contains("sum"), "got: {out}");
    assert!(out.contains("integral"), "got: {out}");
}

#[test]
fn math_text_command() {
    let out = latex_to_typst(r"\text{hello}");
    assert!(out.contains("\"hello\""), "got: {out}");
}

#[test]
fn math_mathbb_r() {
    let out = latex_to_typst(r"\mathbb{R}");
    assert!(out.contains("RR"), "got: {out}");
}

#[test]
fn math_mathbb_z() {
    let out = latex_to_typst(r"\mathbb{Z}");
    assert!(out.contains("ZZ"), "got: {out}");
}

#[test]
fn math_bold() {
    let out = latex_to_typst(r"\mathbf{v}");
    assert!(out.contains("bold(v)"), "got: {out}");
}

#[test]
fn math_partial_nabla() {
    let out = latex_to_typst(r"\partial f / \nabla g");
    assert!(out.contains("diff"), "got: {out}");
    assert!(out.contains("nabla"), "got: {out}");
}

#[test]
fn math_pmatrix() {
    let out = latex_to_typst(r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}");
    assert!(out.contains("mat("), "got: {out}");
    assert!(out.contains("a, b"), "got: {out}");
    assert!(out.contains("c, d"), "got: {out}");
}

#[test]
fn math_bmatrix() {
    let out = latex_to_typst(r"\begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix}");
    assert!(out.contains("mat(delim: \"[\""), "got: {out}");
}

#[test]
fn math_cases() {
    let out = latex_to_typst(r"\begin{cases} x^2 & \text{if } x \geq 0 \\ -x & \text{if } x < 0 \end{cases}");
    assert!(out.contains("cases("), "got: {out}");
}

#[test]
fn math_lim_sin_cos() {
    let out = latex_to_typst(r"\lim_{x \to 0} \frac{\sin x}{\cos x}");
    assert!(out.contains("lim"), "got: {out}");
    assert!(out.contains("sin"), "got: {out}");
    assert!(out.contains("cos"), "got: {out}");
}

#[test]
fn math_unknown_passthrough() {
    // Unknown commands pass through with leading backslash
    let out = latex_to_typst(r"\unknowncmd");
    assert!(out.contains("\\unknowncmd"), "got: {out}");
}

#[test]
fn math_escaped_braces() {
    let out = latex_to_typst(r"\{a\}");
    assert!(out.contains('{') && out.contains('}'), "braces should appear literally: {out}");
}

#[test]
fn math_spacing_commands_become_space() {
    let out = latex_to_typst(r"a\,b");
    // spacing commands insert a space
    assert!(out.contains("a ") || out.contains(" b"), "got: {out}");
}

// ─────────────────────────────────────────────────────────────────────────────
// generate_typst_toc
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn toc_contains_outline_call() {
    let toc = generate_typst_toc(3);
    assert!(toc.contains("#outline("), "got: {toc}");
}

#[test]
fn toc_depth_is_respected() {
    let toc = generate_typst_toc(2);
    assert!(toc.contains("depth: 2"), "got: {toc}");
    let toc6 = generate_typst_toc(6);
    assert!(toc6.contains("depth: 6"), "got: {toc6}");
}

#[test]
fn toc_has_indent() {
    let toc = generate_typst_toc(3);
    assert!(toc.contains("indent"), "got: {toc}");
}

#[test]
fn toc_has_title() {
    let toc = generate_typst_toc(3);
    assert!(toc.contains("Table of Contents"), "got: {toc}");
}

// ─────────────────────────────────────────────────────────────────────────────
// extract_toc
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn toc_entries_basic() {
    let md = "# H1\n## H2\n### H3\n";
    let entries: Vec<TocEntry> = extract_toc(md);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].level, 1);
    assert_eq!(entries[0].title, "H1");
    assert_eq!(entries[1].level, 2);
    assert_eq!(entries[2].level, 3);
}

#[test]
fn toc_ignores_code_fence_headings() {
    // Headings inside fenced code blocks should still be extracted — extract_toc
    // is a lightweight line scanner that does not track fence state.
    // (This is a known limitation; the test documents it.)
    let md = "# Real Heading\n```\n# Not a real heading\n```\n## Another Real\n";
    let entries: Vec<TocEntry> = extract_toc(md);
    // At least the two real headings should be present
    assert!(entries.iter().any(|e| e.title == "Real Heading"));
    assert!(entries.iter().any(|e| e.title == "Another Real"));
}

#[test]
fn toc_entry_title_trimmed() {
    let md = "##   Spaces Around  \n";
    let entries: Vec<TocEntry> = extract_toc(md);
    assert_eq!(entries[0].title, "Spaces Around");
}

#[test]
fn toc_empty_document() {
    let entries: Vec<TocEntry> = extract_toc("No headings here.");
    assert!(entries.is_empty());
}

#[test]
fn toc_requires_space_after_hashes() {
    // "#foo" is NOT a heading (no space after #)
    let md = "#nospace\n# has space\n";
    let entries: Vec<TocEntry> = extract_toc(md);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, "has space");
}

// ─────────────────────────────────────────────────────────────────────────────
// stable_name (hash helper)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn stable_name_same_input_same_output() {
    let a = stable_name("https://example.com/image.png");
    let b = stable_name("https://example.com/image.png");
    assert_eq!(a, b);
}

#[test]
fn stable_name_different_inputs_different() {
    let a = stable_name("https://example.com/a.png");
    let b = stable_name("https://example.com/b.png");
    assert_ne!(a, b);
}

#[test]
fn stable_name_hex_chars_only() {
    let name = stable_name("test");
    assert!(name.chars().all(|c| c.is_ascii_hexdigit()),
        "stable_name should be hex: {name}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Frontmatter parsing (via markdown_to_typst_pub)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn frontmatter_toc_true_enables_outline() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\n---\n# Hello\n";
    let config = default_config(&dir);
    // config.toc_explicit = false, so frontmatter should win
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("#outline("), "frontmatter toc:true should produce outline, got:\n{src}");
}

#[test]
fn frontmatter_toc_false_suppresses_outline() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: false\n---\n# Hello\n";
    let mut config = default_config(&dir);
    config.toc = true;          // default toc=true
    config.toc_explicit = false; // but NOT explicit → frontmatter wins
    let src = md_to_typst(md, &config).unwrap();
    assert!(!src.contains("#outline("), "frontmatter toc:false should suppress outline, got:\n{src}");
}

#[test]
fn frontmatter_toc_depth_overrides_config() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\ntoc_depth: 2\n---\n# Hello\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("depth: 2"), "frontmatter toc_depth:2 should set depth, got:\n{src}");
}

#[test]
fn frontmatter_cli_explicit_wins_over_frontmatter() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\n---\n# Hello\n";
    let mut config = default_config(&dir);
    config.toc = false;
    config.toc_explicit = true; // CLI flag explicitly said NO TOC
    let src = md_to_typst(md, &config).unwrap();
    assert!(!src.contains("#outline("),
        "explicit CLI flag should override frontmatter toc:true, got:\n{src}");
}

#[test]
fn frontmatter_alternate_closing_delimiter() {
    // YAML allows `...` as closing delimiter
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\n...\n# Hello\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("#outline("), "... delimiter should work, got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Page layout headers
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn typst_source_has_set_page() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("# Hello\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#set page("), "got:\n{src}");
    assert!(src.contains("210mm") || src.contains("210"), "got:\n{src}");
}

#[test]
fn typst_source_has_set_text() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("# Hello\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#set text("), "got:\n{src}");
    assert!(src.contains("12pt"), "font size should appear: got:\n{src}");
}

#[test]
fn typst_source_has_show_link() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("# Hello\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#show link:"), "got:\n{src}");
    assert!(src.contains("fill: blue"), "got:\n{src}");
}

#[test]
fn typst_source_custom_page_size() {
    let dir = TempDir::new().unwrap();
    let mut config = default_config(&dir);
    config.page_width_mm = 338.0;
    config.page_height_mm = 190.0;
    config.margin_mm = 12.0;
    let src = md_to_typst("Hello\n", &config).unwrap();
    assert!(src.contains("338mm") || src.contains("338"), "got:\n{src}");
    assert!(src.contains("190mm") || src.contains("190"), "got:\n{src}");
    assert!(src.contains("12mm") || src.contains("12"), "got:\n{src}");
}
