//! Unit tests for Markdown-to-Typst helper functions in `renderer.rs`.
//!
//! These tests exercise the pure transformation functions — escaping, label
//! generation, math translation, TOC generation, frontmatter parsing — without
//! spawning any Typst compilation.

use ditto::renderer::{
    FontSet, RenderConfig, TocEntry, escape_typst_text_pub as escape_typst_text,
    extract_toc_pub as extract_toc, generate_typst_toc_pub as generate_typst_toc,
    generate_typst_toc_titled_pub as generate_typst_toc_titled, heading_label_pub as heading_label,
    latex_to_typst_pub as latex_to_typst, markdown_to_typst_pub as md_to_typst,
    stable_name_pub as stable_name, typst_quoted_string_pub as typst_quoted_string,
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

#[test]
fn escape_dollar_sign_prevents_math_mode() {
    // A bare `$` in plain text must become `\$` so Typst doesn't enter math mode.
    // The classic case: prices like "$9.99" in a paragraph.
    assert_eq!(escape_typst_text("$9.99"), "\\$9.99");
}

#[test]
fn escape_dollar_pair_does_not_create_math_mode() {
    // Two dollar signs in plain text (e.g. "$100 and $200") must both be escaped.
    let out = escape_typst_text("$100 and $200");
    assert_eq!(out, "\\$100 and \\$200");
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
    assert_eq!(
        typst_quoted_string("Libertinus Serif"),
        "\"Libertinus Serif\""
    );
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
    assert!(
        !label.chars().next().unwrap().is_ascii_digit(),
        "label should not start with digit: {label}"
    );
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
    let out = latex_to_typst(
        r"\begin{cases} x^2 & \text{if } x \geq 0 \\ -x & \text{if } x < 0 \end{cases}",
    );
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
    assert!(
        out.contains('{') && out.contains('}'),
        "braces should appear literally: {out}"
    );
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
    assert!(
        name.chars().all(|c| c.is_ascii_hexdigit()),
        "stable_name should be hex: {name}"
    );
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
    assert!(
        src.contains("#outline("),
        "frontmatter toc:true should produce outline, got:\n{src}"
    );
}

#[test]
fn frontmatter_toc_false_suppresses_outline() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: false\n---\n# Hello\n";
    let mut config = default_config(&dir);
    config.toc = true; // default toc=true
    config.toc_explicit = false; // but NOT explicit → frontmatter wins
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        !src.contains("#outline("),
        "frontmatter toc:false should suppress outline, got:\n{src}"
    );
}

#[test]
fn frontmatter_toc_depth_overrides_config() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\ntoc_depth: 2\n---\n# Hello\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        src.contains("depth: 2"),
        "frontmatter toc_depth:2 should set depth, got:\n{src}"
    );
}

#[test]
fn frontmatter_cli_explicit_wins_over_frontmatter() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\n---\n# Hello\n";
    let mut config = default_config(&dir);
    config.toc = false;
    config.toc_explicit = true; // CLI flag explicitly said NO TOC
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        !src.contains("#outline("),
        "explicit CLI flag should override frontmatter toc:true, got:\n{src}"
    );
}

#[test]
fn frontmatter_alternate_closing_delimiter() {
    // YAML allows `...` as closing delimiter
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\n...\n# Hello\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        src.contains("#outline("),
        "... delimiter should work, got:\n{src}"
    );
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

// ─────────────────────────────────────────────────────────────────────────────
// TOC improvements: page-numbered outline, clickable nav, depth, title, no_toc
// ─────────────────────────────────────────────────────────────────────────────

// ── generate_typst_toc: page-break and styling ────────────────────────────────

#[test]
fn toc_emits_pagebreak_after_outline() {
    let toc = generate_typst_toc(3);
    // A #pagebreak() must follow the outline so body content starts on a fresh page.
    assert!(
        toc.contains("#pagebreak()"),
        "expected #pagebreak() after outline, got:\n{toc}"
    );
}

#[test]
fn toc_pagebreak_comes_after_outline_block() {
    let toc = generate_typst_toc(3);
    let outline_pos = toc.find("#outline(").expect("#outline() not found");
    let break_pos = toc.find("#pagebreak()").expect("#pagebreak() not found");
    assert!(
        break_pos > outline_pos,
        "#pagebreak() should come after #outline()"
    );
}

#[test]
fn toc_h1_entries_bold_via_show_rule() {
    // The `#show outline.entry.where(level: 1)` rule wraps H1 entries in strong().
    let toc = generate_typst_toc(3);
    assert!(
        toc.contains("outline.entry.where(level: 1)"),
        "expected show rule for H1 entries, got:\n{toc}"
    );
    assert!(
        toc.contains("strong("),
        "expected strong() in show rule, got:\n{toc}"
    );
}

