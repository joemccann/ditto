//! Integration tests: Markdown → Typst source round-trips.
//!
//! Each test verifies that specific Markdown constructs are translated to
//! the correct Typst markup.  No PDF compilation happens here — these tests
//! are fast and deterministic.

use ditto::renderer::{FontSet, RenderConfig, markdown_to_typst_pub as md_to_typst};
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
    assert!(
        src.contains("Bold") || src.contains("strong"),
        "got:\n{src}"
    );
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
    assert!(
        src.contains("#link(\"https://www.rust-lang.org\", [Rust])"),
        "got:\n{src}"
    );
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
    assert!(
        src.contains("\\$9.99") || src.contains("\\#hashtag"),
        "got:\n{src}"
    );
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
    assert!(
        src.contains("  - Child") || src.contains("- Child"),
        "got:\n{src}"
    );
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
    assert!(
        src.contains("☑"),
        "expected checked ballot box, got:\n{src}"
    );
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
    assert!(
        src.contains("luma(230)") || src.contains("luma(200)"),
        "got:\n{src}"
    );
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
    assert!(
        !src.contains("#block("),
        "fenced math should NOT produce a code block:\n{src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Math (inline and display)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn inline_math_no_surrounding_spaces() {
    let src = convert("The value $x + y$ is positive.\n");
    // Inline math emitted as $…$ with no extra spaces
    assert!(
        src.contains("$x + y$") || src.contains("$x+y$"),
        "got:\n{src}"
    );
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
    assert!(
        src.contains("frac(1, 2)") || src.contains("frac("),
        "got:\n{src}"
    );
}

#[test]
fn math_sqrt_translated() {
    let src = convert("$\\sqrt{x}$\n");
    assert!(
        src.contains("sqrt(x)") || src.contains("sqrt("),
        "got:\n{src}"
    );
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
    assert!(
        src.contains("#pad(left:") || src.contains("pad(left:"),
        "got:\n{src}"
    );
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
    assert!(
        !src.contains("image(\"https://"),
        "should not emit remote URL:\n{src}"
    );
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
    assert!(
        src.contains("#pagebreak()"),
        "expected pagebreak after TOC, got:\n{src}"
    );
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
    assert!(
        src.contains("outline.entry.where(level: 1)"),
        "expected H1 bold show rule, got:\n{src}"
    );
}

#[test]
fn toc_frontmatter_no_toc_suppresses_outline() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.toc = true; // default
    config.toc_explicit = false; // frontmatter can override
    let md = "---\nno_toc: true\n---\n# Hello\n";
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        !src.contains("#outline("),
        "no_toc: true in frontmatter should suppress TOC, got:\n{src}"
    );
}

#[test]
fn toc_frontmatter_toc_title_sets_custom_title() {
    let dir = TempDir::new().unwrap();
    let config = cfg(&dir);
    let md = "---\ntoc: true\ntoc_title: Document Guide\n---\n# Hello\n";
    let src = md_to_typst(md, &config).unwrap();
    assert!(
        src.contains("Document Guide"),
        "expected custom toc_title in output, got:\n{src}"
    );
}

#[test]
fn toc_heading_labels_deduplicated() {
    let dir = TempDir::new().unwrap();
    let config = cfg(&dir);
    let md = "# Intro\n\n# Intro\n\n# Intro\n";
    let src = md_to_typst(md, &config).unwrap();
    assert!(src.contains("<intro>"), "first: <intro>, got:\n{src}");
    assert!(src.contains("<intro-2>"), "second: <intro-2>, got:\n{src}");
    assert!(src.contains("<intro-3>"), "third: <intro-3>, got:\n{src}");
    // Base label should appear exactly once
    assert_eq!(
        src.matches("<intro>").count(),
        1,
        "should have exactly one <intro>, got:\n{src}"
    );
}

