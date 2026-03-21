//! Integration tests: Markdown → Typst source round-trips.
//!
//! Each test verifies that specific Markdown constructs are translated to
//! the correct Typst markup.  No PDF compilation happens here — these tests
//! are fast and deterministic.

use md_to_pdf::renderer::{FontSet, RenderConfig, markdown_to_typst_pub as md_to_typst};
use tempfile::TempDir;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn cfg(dir: &TempDir) -> RenderConfig {
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

fn convert(md: &str) -> String {
    let dir = TempDir::new().unwrap();
    md_to_typst(md, &cfg(&dir)).unwrap()
}

// ─────────────────────────────────────────────────────────────────────────────
// Headings
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn heading_h1_uses_single_equals() {
    let src = convert("# Hello\n");
    assert!(src.contains("= Hello"), "got:\n{src}");
}

#[test]
fn heading_h2_uses_two_equals() {
    let src = convert("## Section\n");
    assert!(src.contains("== Section"), "got:\n{src}");
}

#[test]
fn heading_h6_uses_six_equals() {
    let src = convert("###### Deep\n");
    assert!(src.contains("====== Deep"), "got:\n{src}");
}

#[test]
fn heading_emits_label() {
    let src = convert("# Getting Started\n");
    // Heading should have a <label> for TOC cross-links
    assert!(src.contains("<getting-started>"), "got:\n{src}");
}

#[test]
fn heading_bold_in_title() {
    let src = convert("## **Bold** Heading\n");
    assert!(src.contains("=="), "got:\n{src}");
    assert!(src.contains("Bold") || src.contains("strong"), "got:\n{src}");
}

#[test]
fn headings_all_levels_present() {
    let md = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\n";
    let src = convert(md);
    assert!(src.contains("= H1"), "H1: {src}");
    assert!(src.contains("== H2"), "H2: {src}");
    assert!(src.contains("=== H3"), "H3: {src}");
    assert!(src.contains("==== H4"), "H4: {src}");
    assert!(src.contains("===== H5"), "H5: {src}");
    assert!(src.contains("====== H6"), "H6: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Paragraphs & inline formatting
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn paragraph_emits_text() {
    let src = convert("Hello, world!\n");
    assert!(src.contains("Hello, world!"), "got:\n{src}");
}

#[test]
fn bold_becomes_strong() {
    let src = convert("**bold**\n");
    assert!(src.contains("#strong[bold]"), "got:\n{src}");
}

#[test]
fn italic_becomes_emph() {
    let src = convert("*italic*\n");
    assert!(src.contains("#emph[italic]"), "got:\n{src}");
}

#[test]
fn strikethrough_becomes_strike() {
    let src = convert("~~struck~~\n");
    assert!(src.contains("#strike[struck]"), "got:\n{src}");
}

#[test]
fn inline_code_backtick() {
    let src = convert("`foo()`\n");
    assert!(src.contains("`foo()`"), "got:\n{src}");
}

#[test]
fn link_with_label() {
    let src = convert("[Rust](https://www.rust-lang.org)\n");
    assert!(src.contains("#link(\"https://www.rust-lang.org\", [Rust])"), "got:\n{src}");
}

#[test]
fn autolink_compact_form() {
    // When label == URL, emit compact #link(url)
    let src = convert("<https://example.com>\n");
    // Either compact #link("…") or with label — both are valid; just ensure link is present
    assert!(src.contains("#link("), "got:\n{src}");
    assert!(src.contains("example.com"), "got:\n{src}");
}

#[test]
fn typst_special_chars_escaped_in_text() {
    let src = convert("Price: $9.99 and #hashtag\n");
    // Dollar and hash must be escaped so Typst doesn't treat them as math/code
    assert!(src.contains("\\$9.99") || src.contains("\\#hashtag"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Blockquotes
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn blockquote_uses_block_inset() {
    let src = convert("> A quote.\n");
    assert!(src.contains("#block("), "got:\n{src}");
    assert!(src.contains("inset"), "got:\n{src}");
    assert!(src.contains("stroke"), "got:\n{src}");
}

#[test]
fn blockquote_preserves_content() {
    let src = convert("> Something important\n");
    assert!(src.contains("Something important"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Unordered lists
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn bullet_list_uses_dash_marker() {
    let src = convert("- Alpha\n- Beta\n");
    assert!(src.contains("- Alpha"), "got:\n{src}");
    assert!(src.contains("- Beta"), "got:\n{src}");
}

#[test]
fn nested_bullet_list_indented() {
    let src = convert("- Parent\n  - Child\n");
    // Child should be indented relative to parent
    assert!(src.contains("  - Child") || src.contains("- Child"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Ordered lists
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ordered_list_uses_plus_marker() {
    let src = convert("1. First\n2. Second\n3. Third\n");
    assert!(src.contains("+ First"), "got:\n{src}");
    assert!(src.contains("+ Second"), "got:\n{src}");
    assert!(src.contains("+ Third"), "got:\n{src}");
}

#[test]
fn ordered_list_non_one_start() {
    // Lists can start from non-1
    let src = convert("5. Fifth\n6. Sixth\n");
    assert!(src.contains("+ Fifth"), "got:\n{src}");
    assert!(src.contains("+ Sixth"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Task lists
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn task_list_checked_uses_ballot_box_checked() {
    let src = convert("- [x] Done\n");
    assert!(src.contains("☑"), "expected checked ballot box, got:\n{src}");
}

#[test]
fn task_list_unchecked_uses_ballot_box_empty() {
    let src = convert("- [ ] Todo\n");
    assert!(src.contains("☐"), "expected empty ballot box, got:\n{src}");
}

#[test]
fn task_list_mixed_items() {
    let src = convert("- [x] Done\n- [ ] Pending\n");
    assert!(src.contains("☑"), "got:\n{src}");
    assert!(src.contains("☐"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Horizontal rule
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn thematic_break_becomes_line() {
    let src = convert("---\n");
    assert!(src.contains("#line(length: 100%)"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Tables
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn table_emits_typst_table_call() {
    let src = convert("| A | B |\n|---|---|\n| 1 | 2 |\n");
    assert!(src.contains("#table("), "got:\n{src}");
}

#[test]
fn table_header_row_has_strong() {
    let src = convert("| Name | Score |\n|------|-------|\n| Bob | 90 |\n");
    // Header cells are wrapped in #strong[…]
    assert!(src.contains("#strong["), "got:\n{src}");
}

#[test]
fn table_column_count_in_columns_spec() {
    let src = convert("| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n");
    // Three 1fr columns
    assert!(src.contains("1fr"), "got:\n{src}");
    // Should appear 3 times (one per column)
    let count = src.matches("1fr").count();
    assert_eq!(count, 3, "expected 3 columns, got {count} in:\n{src}");
}

#[test]
fn table_alignment_emits_per_cell_align() {
    let src = convert("| Left | Center | Right |\n|:-----|:------:|------:|\n| a | b | c |\n");
    assert!(src.contains("align: left"), "got:\n{src}");
    assert!(src.contains("align: center"), "got:\n{src}");
    assert!(src.contains("align: right"), "got:\n{src}");
}

#[test]
fn table_fill_header_row() {
    let src = convert("| H |\n|---|\n| v |\n");
    // First row gets a different fill
    assert!(src.contains("luma(230)") || src.contains("luma(200)"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Code blocks
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn code_block_rust_emits_block() {
    let src = convert("```rust\nfn main() {}\n```\n");
    assert!(src.contains("#block("), "got:\n{src}");
    // Syntax-highlighted blocks wrap each token in #text(…)[…]; confirm `fn` token appears
    assert!(src.contains("[fn]") || src.contains("fn"), "got:\n{src}");
}

#[test]
fn code_block_uses_mono_font() {
    let src = convert("```python\nprint('hi')\n```\n");
    assert!(src.contains("DejaVu Sans Mono"), "got:\n{src}");
}

#[test]
fn code_block_plain_no_lang() {
    let src = convert("```\nplain text\n```\n");
    assert!(src.contains("plain text"), "got:\n{src}");
    assert!(src.contains("#block("), "got:\n{src}");
}

#[test]
fn code_block_math_lang_is_display_math() {
    let src = convert("```math\nE = mc^2\n```\n");
    // A fenced math block → Typst display math `$ … $`
    assert!(src.contains("$ ") && src.contains(" $"), "got:\n{src}");
    assert!(!src.contains("#block("), "fenced math should NOT produce a code block:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Math (inline and display)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn inline_math_no_surrounding_spaces() {
    let src = convert("The value $x + y$ is positive.\n");
    // Inline math emitted as $…$ with no extra spaces
    assert!(src.contains("$x + y$") || src.contains("$x+y$"), "got:\n{src}");
}

#[test]
fn display_math_block_format() {
    let src = convert("$$a^2 + b^2 = c^2$$\n");
    // Display math has spaces: `$ expr $`
    assert!(src.contains("$ ") && src.contains(" $"), "got:\n{src}");
}

#[test]
fn math_frac_translated() {
    let src = convert("$\\frac{1}{2}$\n");
    assert!(src.contains("frac(1, 2)") || src.contains("frac("), "got:\n{src}");
}

#[test]
fn math_sqrt_translated() {
    let src = convert("$\\sqrt{x}$\n");
    assert!(src.contains("sqrt(x)") || src.contains("sqrt("), "got:\n{src}");
}

#[test]
fn math_greek_letters_translated() {
    let src = convert("$\\alpha + \\beta$\n");
    assert!(src.contains("alpha") && src.contains("beta"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Footnotes
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn footnote_reference_emits_superscript() {
    let src = convert("Text[^1].\n\n[^1]: Footnote.\n");
    assert!(src.contains("#super["), "got:\n{src}");
}

#[test]
fn footnote_definition_emitted_at_end() {
    let src = convert("Para[^fn].\n\n[^fn]: The body.\n");
    // Footnote section separator
    assert!(src.contains("#line(length: 100%)"), "got:\n{src}");
    // Body text
    assert!(src.contains("The body."), "got:\n{src}");
}

#[test]
fn multiple_footnotes_all_emitted() {
    let src = convert("A[^a] and B[^b].\n\n[^a]: First.\n\n[^b]: Second.\n");
    assert!(src.contains("First."), "got:\n{src}");
    assert!(src.contains("Second."), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Description lists
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn description_term_is_bold() {
    let src = convert("Term\n: Definition\n");
    assert!(src.contains("#strong[Term]"), "got:\n{src}");
}

#[test]
fn description_details_are_padded() {
    let src = convert("Term\n: Definition\n");
    assert!(src.contains("#pad(left:") || src.contains("pad(left:"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Images
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn missing_local_image_emits_fallback() {
    let src = convert("![alt](no-such-file.png)\n");
    assert!(src.contains("#block("), "got:\n{src}");
    assert!(src.contains("\\[Image:"), "got:\n{src}");
}

#[test]
fn remote_image_disabled_emits_fallback() {
    let src = convert("![Cloud](https://example.com/photo.png)\n");
    assert!(!src.contains("image(\"https://"), "should not emit remote URL:\n{src}");
    assert!(src.contains("#block("), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// TOC flag behaviour
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn toc_false_no_outline_emitted() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.toc = false;
    config.toc_explicit = true;
    let src = md_to_typst("# Hello\n## World\n", &config).unwrap();
    assert!(!src.contains("#outline("), "got:\n{src}");
}

#[test]
fn toc_true_outline_emitted() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.toc = true;
    config.toc_explicit = true;
    let src = md_to_typst("# Hello\n## World\n", &config).unwrap();
    assert!(src.contains("#outline("), "got:\n{src}");
}

#[test]
fn toc_depth_controls_outline_depth() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.toc = true;
    config.toc_explicit = true;
    config.toc_depth = 2;
    let src = md_to_typst("# H1\n## H2\n### H3\n", &config).unwrap();
    assert!(src.contains("depth: 2"), "got:\n{src}");
}

#[test]
fn toc_emits_pagebreak_after_outline() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.toc = true;
    config.toc_explicit = true;
    let src = md_to_typst("# Hello\n## World\n", &config).unwrap();
    assert!(src.contains("#pagebreak()"), "expected pagebreak after TOC, got:\n{src}");
    // pagebreak must come after the outline block
    let outline_pos = src.find("#outline(").expect("outline missing");
    let break_pos = src.find("#pagebreak()").expect("pagebreak missing");
    assert!(break_pos > outline_pos, "pagebreak should follow outline");
}

#[test]
fn toc_h1_show_rule_emitted() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.toc = true;
    config.toc_explicit = true;
    let src = md_to_typst("# H1\n", &config).unwrap();
    assert!(src.contains("outline.entry.where(level: 1)"),
        "expected H1 bold show rule, got:\n{src}");
}

#[test]
fn toc_frontmatter_no_toc_suppresses_outline() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.toc = true;          // default
    config.toc_explicit = false; // frontmatter can override
    let md = "---\nno_toc: true\n---\n# Hello\n";
    let src = md_to_typst(md, &config).unwrap();
    assert!(!src.contains("#outline("),
        "no_toc: true in frontmatter should suppress TOC, got:\n{src}");
}

#[test]
fn toc_frontmatter_toc_title_sets_custom_title() {
    let dir = TempDir::new().unwrap();
    let config = cfg(&dir);
    let md = "---\ntoc: true\ntoc_title: Document Guide\n---\n# Hello\n";
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("Document Guide"),
        "expected custom toc_title in output, got:\n{src}");
}

#[test]
fn toc_heading_labels_deduplicated() {
    let dir = TempDir::new().unwrap();
    let config = cfg(&dir);
    let md = "# Intro\n\n# Intro\n\n# Intro\n";
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("<intro>"),   "first: <intro>, got:\n{src}");
    assert!(src.contains("<intro-2>"), "second: <intro-2>, got:\n{src}");
    assert!(src.contains("<intro-3>"), "third: <intro-3>, got:\n{src}");
    // Base label should appear exactly once
    assert_eq!(src.matches("<intro>").count(), 1, "should have exactly one <intro>, got:\n{src}");
}

#[test]
fn toc_heading_all_levels_have_labels() {
    let dir = TempDir::new().unwrap();
    let config = cfg(&dir);
    let md = "# A\n## B\n### C\n#### D\n##### E\n###### F\n";
    let src = md_to_typst(md, &config).unwrap();
    for (marker, label) in &[("= A", "<a>"), ("== B", "<b>"), ("=== C", "<c>"),
                               ("==== D", "<d>"), ("===== E", "<e>"), ("====== F", "<f>")] {
        assert!(src.contains(marker), "missing heading: got:\n{src}");
        assert!(src.contains(label), "missing label {label}: got:\n{src}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sample fixture files (round-trip: fixture file → Typst → no error)
// ─────────────────────────────────────────────────────────────────────────────

fn convert_fixture(filename: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/samples")
        .join(filename);
    let md = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Cannot read fixture: {}", path.display()));
    let dir = TempDir::new().unwrap();
    md_to_typst(&md, &cfg(&dir)).unwrap()
}

#[test]
fn fixture_basic_converts_without_error() {
    let src = convert_fixture("basic.md");
    // Has headings, lists, tables, code blocks
    assert!(src.contains("= Basic Markdown Features"), "got:\n{src}");
    assert!(src.contains("#table("), "got:\n{src}");
    assert!(src.contains("#block("), "got:\n{src}");
}

#[test]
fn fixture_math_converts_without_error() {
    let src = convert_fixture("math.md");
    assert!(src.contains("frac("), "got:\n{src}");
    assert!(src.contains("sqrt("), "got:\n{src}");
}

#[test]
fn fixture_toc_emits_outline() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/samples/toc.md");
    let md = std::fs::read_to_string(path).unwrap();
    let dir = TempDir::new().unwrap();
    // Let frontmatter control TOC (toc_explicit = false)
    let config = RenderConfig {
        toc: false,
        toc_explicit: false,
        ..cfg(&dir)
    };
    let src = md_to_typst(&md, &config).unwrap();
    assert!(src.contains("#outline("), "frontmatter toc:true should produce outline:\n{src}");
    assert!(src.contains("depth: 3"), "frontmatter toc_depth:3 should set depth:\n{src}");
}

#[test]
fn fixture_special_chars_converts_without_error() {
    let src = convert_fixture("special_chars.md");
    // Should compile without panicking; key content present
    assert!(!src.is_empty());
}

#[test]
fn fixture_footnotes_converts_without_error() {
    let src = convert_fixture("footnotes.md");
    // Footnote section separator
    assert!(src.contains("#line(length: 100%)"), "got:\n{src}");
}

#[test]
fn fixture_code_blocks_converts_without_error() {
    let src = convert_fixture("code_blocks.md");
    assert!(src.contains("#block("), "got:\n{src}");
    // `fn` and `greet` appear as separate highlighted tokens; just verify block exists
    assert!(src.contains("[fn]") || src.contains("fn"), "got:\n{src}");
}

#[test]
fn fixture_tables_converts_without_error() {
    let src = convert_fixture("tables.md");
    assert!(src.contains("#table("), "got:\n{src}");
    assert!(src.contains("align: center"), "got:\n{src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM fidelity — nested lists
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn nested_bullet_three_levels_clean() {
    let src = convert("- Top\n  - Mid\n    - Deep\n");
    assert!(src.contains("- Top"), "top level: {src}");
    assert!(src.contains("  - Mid"), "mid level: {src}");
    assert!(src.contains("    - Deep"), "deep level: {src}");
    // No blank line between parent item and its nested list
    assert!(
        !src.contains("Top\n\n  - Mid"),
        "spurious blank line before nested items: {src}"
    );
}

#[test]
fn nested_mixed_bullet_and_ordered() {
    let src = convert("- Category\n  1. Sub one\n  2. Sub two\n");
    assert!(src.contains("- Category"), "bullet parent: {src}");
    assert!(src.contains("  + Sub one") || src.contains("  + Sub"), "ordered child: {src}");
}

#[test]
fn nested_ordered_sublist_clean() {
    let src = convert("1. Alpha\n2. Beta\n   1. Sub-beta-one\n   2. Sub-beta-two\n3. Gamma\n");
    assert!(src.contains("+ Alpha"), "alpha: {src}");
    assert!(src.contains("+ Beta"), "beta: {src}");
    assert!(src.contains("  + Sub-beta-one") || src.contains("+ Sub-beta-one"), "sub: {src}");
    assert!(src.contains("+ Gamma"), "gamma: {src}");
    // No blank line between 'Beta' and its nested list
    assert!(
        !src.contains("Beta\n\n  + Sub"),
        "blank line before nested ordered: {src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM fidelity — ordered list start numbering
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ordered_list_default_start_one_no_set_enum() {
    // Lists starting at 1 should NOT emit a #set enum(start:) directive
    let src = convert("1. First\n2. Second\n3. Third\n");
    assert!(!src.contains("set enum(start:)"), "start:1 should not emit set enum: {src}");
    assert!(src.contains("+ First"), "items present: {src}");
}

#[test]
fn ordered_list_start_at_5() {
    let src = convert("5. Five\n6. Six\n7. Seven\n");
    assert!(
        src.contains("start: 5") || src.contains("start:5"),
        "expected start:5 directive: {src}"
    );
    assert!(src.contains("+ Five"), "item text: {src}");
}

#[test]
fn ordered_list_start_at_1_explicit() {
    // `1. First` explicitly = start 1; should not wrap in block
    let src = convert("1. One\n2. Two\n");
    assert!(!src.contains("#block[\n#set enum"), "no wrapper for start=1: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM fidelity — task list layout
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn task_list_uses_box_checkbox() {
    let src = convert("- [x] Alpha\n- [ ] Beta\n");
    // Checkbox is an inline #box(width: 1em)[…]
    assert!(src.contains("#box(width: 1em)[☑]"), "checked: {src}");
    assert!(src.contains("#box(width: 1em)[☐]"), "unchecked: {src}");
}

#[test]
fn task_list_no_double_space_before_text() {
    // The old rendering emitted `- ☑  text` (double space); now it must be single
    let src = convert("- [x] Item text\n");
    // Should contain `[☑] Item text` without double space
    assert!(
        src.contains("[☑] Item text"),
        "single space after checkbox: {src}"
    );
    assert!(
        !src.contains("[☑]  Item text"),
        "no double space after checkbox: {src}"
    );
}

#[test]
fn nested_task_list_indented_correctly() {
    let src = convert("- [x] Parent\n  - [ ] Child\n");
    assert!(src.contains("- #box(width: 1em)[☑] Parent"), "parent: {src}");
    assert!(
        src.contains("  - #box(width: 1em)[☐] Child"),
        "indented child task: {src}"
    );
    // No blank line between parent and nested task
    assert!(
        !src.contains("Parent\n\n  - "),
        "blank line before nested task: {src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM fidelity — table alignment markers
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn table_header_cells_bolded_directly() {
    // Header cells must use #strong[…] directly in the cell content, not via
    // fragile string replacement of )[
    let src = convert("| H1 | H2 |\n|----|----|\n| d1 | d2 |\n");
    assert!(src.contains("table.cell(align: left)[#strong[H1]]"), "header cell: {src}");
    assert!(src.contains("table.cell(align: left)[#strong[H2]]"), "header cell: {src}");
    // Data cells must NOT be bold
    assert!(src.contains("table.cell(align: left)[d1]"), "data cell: {src}");
}

#[test]
fn table_all_three_alignments() {
    let src = convert("| L | C | R |\n|:--|:-:|--:|\n| a | b | c |\n");
    // Header row
    assert!(src.contains("table.cell(align: left)[#strong[L]]"), "left header: {src}");
    assert!(src.contains("table.cell(align: center)[#strong[C]]"), "center header: {src}");
    assert!(src.contains("table.cell(align: right)[#strong[R]]"), "right header: {src}");
    // Data row
    assert!(src.contains("table.cell(align: left)[a]"), "left data: {src}");
    assert!(src.contains("table.cell(align: center)[b]"), "center data: {src}");
    assert!(src.contains("table.cell(align: right)[c]"), "right data: {src}");
}

#[test]
fn table_no_alignment_defaults_to_left() {
    let src = convert("| A | B |\n|---|---|\n| 1 | 2 |\n");
    // Without alignment markers, all cells default to left
    assert!(src.contains("align: left"), "default left: {src}");
    assert!(!src.contains("align: right"), "no right without marker: {src}");
    assert!(!src.contains("align: center"), "no center without marker: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM fidelity — autolinks
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn bare_url_autolink_compact_form() {
    // comrak's autolink extension: bare `https://…` in text → Link node where label == url
    let src = convert("Go to https://example.com today.\n");
    // Compact form: #link("url") without a separate label
    assert!(src.contains("#link(\"https://example.com\")"), "compact autolink: {src}");
}

#[test]
fn angle_bracket_url_autolink() {
    let src = convert("See <https://rust-lang.org>.\n");
    assert!(src.contains("#link("), "link call: {src}");
    assert!(src.contains("rust-lang.org"), "URL in output: {src}");
}

#[test]
fn email_autolink_uses_mailto() {
    let src = convert("Mail to <hello@example.com>.\n");
    assert!(src.contains("mailto:hello@example.com"), "mailto: {src}");
    // @ in display label must be escaped
    assert!(src.contains("hello\\@example.com"), "escaped @: {src}");
}

#[test]
fn explicit_link_keeps_label() {
    let src = convert("[GitHub](https://github.com)\n");
    assert!(
        src.contains("#link(\"https://github.com\", [GitHub])"),
        "explicit label retained: {src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM fidelity — footnotes
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn footnote_single_reference_and_definition() {
    let src = convert("Text[^note].\n\n[^note]: The note body.\n");
    assert!(src.contains("#super[1]"), "superscript: {src}");
    assert!(src.contains("The note body."), "body: {src}");
    assert!(src.contains("#line(length: 100%)"), "separator: {src}");
}

#[test]
fn footnote_two_in_document_order() {
    // [^a] is referenced first, [^b] second.
    let src = convert("First[^a] and second[^b].\n\n[^a]: Alpha.\n\n[^b]: Beta.\n");
    let pos_super1 = src.find("#super[1]").expect("super[1] missing");
    let pos_super2 = src.find("#super[2]").expect("super[2] missing");
    // In-text references: 1 comes before 2
    assert!(pos_super1 < pos_super2, "super[1] should precede super[2]: {src}");
    // Definitions section: Alpha (first-referenced) before Beta
    let footer = src.split("#line(length: 100%)").nth(1).unwrap_or("");
    assert!(footer.contains("Alpha."), "Alpha in footer: {src}");
    assert!(footer.contains("Beta."), "Beta in footer: {src}");
    let alpha_pos = footer.find("Alpha.").unwrap();
    let beta_pos = footer.find("Beta.").unwrap();
    assert!(alpha_pos < beta_pos, "Alpha should come before Beta in footnote section: {src}");
}

#[test]
fn footnote_definition_before_reference_still_ordered() {
    // Definition appears before its reference in source — footnotes still ordered
    // by first reference in text, not by definition source position.
    let src = convert("[^z]: Zeta definition.\n\nText[^z] here.\n");
    assert!(src.contains("#super[1]"), "super[1]: {src}");
    assert!(src.contains("Zeta definition."), "body: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM fidelity — definition lists
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn definition_list_single_term_single_detail() {
    let src = convert("Term\n: Detail.\n");
    assert!(src.contains("#strong[Term]"), "term: {src}");
    assert!(src.contains("#pad(left: 1.5em)[Detail.]"), "detail: {src}");
}

#[test]
fn definition_list_single_term_two_details() {
    let src = convert("Term\n: First.\n: Second.\n");
    assert!(src.contains("#strong[Term]"), "term: {src}");
    assert!(src.contains("First."), "first: {src}");
    assert!(src.contains("Second."), "second: {src}");
    let pads = src.matches("#pad(left: 1.5em)").count();
    assert!(pads >= 2, "both details padded ({pads} pads): {src}");
}

#[test]
fn definition_list_two_separate_entries() {
    let src = convert("Alpha\n: Def of alpha.\n\nBeta\n: Def of beta.\n");
    assert!(src.contains("#strong[Alpha]"), "Alpha: {src}");
    assert!(src.contains("#strong[Beta]"), "Beta: {src}");
    assert!(src.contains("Def of alpha."), "alpha body: {src}");
    assert!(src.contains("Def of beta."), "beta body: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM fixture file round-trip
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn gfm_fixture_converts_without_error() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("gfm-fixture.md");
    let md = std::fs::read_to_string(path).expect("gfm-fixture.md must exist");
    let dir = TempDir::new().unwrap();
    let src = md_to_typst(&md, &cfg(&dir)).unwrap();
    assert!(!src.is_empty(), "GFM fixture should produce non-empty output");
    // All major sections present
    assert!(src.contains("#table("), "table: {src}");
    assert!(src.contains("#box(width: 1em)"), "task list: {src}");
    assert!(src.contains("#link("), "autolink: {src}");
    assert!(src.contains("#line(length: 100%)"), "footnote section: {src}");
    assert!(src.contains("#pad(left: 1.5em)"), "definition list: {src}");
}