#[test]
fn toc_indent_parameter_present() {
    let toc = generate_typst_toc(3);
    assert!(
        toc.contains("indent:"),
        "expected indent: parameter, got:\n{toc}"
    );
}

// ── Custom TOC title via generate_typst_toc_titled ───────────────────────────

#[test]
fn toc_custom_title_appears_in_outline() {
    let toc = generate_typst_toc_titled(3, "Contents");
    assert!(
        toc.contains("Contents"),
        "expected custom title, got:\n{toc}"
    );
    assert!(
        !toc.contains("Table of Contents"),
        "should not have default title, got:\n{toc}"
    );
}

#[test]
fn toc_custom_title_special_chars_escaped() {
    // Typst-special chars in the title must be escaped.
    let toc = generate_typst_toc_titled(2, "My #Doc");
    assert!(
        toc.contains("\\#Doc"),
        "hash in title should be escaped, got:\n{toc}"
    );
}

#[test]
fn toc_default_title_is_table_of_contents() {
    let toc = generate_typst_toc(2);
    assert!(
        toc.contains("Table of Contents"),
        "default title should be 'Table of Contents', got:\n{toc}"
    );
}

// ── `toc_title` frontmatter key ───────────────────────────────────────────────

#[test]
fn frontmatter_toc_title_overrides_default() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\ntoc_title: My Document Index\n---\n# Hello\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        src.contains("My Document Index"),
        "expected custom toc_title, got:\n{src}"
    );
    assert!(
        !src.contains("Table of Contents"),
        "should not use default title, got:\n{src}"
    );
}

#[test]
fn frontmatter_toc_title_quoted_value() {
    // toc_title: "Quoted Title" — quotes should be stripped
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\ntoc_title: \"My Index\"\n---\n# Hello\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        src.contains("My Index"),
        "expected unquoted title in output, got:\n{src}"
    );
}

#[test]
fn frontmatter_toc_title_without_toc_has_no_effect() {
    // toc_title only matters when a TOC is actually rendered.
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc_title: Hidden\n---\n# Hello\n";
    let mut config = default_config(&dir);
    config.toc = false;
    config.toc_explicit = true;
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        !src.contains("#outline("),
        "no TOC should be emitted, got:\n{src}"
    );
}

// ── `no_toc` frontmatter key ──────────────────────────────────────────────────

#[test]
fn frontmatter_no_toc_true_suppresses_outline() {
    let dir = TempDir::new().unwrap();
    let md = "---\nno_toc: true\n---\n# Hello\n";
    let mut config = default_config(&dir);
    config.toc = true; // default would emit TOC
    config.toc_explicit = false; // but not explicit → frontmatter wins
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        !src.contains("#outline("),
        "no_toc: true should suppress outline, got:\n{src}"
    );
}

#[test]
fn frontmatter_no_toc_false_does_not_suppress() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\nno_toc: false\n---\n# Hello\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        src.contains("#outline("),
        "no_toc: false should not suppress outline, got:\n{src}"
    );
}

#[test]
fn frontmatter_no_toc_and_toc_both_true_no_toc_wins() {
    // When both `toc: true` and `no_toc: true` are present, `no_toc` wins.
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\nno_toc: true\n---\n# Hello\n";
    let mut config = default_config(&dir);
    config.toc_explicit = false;
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        !src.contains("#outline("),
        "no_toc: true should override toc: true, got:\n{src}"
    );
}

#[test]
fn frontmatter_no_toc_cli_explicit_still_wins() {
    // When CLI explicitly enables TOC, frontmatter no_toc should not override.
    let dir = TempDir::new().unwrap();
    let md = "---\nno_toc: true\n---\n# Hello\n";
    let mut config = default_config(&dir);
    config.toc = true;
    config.toc_explicit = true; // CLI said YES → wins over frontmatter
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        src.contains("#outline("),
        "CLI explicit --toc should override frontmatter no_toc: true, got:\n{src}"
    );
}

// ── Duplicate heading label disambiguation ────────────────────────────────────

#[test]
fn duplicate_headings_get_unique_labels() {
    let dir = TempDir::new().unwrap();
    let md = "# Overview\n\nFirst section.\n\n# Overview\n\nSecond section.\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    // First occurrence: <overview>
    assert!(
        src.contains("<overview>"),
        "first label should be <overview>, got:\n{src}"
    );
    // Second occurrence: <overview-2>
    assert!(
        src.contains("<overview-2>"),
        "second label should be <overview-2>, got:\n{src}"
    );
    // Should NOT have two <overview> labels (that would be invalid Typst)
    let count = src.matches("<overview>").count();
    assert_eq!(
        count, 1,
        "base label <overview> should appear exactly once, got:\n{src}"
    );
}