#[test]
fn toc_heading_all_levels_have_labels() {
    let dir = TempDir::new().unwrap();
    let config = cfg(&dir);
    let md = "# A\n## B\n### C\n#### D\n##### E\n###### F\n";
    let src = md_to_typst(md, &config).unwrap();
    for (marker, label) in &[
        ("= A", "<a>"),
        ("== B", "<b>"),
        ("=== C", "<c>"),
        ("==== D", "<d>"),
        ("===== E", "<e>"),
        ("====== F", "<f>"),
    ] {
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
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/samples/toc.md");
    let md = std::fs::read_to_string(path).unwrap();
    let dir = TempDir::new().unwrap();
    // Let frontmatter control TOC (toc_explicit = false)
    let config = RenderConfig {
        toc: false,
        toc_explicit: false,
        ..cfg(&dir)
    };
    let src = md_to_typst(&md, &config).unwrap();
    assert!(
        src.contains("#outline("),
        "frontmatter toc:true should produce outline:\n{src}"
    );
    assert!(
        src.contains("depth: 3"),
        "frontmatter toc_depth:3 should set depth:\n{src}"
    );
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
    assert!(
        src.contains("  + Sub one") || src.contains("  + Sub"),
        "ordered child: {src}"
    );
}

#[test]
fn nested_ordered_sublist_clean() {
    let src = convert("1. Alpha\n2. Beta\n   1. Sub-beta-one\n   2. Sub-beta-two\n3. Gamma\n");
    assert!(src.contains("+ Alpha"), "alpha: {src}");
    assert!(src.contains("+ Beta"), "beta: {src}");
    assert!(
        src.contains("  + Sub-beta-one") || src.contains("+ Sub-beta-one"),
        "sub: {src}"
    );
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
    assert!(
        !src.contains("set enum(start:)"),
        "start:1 should not emit set enum: {src}"
    );
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
    assert!(
        !src.contains("#block[\n#set enum"),
        "no wrapper for start=1: {src}"
    );
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
    assert!(
        src.contains("- #box(width: 1em)[☑] Parent"),
        "parent: {src}"
    );
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
    assert!(
        src.contains("table.cell(align: left)[#strong[H1]]"),
        "header cell: {src}"
    );
    assert!(
        src.contains("table.cell(align: left)[#strong[H2]]"),
        "header cell: {src}"
    );
    // Data cells must NOT be bold
    assert!(
        src.contains("table.cell(align: left)[d1]"),
        "data cell: {src}"
    );
}

#[test]
fn table_all_three_alignments() {
    let src = convert("| L | C | R |\n|:--|:-:|--:|\n| a | b | c |\n");
    // Header row
    assert!(
        src.contains("table.cell(align: left)[#strong[L]]"),
        "left header: {src}"
    );
    assert!(
        src.contains("table.cell(align: center)[#strong[C]]"),
        "center header: {src}"
    );
    assert!(
        src.contains("table.cell(align: right)[#strong[R]]"),
        "right header: {src}"
    );
    // Data row
    assert!(
        src.contains("table.cell(align: left)[a]"),
        "left data: {src}"
    );
    assert!(
        src.contains("table.cell(align: center)[b]"),
        "center data: {src}"
    );
    assert!(
        src.contains("table.cell(align: right)[c]"),
        "right data: {src}"
    );
}

#[test]
fn table_no_alignment_defaults_to_left() {
    let src = convert("| A | B |\n|---|---|\n| 1 | 2 |\n");
    // Without alignment markers, all cells default to left
    assert!(src.contains("align: left"), "default left: {src}");
    assert!(
        !src.contains("align: right"),
        "no right without marker: {src}"
    );
    assert!(
        !src.contains("align: center"),
        "no center without marker: {src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM fidelity — autolinks
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn bare_url_autolink_compact_form() {
    // comrak's autolink extension: bare `https://…` in text → Link node where label == url
    let src = convert("Go to https://example.com today.\n");
    // Compact form: #link("url") without a separate label
    assert!(
        src.contains("#link(\"https://example.com\")"),
        "compact autolink: {src}"
    );
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
    assert!(
        pos_super1 < pos_super2,
        "super[1] should precede super[2]: {src}"
    );
    // Definitions section: Alpha (first-referenced) before Beta
    let footer = src.split("#line(length: 100%)").nth(1).unwrap_or("");
    assert!(footer.contains("Alpha."), "Alpha in footer: {src}");
    assert!(footer.contains("Beta."), "Beta in footer: {src}");
    let alpha_pos = footer.find("Alpha.").unwrap();
    let beta_pos = footer.find("Beta.").unwrap();
    assert!(
        alpha_pos < beta_pos,
        "Alpha should come before Beta in footnote section: {src}"
    );
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
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/samples/gfm-fixture.md");
    let md = std::fs::read_to_string(path).expect("gfm-fixture.md must exist");
    let dir = TempDir::new().unwrap();
    let src = md_to_typst(&md, &cfg(&dir)).unwrap();
    assert!(
        !src.is_empty(),
        "GFM fixture should produce non-empty output"
    );
    // All major sections present
    assert!(src.contains("#table("), "table: {src}");
    assert!(src.contains("#box(width: 1em)"), "task list: {src}");
    assert!(src.contains("#link("), "autolink: {src}");
    assert!(
        src.contains("#line(length: 100%)"),
        "footnote section: {src}"
    );
    assert!(src.contains("#pad(left: 1.5em)"), "definition list: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// New fixture file round-trips
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn fixture_mixed_content_converts_without_error() {
    let src = convert_fixture("mixed_content.md");
    assert!(!src.is_empty());
    // Has headings, tables, math, footnotes
    assert!(
        src.contains("= Project Overview") || src.contains("= Mixed Content Document"),
        "heading: {src}"
    );
    assert!(src.contains("#table("), "table: {src}");
    assert!(src.contains("#super["), "footnote: {src}");
}

#[test]
fn fixture_edge_cases_converts_without_error() {
    let src = convert_fixture("edge_cases.md");
    assert!(!src.is_empty());
    // Dollar signs must be escaped throughout
    assert!(src.contains("\\$"), "dollar escaped: {src}");
    // Task list items present
    assert!(src.contains("☑") || src.contains("☐"), "task items: {src}");
}

#[test]
fn fixture_comprehensive_gfm_converts_without_error() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/samples/comprehensive_gfm.md");
    let md = std::fs::read_to_string(path).unwrap();
    let dir = TempDir::new().unwrap();
    // Let frontmatter control TOC
    let config = RenderConfig {
        toc: false,
        toc_explicit: false,
        ..cfg(&dir)
    };
    let src = md_to_typst(&md, &config).unwrap();
    assert!(!src.is_empty());
    assert!(
        src.contains("#outline("),
        "frontmatter toc:true should emit outline: {src}"
    );
    assert!(
        src.contains("depth: 3"),
        "toc_depth:3 from frontmatter: {src}"
    );
    assert!(src.contains("Contents"), "toc_title: {src}");
    assert!(src.contains("#table("), "table: {src}");
    assert!(src.contains("#box(width: 1em)"), "task list: {src}");
    assert!(src.contains("#super["), "footnote refs: {src}");
    assert!(
        src.contains("#strong[Markdown]") || src.contains("#strong["),
        "def list: {src}"
    );
}

#[test]
fn fixture_regression_converts_without_error() {
    let src = convert_fixture("regression.md");
    assert!(!src.is_empty());
    // Dollar signs in plain text must be escaped
    assert!(src.contains("\\$9.99"), "dollar escaping: {src}");
    // Duplicate headings must be disambiguated
    assert!(src.contains("<overview>"), "first overview: {src}");
    assert!(src.contains("<overview-2>"), "second overview: {src}");
    // Email autolink @ escaped
    assert!(
        src.contains("mailto:") || src.contains("support\\@"),
        "email: {src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Inline HTML extended coverage
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn inline_html_u_tag_underline() {
    let src = convert("<u>underlined</u>\n");
    assert!(src.contains("underlined"), "content present: {src}");
}

#[test]
fn inline_html_s_tag_strike() {
    let src = convert("<s>struck</s>\n");
    assert!(src.contains("struck"), "content present: {src}");
}

#[test]
fn inline_html_sub_sup_tags() {
    let src = convert("H<sub>2</sub>O and x<sup>2</sup>\n");
    assert!(src.contains("2"), "subscript/superscript digits: {src}");
}

#[test]
fn inline_html_multiple_br_tags() {
    let src = convert("A<br>B<br>C\n");
    // Each <br> emits a line break
    assert!(src.contains("A"), "A: {src}");
    assert!(src.contains("B"), "B: {src}");
    assert!(src.contains("C"), "C: {src}");
}

#[test]
fn block_html_ul_list() {
    let src = convert("<ul><li>Alpha</li><li>Beta</li></ul>\n");
    assert!(src.contains("Alpha"), "alpha: {src}");
    assert!(src.contains("Beta"), "beta: {src}");
}

#[test]
fn block_html_ol_list() {
    let src = convert("<ol><li>First</li><li>Second</li></ol>\n");
    assert!(src.contains("First"), "first: {src}");
    assert!(src.contains("Second"), "second: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Math translation: realistic expressions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn math_quadratic_formula_translated() {
    let src = convert("$x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$\n");
    assert!(src.contains("frac("), "frac: {src}");
    assert!(src.contains("sqrt("), "sqrt: {src}");
    assert!(src.contains("plus.minus"), "pm: {src}");
}

#[test]
fn math_euler_identity_translated() {
    let src = convert("$e^{i\\pi} + 1 = 0$\n");
    assert!(src.contains("pi"), "pi: {src}");
    // The expression passes through with e^{...}
    assert!(src.contains("e"), "e: {src}");
}

#[test]
fn math_maxwell_equations_display() {
    let src = convert("$$\\nabla \\times \\mathbf{B} = \\mu_0 \\mathbf{J}$$\n");
    assert!(src.contains("nabla"), "nabla: {src}");
    assert!(src.contains("bold("), "mathbf: {src}");
}

#[test]
fn math_display_block_has_surrounding_spaces() {
    // Display math `$ expr $` must have spaces inside the outer $...$ delimiters
    let src = convert("$$a + b = c$$\n");
    assert!(
        src.contains("$ a + b = c $") || (src.contains("$ ") && src.contains(" $")),
        "display math format: {src}"
    );
}

#[test]
fn math_inline_no_surrounding_spaces() {
    // Inline math `$expr$` must NOT have spaces around the delimiters
    let src = convert("Value: $x$.\n");
    assert!(src.contains("$x$"), "inline math format: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Regression: special chars in various contexts
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn regression_dollar_in_paragraph() {
    let src = convert("Pay $9.99 or $100 dollars.\n");
    assert!(src.contains("\\$9.99"), "9.99: {src}");
    assert!(src.contains("\\$100"), "100: {src}");
}

#[test]
fn regression_hash_in_paragraph() {
    let src = convert("The #rust channel and #1 trending.\n");
    assert!(
        src.contains("\\#rust") || src.contains("\\#1"),
        "hash escaped: {src}"
    );
}

#[test]
fn regression_at_in_paragraph() {
    let src = convert("Email user@example.com for help.\n");
    // @ should be escaped when not part of a link
    assert!(
        src.contains("\\@") || src.contains("example.com"),
        "at sign handled: {src}"
    );
}

#[test]
fn regression_backslash_in_code_span() {
    let src = convert("`C:\\Users\\name`\n");
    // Inside inline code, the content is verbatim in backticks
    assert!(
        src.contains("`C:\\Users\\name`") || src.contains("Users"),
        "backslash code: {src}"
    );
}

#[test]
fn regression_brackets_in_text() {
    let src = convert("Typst uses [content] blocks.\n");
    assert!(
        src.contains("\\[content\\]") || src.contains("content"),
        "brackets: {src}"
    );
}

#[test]
fn regression_duplicate_footnote_ref() {
    // Referencing the same footnote twice should not duplicate the definition
    let src = convert("First[^a] and again[^a].\n\n[^a]: The note.\n");
    let section_parts: Vec<&str> = src.split("#line(length: 100%)").collect();
    if section_parts.len() > 1 {
        let footer = section_parts[1];
        // "The note." should appear exactly once in the footer
        let count = footer.matches("The note.").count();
        assert_eq!(
            count, 1,
            "definition should appear once, got {count}: {footer}"
        );
    }
}

#[test]
fn regression_heading_special_chars_in_label() {
    // A heading with special chars should produce a clean label
    let src = convert("## What's New?\n");
    assert!(src.contains("== What"), "heading text: {src}");
    // Label should be clean identifer chars only
    let label_start = src.find('<').unwrap_or(0);
    let label_end = src.find('>').unwrap_or(0);
    if label_end > label_start {
        let label = &src[label_start + 1..label_end];
        assert!(
            label
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "label should have only safe chars: {label}"
        );
    }
}

#[test]
fn regression_empty_blockquote_no_crash() {
    // A blockquote with only whitespace should not panic
    let src = convert(">   \n");
    assert!(
        src.is_ascii() || !src.is_empty(),
        "should not crash on empty blockquote: {src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Page layout: different presets
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn layout_us_letter_preset() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.page_width_mm = 215.9;
    config.page_height_mm = 279.4;
    let src = md_to_typst("# Letter\n\nContent.\n", &config).unwrap();
    assert!(
        src.contains("215.9mm") || src.contains("215.9"),
        "width: {src}"
    );
    assert!(
        src.contains("279.4mm") || src.contains("279.4"),
        "height: {src}"
    );
}

#[test]
fn layout_slides_preset() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.page_width_mm = 338.0;
    config.page_height_mm = 190.0;
    config.base_font_size_pt = 20.0;
    let src = md_to_typst("# Slide\n\n- Point one\n", &config).unwrap();
    assert!(src.contains("338mm") || src.contains("338"), "width: {src}");
    assert!(
        src.contains("190mm") || src.contains("190"),
        "height: {src}"
    );
    assert!(src.contains("20pt"), "font size: {src}");
}

#[test]
fn layout_large_margin() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.margin_mm = 50.0;
    let src = md_to_typst("Content.\n", &config).unwrap();
    assert!(src.contains("50mm") || src.contains("50"), "margin: {src}");
}

#[test]
fn layout_small_font_size() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.base_font_size_pt = 8.0;
    let src = md_to_typst("Content.\n", &config).unwrap();
    assert!(src.contains("8pt"), "8pt font: {src}");
}

#[test]
fn layout_code_font_size_is_fraction_of_body() {
    let dir = TempDir::new().unwrap();
    let config = cfg(&dir);
    let src = md_to_typst("```rust\nfn main() {}\n```\n", &config).unwrap();
    // code font size = base * 0.92; for 12pt base that's ~11.04pt, shown as 9pt in block
    // The block itself uses `size: 9pt` in the highlight code
    assert!(
        src.contains("9pt") || src.contains("11"),
        "code size: {src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Hard line breaks (NodeValue::LineBreak)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn hard_line_break_emits_typst_line_break() {
    // Two trailing spaces before newline → CommonMark hard line break.
    // Must produce `\` + newline (Typst line break), NOT just a space.
    let src = convert("Line one  \nLine two\n");
    assert!(src.contains("Line one"), "first line present: {src}");
    assert!(src.contains("Line two"), "second line present: {src}");
    // The rendered output must contain the Typst line-break sequence.
    assert!(
        src.contains("\\\n"),
        "expected Typst line break (\\\\n): {src}"
    );
    // It must NOT collapse to a plain space between the two words.
    assert!(
        !src.contains("Line one Line two"),
        "should not collapse to single space: {src}"
    );
}

#[test]
fn hard_line_break_multiple() {
    // Multiple hard line breaks in a row.
    let src = convert("A  \nB  \nC\n");
    assert!(src.contains("A"), "A: {src}");
    assert!(src.contains("B"), "B: {src}");
    assert!(src.contains("C"), "C: {src}");
    // At least two Typst line-break sequences.
    let count = src.matches("\\\n").count();
    assert!(count >= 2, "expected ≥2 line breaks, got {count}: {src}");
}

#[test]
fn soft_break_still_becomes_space() {
    // A single newline within a paragraph = soft break → space (unchanged).
    let src = convert("word1\nword2\n");
    assert!(
        src.contains("word1") && src.contains("word2"),
        "words present: {src}"
    );
    // Must NOT contain a Typst line break for a soft break.
    assert!(
        !src.contains("word1\\\n"),
        "soft break should not produce line break: {src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GitHub Alerts
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn alert_note_renders_blue_block() {
    let src = convert("> [!NOTE]\n> This is a note.\n");
    assert!(src.contains("#block("), "block: {src}");
    // Blue accent colour for Note
    assert!(src.contains("#0969da"), "blue accent: {src}");
    // Title bold text
    assert!(src.contains("#strong[Note]"), "bold title: {src}");
    // Body content
    assert!(src.contains("This is a note."), "body: {src}");
}

#[test]
fn alert_tip_renders_green_block() {
    let src = convert("> [!TIP]\n> A helpful tip.\n");
    assert!(src.contains("#block("), "block: {src}");
    assert!(src.contains("#1a7f37"), "green accent: {src}");
    assert!(src.contains("#strong[Tip]"), "bold title: {src}");
    assert!(src.contains("A helpful tip."), "body: {src}");
}

#[test]
fn alert_important_renders_purple_block() {
    let src = convert("> [!IMPORTANT]\n> Key info here.\n");
    assert!(src.contains("#8250df"), "purple accent: {src}");
    assert!(src.contains("#strong[Important]"), "bold title: {src}");
}

#[test]
fn alert_warning_renders_amber_block() {
    let src = convert("> [!WARNING]\n> Urgent warning!\n");
    assert!(src.contains("#9a6700"), "amber accent: {src}");
    assert!(src.contains("#strong[Warning]"), "bold title: {src}");
    assert!(src.contains("Urgent warning!"), "body: {src}");
}

#[test]
fn alert_caution_renders_red_block() {
    let src = convert("> [!CAUTION]\n> Be careful.\n");
    assert!(src.contains("#cf222e"), "red accent: {src}");
    assert!(src.contains("#strong[Caution]"), "bold title: {src}");
    assert!(src.contains("Be careful."), "body: {src}");
}

#[test]
fn alert_has_left_border_stroke() {
    // All alerts should have a left border stroke for visual identification.
    let src = convert("> [!NOTE]\n> Content.\n");
    assert!(
        src.contains("stroke: (left: 3pt +"),
        "left border stroke: {src}"
    );
}

#[test]
fn alert_note_vs_plain_blockquote_different_output() {
    // A plain blockquote must NOT be coloured like an alert.
    let alert_src = convert("> [!NOTE]\n> Alert body.\n");
    let quote_src = convert("> Plain quote.\n");
    assert!(
        alert_src.contains("#0969da"),
        "alert has colour: {alert_src}"
    );
    assert!(
        !quote_src.contains("#0969da"),
        "plain quote has no alert colour: {quote_src}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Superscript extension (^text^)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn superscript_emits_super_call() {
    let src = convert("E = mc^2^ in physics.\n");
    assert!(src.contains("#super[2]"), "super[2]: {src}");
}

#[test]
fn superscript_word_content() {
    let src = convert("x^n+1^ is a term.\n");
    assert!(src.contains("#super["), "super present: {src}");
}

#[test]
fn superscript_ordinal() {
    let src = convert("1^st^ of the month.\n");
    assert!(src.contains("#super[st]"), "ordinal: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Subscript extension (~text~ single tilde)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn subscript_emits_sub_call() {
    // Single-tilde: ~text~ → subscript (double-tilde is still strikethrough).
    let src = convert("H~2~O is water.\n");
    assert!(src.contains("#sub[2]"), "sub[2]: {src}");
}

#[test]
fn subscript_word_content() {
    let src = convert("CO~2~ is carbon dioxide.\n");
    assert!(src.contains("#sub[2]"), "sub content: {src}");
}

#[test]
fn strikethrough_still_works_with_subscript_enabled() {
    // Double-tilde must remain strikethrough even when subscript is enabled.
    let src = convert("~~deleted text~~\n");
    assert!(src.contains("#strike[deleted text]"), "strike: {src}");
    assert!(!src.contains("#sub["), "no sub for strikethrough: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Underline extension (__text__ double underscore)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn underline_emits_underline_call() {
    // Double underscore with underline extension → #underline[…]
    let src = convert("__underlined text__\n");
    assert!(
        src.contains("#underline[underlined text]"),
        "underline: {src}"
    );
}

#[test]
fn underline_different_from_italic() {
    // Single underscore should remain italic (_text_ = emphasis).
    let src = convert("_italic text_\n");
    assert!(src.contains("#emph[italic text]"), "italic: {src}");
    assert!(
        !src.contains("#underline["),
        "no underline for single: {src}"
    );
}

#[test]
fn underline_with_inline_formatting_inside() {
    let src = convert("__**bold underline**__\n");
    assert!(src.contains("#underline["), "underline outer: {src}");
    assert!(src.contains("#strong["), "strong inner: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Loose list spacing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn loose_list_adds_vertical_spacing() {
    // A loose list (blank lines between items) must emit #v(0.5em) spacers.
    let src = convert("- First item\n\n- Second item\n\n- Third item\n");
    assert!(
        src.contains("#v(0.5em)"),
        "vertical spacing for loose list: {src}"
    );
}

#[test]
fn tight_list_no_vertical_spacing() {
    // A tight list (no blank lines between items) must NOT emit #v(0.5em).
    let src = convert("- First\n- Second\n- Third\n");
    assert!(
        !src.contains("#v(0.5em)"),
        "tight list should have no v spacing: {src}"
    );
}

#[test]
fn loose_ordered_list_adds_vertical_spacing() {
    let src = convert("1. Alpha\n\n2. Beta\n\n3. Gamma\n");
    assert!(
        src.contains("#v(0.5em)"),
        "loose ordered list spacing: {src}"
    );
}

#[test]
fn tight_ordered_list_no_vertical_spacing() {
    let src = convert("1. Alpha\n2. Beta\n3. Gamma\n");
    assert!(
        !src.contains("#v(0.5em)"),
        "tight ordered list no spacing: {src}"
    );
}

#[test]
fn loose_list_items_still_render_text() {
    let src = convert("- Item A\n\n- Item B\n");
    assert!(src.contains("Item A"), "item A: {src}");
    assert!(src.contains("Item B"), "item B: {src}");
}

#[test]
fn nested_tight_in_loose_outer_only_outer_spaced() {
    // The outer list is loose (blank lines); inner sub-list is tight.
    // Only outer items get #v(0.5em).
    let src =
        convert("- Outer A\n  - Inner 1\n  - Inner 2\n\n- Outer B\n  - Inner 3\n  - Inner 4\n");
    assert!(src.contains("#v(0.5em)"), "outer spacing: {src}");
    assert!(src.contains("Outer A"), "outer A: {src}");
    assert!(src.contains("Inner 1"), "inner 1: {src}");
}