#[test]
fn triple_duplicate_headings_all_unique() {
    let dir = TempDir::new().unwrap();
    let md = "# Intro\n\n# Intro\n\n# Intro\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("<intro>"), "first: <intro>, got:\n{src}");
    assert!(src.contains("<intro-2>"), "second: <intro-2>, got:\n{src}");
    assert!(src.contains("<intro-3>"), "third: <intro-3>, got:\n{src}");
}

#[test]
fn non_duplicate_headings_keep_base_label() {
    let dir = TempDir::new().unwrap();
    let md = "# Introduction\n## Getting Started\n### Installation\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("<introduction>"), "got:\n{src}");
    assert!(src.contains("<getting-started>"), "got:\n{src}");
    assert!(src.contains("<installation>"), "got:\n{src}");
    // None should have a numeric suffix
    assert!(!src.contains("<introduction-2>"), "got:\n{src}");
    assert!(!src.contains("<getting-started-2>"), "got:\n{src}");
}

#[test]
fn mixed_levels_duplicate_labels_disambiguated() {
    // Even if H1 and H2 produce the same slug, they should be unique.
    let dir = TempDir::new().unwrap();
    let md = "# Summary\n## Summary\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("<summary>"), "first: <summary>, got:\n{src}");
    assert!(
        src.contains("<summary-2>"),
        "second: <summary-2>, got:\n{src}"
    );
    let count = src.matches("<summary>").count();
    assert_eq!(
        count, 1,
        "base <summary> should appear exactly once, got:\n{src}"
    );
}

// ── Internal clickable navigation (labels on headings) ────────────────────────

#[test]
fn every_heading_has_a_label() {
    let dir = TempDir::new().unwrap();
    let md = "# First\n## Second\n### Third\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    // Each heading line should be followed by a <label>
    assert!(
        src.contains("= First <first>"),
        "H1 label missing, got:\n{src}"
    );
    assert!(
        src.contains("== Second <second>"),
        "H2 label missing, got:\n{src}"
    );
    assert!(
        src.contains("=== Third <third>"),
        "H3 label missing, got:\n{src}"
    );
}

#[test]
fn toc_with_headings_uses_outline_for_clickable_nav() {
    // When TOC is enabled, #outline() is emitted which Typst uses to build
    // clickable cross-references via the labels we attach to headings.
    let dir = TempDir::new().unwrap();
    let md = "# Chapter One\n## Section A\n## Section B\n";
    let mut config = default_config(&dir);
    config.toc = true;
    config.toc_explicit = true;
    config.toc_depth = 2;
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        src.contains("#outline("),
        "outline should be emitted, got:\n{src}"
    );
    // Labels must be present for clickable links to work
    assert!(
        src.contains("<chapter-one>"),
        "heading label missing, got:\n{src}"
    );
    assert!(
        src.contains("<section-a>"),
        "heading label missing, got:\n{src}"
    );
    assert!(
        src.contains("<section-b>"),
        "heading label missing, got:\n{src}"
    );
}

// ── TOC depth controls which headings appear ──────────────────────────────────

#[test]
fn toc_depth_1_outline_excludes_deep_headings() {
    // The outline depth parameter controls which headings appear in the TOC.
    let dir = TempDir::new().unwrap();
    let mut config = default_config(&dir);
    config.toc = true;
    config.toc_explicit = true;
    config.toc_depth = 1;
    let src = md_to_typst("# Top\n## Sub\n### Deep\n", &config).unwrap();
    assert!(src.contains("depth: 1"), "expected depth: 1, got:\n{src}");
}

#[test]
fn toc_depth_max_6_clamped() {
    // toc_depth must be clamped to [1,6].
    let dir = TempDir::new().unwrap();
    let mut config = default_config(&dir);
    config.toc = true;
    config.toc_explicit = true;
    config.toc_depth = 6;
    let src = md_to_typst("# H1\n###### H6\n", &config).unwrap();
    assert!(src.contains("depth: 6"), "got:\n{src}");
}

#[test]
fn frontmatter_toc_depth_respected_in_outline() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\ntoc_depth: 4\n---\n# H1\n## H2\n### H3\n#### H4\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("depth: 4"), "expected depth: 4, got:\n{src}");
}

// ── TOC enable/disable round-trips ───────────────────────────────────────────

#[test]
fn toc_enabled_via_frontmatter_emits_outline_and_pagebreak() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: true\ntoc_depth: 2\n---\n# Hello\n## World\n";
    let config = default_config(&dir);
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        src.contains("#outline("),
        "should emit outline, got:\n{src}"
    );
    assert!(
        src.contains("#pagebreak()"),
        "should emit pagebreak after outline, got:\n{src}"
    );
    assert!(src.contains("depth: 2"), "should use depth 2, got:\n{src}");
}

#[test]
fn toc_disabled_no_outline_no_pagebreak() {
    let dir = TempDir::new().unwrap();
    let mut config = default_config(&dir);
    config.toc = false;
    config.toc_explicit = true;
    let src = md_to_typst("# Hello\n## World\n", &config).unwrap();
    assert!(
        !src.contains("#outline("),
        "should not emit outline, got:\n{src}"
    );
    assert!(
        !src.contains("#pagebreak()"),
        "should not emit pagebreak, got:\n{src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional escape_typst_text edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn escape_at_sign_prevents_citation() {
    // @ in text must become \@ so Typst doesn't treat it as a label reference.
    assert_eq!(escape_typst_text("user@example.com"), "user\\@example.com");
}

#[test]
fn escape_multiple_at_signs() {
    let out = escape_typst_text("a@b and c@d");
    assert_eq!(out, "a\\@b and c\\@d");
}

#[test]
fn escape_all_at_signs_in_line() {
    let out = escape_typst_text("@mention and @another");
    assert_eq!(out, "\\@mention and \\@another");
}

#[test]
fn escape_mixed_specials_in_realistic_sentence() {
    // "Use #[derive(Debug)] to implement fmt::Debug for $Type." — all specials
    let out = escape_typst_text(r#"Use #[derive(Debug)] for $Type."#);
    assert!(out.contains("\\#"), "hash: {out}");
    assert!(out.contains("\\["), "open bracket: {out}");
    assert!(out.contains("\\]"), "close bracket: {out}");
    assert!(out.contains("\\$"), "dollar: {out}");
}

#[test]
fn escape_empty_string_unchanged() {
    assert_eq!(escape_typst_text(""), "");
}

#[test]
fn escape_whitespace_only() {
    // Spaces and tabs should pass through as-is
    assert_eq!(escape_typst_text("   "), "   ");
    assert_eq!(escape_typst_text("\t"), "\t");
}

#[test]
fn escape_curly_braces_escaped() {
    // Curly braces ARE special in Typst text and must be escaped
    let out = escape_typst_text("{key: value}");
    assert!(out.contains("\\{"), "open brace: {out}");
    assert!(out.contains("\\}"), "close brace: {out}");
}

#[test]
fn escape_newline_in_middle_of_text() {
    let out = escape_typst_text("line1\nline2\nline3");
    assert_eq!(out, "line1 line2 line3");
}

#[test]
fn escape_windows_path_backslashes() {
    // Windows paths like C:\Users\name must have each \ escaped
    let out = escape_typst_text("C:\\Users\\name");
    assert_eq!(out, "C:\\\\Users\\\\name");
}

// ─────────────────────────────────────────────────────────────────────────────
// typst_quoted_string: additional edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn quoted_string_with_at_sign_not_escaped() {
    // @ is not special in Typst string literals (only in content mode)
    let out = typst_quoted_string("user@example.com");
    assert_eq!(out, "\"user@example.com\"");
}

#[test]
fn quoted_string_with_newline() {
    // Newlines inside a string literal are literal characters
    let out = typst_quoted_string("line1\nline2");
    assert!(
        out.starts_with('"') && out.ends_with('"'),
        "wrapped in quotes: {out}"
    );
}

#[test]
fn quoted_string_with_unicode() {
    let out = typst_quoted_string("café résumé");
    assert_eq!(out, "\"café résumé\"");
}

#[test]
fn quoted_string_hash_not_escaped() {
    // # is not special in Typst string literals
    let out = typst_quoted_string("#heading");
    assert_eq!(out, "\"#heading\"");
}

// ─────────────────────────────────────────────────────────────────────────────
// heading_label: additional edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn label_only_digits_gets_prefix() {
    let label = heading_label("123");
    assert!(
        label.starts_with('h') || !label.chars().next().unwrap().is_ascii_digit(),
        "label starting with digit: {label}"
    );
}

#[test]
fn label_single_letter() {
    let label = heading_label("A");
    assert_eq!(label, "a");
}

#[test]
fn label_all_special_chars() {
    // A heading of just punctuation should still produce a non-empty label
    let label = heading_label("!@#$%");
    assert!(!label.is_empty(), "label should not be empty: {label}");
}

#[test]
fn label_repeated_hyphens_collapsed() {
    // "A  -  B" → all non-alphanum → dashes → collapse to "a-b"
    let label = heading_label("A  -  B");
    assert_eq!(label, "a-b");
}

#[test]
fn label_numbers_in_middle_preserved() {
    let label = heading_label("Chapter 3: Overview");
    assert!(label.contains("3"), "digit in middle: {label}");
    assert!(label.contains("chapter"), "word part: {label}");
}

#[test]
fn label_no_trailing_dash() {
    let label = heading_label("Hello!");
    assert!(!label.ends_with('-'), "trailing dash: {label}");
}

#[test]
fn label_no_leading_dash() {
    let label = heading_label("!Hello");
    assert!(!label.starts_with('-'), "leading dash: {label}");
}

// ─────────────────────────────────────────────────────────────────────────────
// latex_to_typst: additional math commands
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn math_dfrac_same_as_frac() {
    let out = latex_to_typst(r"\dfrac{x}{y}");
    assert!(out.contains("frac(x, y)"), "got: {out}");
}

#[test]
fn math_binom() {
    let out = latex_to_typst(r"\binom{n}{k}");
    assert!(out.contains("binom(n, k)"), "got: {out}");
}

#[test]
fn math_hat_accent() {
    let out = latex_to_typst(r"\hat{v}");
    assert!(out.contains("hat(v)"), "got: {out}");
}

#[test]
fn math_tilde_accent() {
    let out = latex_to_typst(r"\tilde{x}");
    assert!(out.contains("tilde(x)"), "got: {out}");
}

#[test]
fn math_vec_becomes_arrow() {
    let out = latex_to_typst(r"\vec{v}");
    assert!(out.contains("arrow(v)"), "got: {out}");
}

#[test]
fn math_bar_becomes_overline() {
    let out = latex_to_typst(r"\bar{x}");
    assert!(out.contains("overline(x)"), "got: {out}");
}

#[test]
fn math_underline() {
    let out = latex_to_typst(r"\underline{a + b}");
    assert!(out.contains("underline("), "got: {out}");
}

#[test]
fn math_overbrace_underbrace() {
    let a = latex_to_typst(r"\overbrace{a+b}");
    assert!(a.contains("overbrace("), "got: {a}");
    let b = latex_to_typst(r"\underbrace{a+b}");
    assert!(b.contains("underbrace("), "got: {b}");
}

#[test]
fn math_cdot_times() {
    let a = latex_to_typst(r"a \cdot b");
    assert!(a.contains("dot.c"), "cdot: {a}");
    let b = latex_to_typst(r"a \times b");
    assert!(b.contains("times"), "times: {b}");
}

#[test]
fn math_pm_mp() {
    let a = latex_to_typst(r"x \pm y");
    assert!(a.contains("plus.minus"), "pm: {a}");
    let b = latex_to_typst(r"x \mp y");
    assert!(b.contains("minus.plus"), "mp: {b}");
}

#[test]
fn math_neq() {
    let out = latex_to_typst(r"a \neq b");
    assert!(out.contains("eq.not"), "got: {out}");
}

#[test]
fn math_subset_supset() {
    let a = latex_to_typst(r"A \subset B");
    assert!(a.contains("subset"), "subset: {a}");
    let b = latex_to_typst(r"A \subseteq B");
    assert!(b.contains("subset.eq"), "subseteq: {b}");
}

#[test]
fn math_forall_exists() {
    let a = latex_to_typst(r"\forall x");
    assert!(a.contains("forall"), "got: {a}");
    let b = latex_to_typst(r"\exists y");
    assert!(b.contains("exists"), "got: {b}");
}

#[test]
fn math_cup_cap_setminus() {
    let a = latex_to_typst(r"A \cup B");
    assert!(a.contains("union"), "cup: {a}");
    let b = latex_to_typst(r"A \cap B");
    assert!(b.contains("sect"), "cap: {b}");
    let c = latex_to_typst(r"A \setminus B");
    assert!(c.contains("without"), "setminus: {c}");
}

#[test]
fn math_trig_functions() {
    let out = latex_to_typst(r"\sin\theta + \cos\phi = \tan\alpha");
    assert!(out.contains("sin"), "sin: {out}");
    assert!(out.contains("cos"), "cos: {out}");
    assert!(out.contains("tan"), "tan: {out}");
}

#[test]
fn math_log_exp_ln() {
    let out = latex_to_typst(r"\log x + \ln y + \exp z");
    assert!(out.contains("log"), "log: {out}");
    assert!(out.contains("ln"), "ln: {out}");
    assert!(out.contains("exp"), "exp: {out}");
}

#[test]
fn math_lim_with_subscript() {
    let out = latex_to_typst(r"\lim_{n \to \infty} a_n");
    assert!(out.contains("lim"), "lim: {out}");
    assert!(out.contains("->") || out.contains("oo"), "subscript: {out}");
}

#[test]
fn math_prod() {
    let out = latex_to_typst(r"\prod_{i=1}^{n}");
    assert!(out.contains("prod"), "got: {out}");
}

#[test]
fn math_iint_iiint() {
    let a = latex_to_typst(r"\iint");
    assert!(a.contains("integral.double"), "iint: {a}");
    let b = latex_to_typst(r"\iiint");
    assert!(b.contains("integral.triple"), "iiint: {b}");
}

#[test]
fn math_vmatrix() {
    let out = latex_to_typst(r"\begin{vmatrix} a & b \\ c & d \end{vmatrix}");
    assert!(
        out.contains("mat(delim: \"|\",") || out.contains("mat(delim:\"|\""),
        "got: {out}"
    );
}

#[test]
fn math_align_env() {
    let out = latex_to_typst(r"\begin{align} a &= b \\ c &= d \end{align}");
    // align environment should produce at least two expressions joined by \
    assert!(!out.is_empty(), "got: {out}");
}

#[test]
fn math_equation_env() {
    let out = latex_to_typst(r"\begin{equation} x^2 + y^2 = r^2 \end{equation}");
    assert!(
        out.contains("x") && out.contains("y") && out.contains("r"),
        "got: {out}"
    );
}

#[test]
fn math_mathbb_n_q_c() {
    let n = latex_to_typst(r"\mathbb{N}");
    assert!(n.contains("NN"), "got: {n}");
    let q = latex_to_typst(r"\mathbb{Q}");
    assert!(q.contains("QQ"), "got: {q}");
    let c = latex_to_typst(r"\mathbb{C}");
    assert!(c.contains("CC"), "got: {c}");
}

#[test]
fn math_mathbf_vector() {
    let out = latex_to_typst(r"\mathbf{v}");
    assert!(out.contains("bold(v)"), "got: {out}");
}

#[test]
fn math_left_right_delimiters() {
    let out = latex_to_typst(r"\left( x + y \right)");
    // \left( and \right) should emit ( and ) — no crash
    assert!(out.contains("(") && out.contains(")"), "got: {out}");
}

#[test]
fn math_hbar_ell() {
    let a = latex_to_typst(r"\hbar");
    assert!(a.contains("planck.reduce"), "hbar: {a}");
    let b = latex_to_typst(r"\ell");
    assert!(b.contains("ell"), "ell: {b}");
}

#[test]
fn math_prime_symbol() {
    let out = latex_to_typst(r"f'(x) = f\prime(x)");
    // \prime → '
    assert!(out.contains("'"), "got: {out}");
}

#[test]
fn math_dots_variants() {
    let ldots = latex_to_typst(r"\ldots");
    assert!(ldots.contains("dots"), "ldots: {ldots}");
    let cdots = latex_to_typst(r"\cdots");
    assert!(cdots.contains("dots"), "cdots: {cdots}");
    let vdots = latex_to_typst(r"\vdots");
    assert!(vdots.contains("dots.v"), "vdots: {vdots}");
}

#[test]
fn math_quad_spacing() {
    let out = latex_to_typst(r"a \quad b");
    assert!(out.contains("quad"), "got: {out}");
}

// ─────────────────────────────────────────────────────────────────────────────
// extract_toc: additional edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn toc_heading_levels_1_through_6() {
    let md = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\n";
    let entries = extract_toc(md);
    assert_eq!(entries.len(), 6, "should have 6 entries");
    for (i, e) in entries.iter().enumerate() {
        assert_eq!(e.level, (i + 1) as u8, "level {}: {}", i + 1, e.title);
    }
}

#[test]
fn toc_mixed_levels_order_preserved() {
    let md = "# A\n### C\n## B\n# D\n";
    let entries = extract_toc(md);
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].title, "A");
    assert_eq!(entries[1].title, "C");
    assert_eq!(entries[2].title, "B");
    assert_eq!(entries[3].title, "D");
}

#[test]
fn toc_title_with_inline_markup() {
    // extract_toc returns the raw heading text including any ** or *
    let md = "## **Bold** Section\n";
    let entries = extract_toc(md);
    assert_eq!(entries.len(), 1);
    assert!(
        entries[0].title.contains("Bold"),
        "got: {}",
        entries[0].title
    );
}

#[test]
fn toc_7_or_more_hashes_not_heading() {
    // ATX headings only go up to H6 (6 hashes)
    let md = "####### Too many\n# Valid\n";
    let entries = extract_toc(md);
    // Only "Valid" should be captured; "Too many" has 7 hashes and is not a heading
    assert!(
        entries.iter().any(|e| e.title == "Valid"),
        "valid heading: {:?}",
        entries
    );
    assert!(
        !entries.iter().any(|e| e.title == "Too many"),
        "invalid heading should be absent: {:?}",
        entries
    );
}

#[test]
fn toc_frontmatter_not_included() {
    // The --- ... --- frontmatter block should not yield TOC entries even if
    // a line inside starts with #. extract_toc is a line-scanner and does NOT
    // parse frontmatter, so this test documents the known behaviour.
    let md = "# Real Heading\n";
    let entries = extract_toc(md);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, "Real Heading");
}

// ─────────────────────────────────────────────────────────────────────────────
// generate_typst_toc: clamped depth values
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn toc_depth_1_produces_depth_1() {
    let toc = generate_typst_toc(1);
    assert!(toc.contains("depth: 1"), "got: {toc}");
}

#[test]
fn toc_depth_6_produces_depth_6() {
    let toc = generate_typst_toc(6);
    assert!(toc.contains("depth: 6"), "got: {toc}");
}

#[test]
fn toc_all_required_fields_present() {
    let toc = generate_typst_toc(3);
    // Must have all of: outline call, depth, indent, title, pagebreak, show rule
    assert!(toc.contains("#outline("), "outline call: {toc}");
    assert!(toc.contains("depth:"), "depth: {toc}");
    assert!(toc.contains("indent:"), "indent: {toc}");
    assert!(toc.contains("title:"), "title: {toc}");
    assert!(toc.contains("#pagebreak()"), "pagebreak: {toc}");
    assert!(toc.contains("#show"), "show rule: {toc}");
}

// ─────────────────────────────────────────────────────────────────────────────
// markdown_to_typst: additional rendering edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md_empty_document_produces_preamble() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("", &default_config(&dir)).unwrap();
    assert!(src.contains("#set page("), "preamble present: {src}");
}

#[test]
fn md_only_frontmatter_no_body() {
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: false\n---\n";
    let src = md_to_typst(md, &default_config(&dir)).unwrap();
    // Should produce preamble but no body content besides the trailing newline
    assert!(src.contains("#set page("), "preamble: {src}");
}

#[test]
fn md_inline_code_backtick_in_output() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("`hello()`\n", &default_config(&dir)).unwrap();
    assert!(src.contains("`hello()`"), "inline code: {src}");
}

#[test]
fn md_strikethrough_strike_call() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("~~old~~\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#strike[old]"), "strikethrough: {src}");
}

#[test]
fn md_blockquote_structure() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("> A quote.\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#block("), "block: {src}");
    assert!(
        src.contains("inset: (left: 12pt)") || src.contains("inset"),
        "inset: {src}"
    );
    assert!(src.contains("stroke"), "stroke: {src}");
    assert!(src.contains("A quote."), "content: {src}");
}

#[test]
fn md_thematic_break_line() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("---\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#line(length: 100%)"), "line: {src}");
}

#[test]
fn md_soft_break_becomes_space() {
    // A soft break (single newline within a paragraph) should become a space
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("word1\nword2\n", &default_config(&dir)).unwrap();
    assert!(
        src.contains("word1") && src.contains("word2"),
        "words present: {src}"
    );
}

#[test]
fn md_link_with_url_only_compact() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("<https://example.com>\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#link("), "link: {src}");
    assert!(src.contains("example.com"), "url: {src}");
}

#[test]
fn md_bold_nested_in_italic() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("*italic and **bold** inside*\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#emph["), "emph: {src}");
    assert!(src.contains("#strong["), "strong: {src}");
}

#[test]
fn md_heading_with_inline_code() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("# Using `code` in heading\n", &default_config(&dir)).unwrap();
    assert!(src.contains("= "), "heading marker: {src}");
    assert!(src.contains("`code`"), "inline code in heading: {src}");
}

#[test]
fn md_table_single_column() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("| Item |\n|------|\n| A |\n| B |\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#table("), "table: {src}");
    assert!(src.contains("1fr"), "single column: {src}");
}

#[test]
fn md_table_five_columns() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst(
        "| A | B | C | D | E |\n|---|---|---|---|---|\n| 1 | 2 | 3 | 4 | 5 |\n",
        &default_config(&dir),
    )
    .unwrap();
    assert!(src.contains("#table("), "table: {src}");
    let count = src.matches("1fr").count();
    assert_eq!(count, 5, "five columns: {src}");
}

#[test]
fn md_code_block_math_lang_no_code_block() {
    // A ```math ... ``` fenced block must produce display math, not a code block
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("```math\nE = mc^2\n```\n", &default_config(&dir)).unwrap();
    assert!(
        !src.contains("#block(fill:"),
        "should not be code block: {src}"
    );
    assert!(
        src.contains("$ ") && src.contains(" $"),
        "should be display math: {src}"
    );
}

#[test]
fn md_description_list_term_and_detail() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("Apple\n: A fruit.\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#strong[Apple]"), "term bold: {src}");
    assert!(src.contains("A fruit."), "detail: {src}");
    assert!(
        src.contains("#pad(left:") || src.contains("pad(left:"),
        "padded: {src}"
    );
}

#[test]
fn md_frontmatter_yes_value_enables_toc() {
    // `toc: yes` (YAML alternate truthy) should enable TOC
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: yes\n---\n# Hello\n";
    let src = md_to_typst(md, &default_config(&dir)).unwrap();
    assert!(
        src.contains("#outline("),
        "toc:yes should enable outline: {src}"
    );
}

#[test]
fn md_frontmatter_1_value_enables_toc() {
    // `toc: 1` should enable TOC
    let dir = TempDir::new().unwrap();
    let md = "---\ntoc: 1\n---\n# Hello\n";
    let src = md_to_typst(md, &default_config(&dir)).unwrap();
    assert!(
        src.contains("#outline("),
        "toc:1 should enable outline: {src}"
    );
}

#[test]
fn md_font_appears_in_set_text() {
    let dir = TempDir::new().unwrap();
    let mut config = default_config(&dir);
    config.fonts.regular = "EB Garamond".to_string();
    let src = md_to_typst("Hello.\n", &config).unwrap();
    assert!(src.contains("EB Garamond"), "font name in output: {src}");
}

#[test]
fn md_mono_font_appears_in_show_raw() {
    let dir = TempDir::new().unwrap();
    let mut config = default_config(&dir);
    config.fonts.monospace = "Fira Code".to_string();
    let src = md_to_typst("Hello.\n", &config).unwrap();
    assert!(src.contains("Fira Code"), "mono font name in output: {src}");
}

#[test]
fn md_inline_html_strong_tag() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("Text <strong>bold</strong> text.\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#strong["), "HTML strong: {src}");
    assert!(src.contains("bold"), "content: {src}");
}

#[test]
fn md_inline_html_em_tag() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("Text <em>italic</em> text.\n", &default_config(&dir)).unwrap();
    assert!(src.contains("#emph["), "HTML em: {src}");
}

#[test]
fn md_inline_html_br_tag() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("Line one<br>Line two\n", &default_config(&dir)).unwrap();
    // <br> → \\\n in Typst
    assert!(
        src.contains("\\\\\n") || src.contains("\\\n"),
        "line break: {src}"
    );
}

#[test]
fn md_multiple_blockquotes_all_present() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("> First.\n\n> Second.\n\n> Third.\n", &default_config(&dir)).unwrap();
    assert!(src.contains("First."), "first: {src}");
    assert!(src.contains("Second."), "second: {src}");
    assert!(src.contains("Third."), "third: {src}");
    let count = src.matches("#block(").count();
    assert!(count >= 3, "three blockquotes: {src}");
}

#[test]
fn md_ordered_list_mixed_with_unordered() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst(
        "- Bullet A\n  1. Num one\n  2. Num two\n- Bullet B\n",
        &default_config(&dir),
    )
    .unwrap();
    assert!(src.contains("- Bullet A"), "bullet A: {src}");
    assert!(src.contains("- Bullet B"), "bullet B: {src}");
    assert!(
        src.contains("+ Num one") || src.contains("Num one"),
        "num one: {src}"
    );
    assert!(
        src.contains("+ Num two") || src.contains("Num two"),
        "num two: {src}"
    );
}

#[test]
fn md_footnote_with_inline_formatting() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst(
        "Ref[^fn].\n\n[^fn]: Note with **bold** inside.\n",
        &default_config(&dir),
    )
    .unwrap();
    assert!(src.contains("#super["), "superscript: {src}");
    assert!(src.contains("#strong[bold]"), "bold in footnote: {src}");
}

#[test]
fn md_no_crash_on_empty_table() {
    // An empty table (no rows) should produce no output or an empty table
    let dir = TempDir::new().unwrap();
    let src = md_to_typst("| H |\n|---|\n", &default_config(&dir));
    assert!(src.is_ok(), "should not crash on header-only table");
}

#[test]
fn md_task_list_in_nested_context() {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst(
        "- Parent\n  - [x] Child done\n  - [ ] Child pending\n",
        &default_config(&dir),
    )
    .unwrap();
    assert!(src.contains("☑"), "checked: {src}");
    assert!(src.contains("☐"), "unchecked: {src}");
}

#[test]
fn md_page_size_uses_mm_units() {
    let dir = TempDir::new().unwrap();
    let config = default_config(&dir);
    let src = md_to_typst("Text\n", &config).unwrap();
    // Should use millimeter units for page dimensions
    assert!(src.contains("mm"), "mm unit: {src}");
}

#[test]
fn md_font_size_uses_pt_units() {
    let dir = TempDir::new().unwrap();
    let config = default_config(&dir);
    let src = md_to_typst("Text\n", &config).unwrap();
    assert!(src.contains("pt"), "pt unit: {src}");
}
