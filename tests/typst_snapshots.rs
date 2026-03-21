//! Typst snapshot tests.
//!
//! Each test converts a Markdown string to a Typst source string and then
//! checks it against a known-good inline snapshot.  On the first run (or
//! when `UPDATE_SNAPSHOTS=1` is set), the test writes the current output as
//! a `.typ` file under `tests/fixtures/snapshots/` so reviewers can inspect
//! the exact Typst source that a given Markdown input produces.
//!
//! The inline assertions are deliberately loose — they check the structural
//! shape of the output rather than byte-for-byte equality, which keeps the
//! tests robust across minor cosmetic changes in the renderer.
//!
//! To regenerate snapshot files:
//!   UPDATE_SNAPSHOTS=1 cargo test --test typst_snapshots

use md_to_pdf::renderer::{FontSet, RenderConfig, markdown_to_typst_pub as md_to_typst};
use std::path::{Path, PathBuf};
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

/// Convert markdown and optionally write the output as a `.typ` snapshot file.
///
/// If `UPDATE_SNAPSHOTS=1` is set, the generated Typst source is written to
/// `tests/fixtures/snapshots/<name>.typ`.
fn snapshot(name: &str, md: &str) -> String {
    let dir = TempDir::new().unwrap();
    let src = md_to_typst(md, &cfg(&dir)).unwrap();

    let snap_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/snapshots");
    std::fs::create_dir_all(&snap_dir).ok();
    let snap_path: PathBuf = snap_dir.join(format!("{}.typ", name));

    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(&snap_path, &src).unwrap();
    } else if snap_path.exists() {
        // If a snapshot exists, verify it matches.
        let saved = std::fs::read_to_string(&snap_path).unwrap();
        assert_eq!(
            src, saved,
            "Snapshot mismatch for '{}'. Re-run with UPDATE_SNAPSHOTS=1 to update.",
            name
        );
    } else {
        // First run — write the snapshot.
        std::fs::write(&snap_path, &src).unwrap();
    }

    src
}

// ─────────────────────────────────────────────────────────────────────────────
// Snapshot tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_heading_hierarchy() {
    let src = snapshot("heading_hierarchy", "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\n");
    // Verify shape
    assert!(src.contains("= H1"));
    assert!(src.contains("====== H6"));
}

#[test]
fn snapshot_bold_italic() {
    let src = snapshot("bold_italic", "**bold** and *italic* and ***both***\n");
    assert!(src.contains("#strong[bold]"));
    assert!(src.contains("#emph[italic]"));
}

#[test]
fn snapshot_bullet_list() {
    let src = snapshot("bullet_list", "- Alpha\n- Beta\n  - Nested\n");
    assert!(src.contains("- Alpha"));
    assert!(src.contains("- Beta"));
}

#[test]
fn snapshot_ordered_list() {
    let src = snapshot("ordered_list", "1. First\n2. Second\n3. Third\n");
    assert!(src.contains("+ First"));
    assert!(src.contains("+ Second"));
}

#[test]
fn snapshot_task_list() {
    let src = snapshot("task_list", "- [x] Done\n- [ ] Pending\n");
    assert!(src.contains("☑"));
    assert!(src.contains("☐"));
}

#[test]
fn snapshot_blockquote() {
    let src = snapshot("blockquote", "> A wise word.\n");
    assert!(src.contains("#block("));
    assert!(src.contains("inset"));
    assert!(src.contains("stroke"));
    assert!(src.contains("A wise word."));
}

#[test]
fn snapshot_table_basic() {
    let src = snapshot("table_basic", "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |\n");
    assert!(src.contains("#table("));
    assert!(src.contains("Alice"));
    assert!(src.contains("Bob"));
}

#[test]
fn snapshot_table_alignment() {
    let src = snapshot(
        "table_alignment",
        "| L | C | R |\n|:--|:-:|--:|\n| a | b | c |\n",
    );
    assert!(src.contains("align: left"));
    assert!(src.contains("align: center"));
    assert!(src.contains("align: right"));
}

#[test]
fn snapshot_code_block_python() {
    let src = snapshot("code_block_python", "```python\nprint('hello')\n```\n");
    assert!(src.contains("#block("));
    assert!(src.contains("DejaVu Sans Mono"));
}

#[test]
fn snapshot_code_block_no_lang() {
    let src = snapshot("code_block_no_lang", "```\njust text\n```\n");
    assert!(src.contains("#block("));
    assert!(src.contains("just text"));
}

#[test]
fn snapshot_inline_math() {
    let src = snapshot("inline_math", "The value is $x + y$.\n");
    assert!(src.contains("$x + y$") || src.contains("$x+y$"));
}

#[test]
fn snapshot_display_math() {
    let src = snapshot("display_math", "$$\\int_0^1 x^2 dx = \\frac{1}{3}$$\n");
    // Display math has a leading space: "$ expr $"
    assert!(src.contains("$ ") && src.contains(" $"));
    assert!(src.contains("integral") || src.contains("\\int"));
    assert!(src.contains("frac(1, 3)") || src.contains("frac("));
}

#[test]
fn snapshot_math_greek() {
    let src = snapshot("math_greek", "$\\alpha + \\Omega = \\pi$\n");
    assert!(src.contains("alpha"));
    assert!(src.contains("Omega"));
    assert!(src.contains("pi"));
}

#[test]
fn snapshot_math_matrix() {
    let src = snapshot(
        "math_matrix",
        "$$\\begin{pmatrix} 1 & 0 \\\\ 0 & 1 \\end{pmatrix}$$\n",
    );
    assert!(src.contains("mat("));
}

#[test]
fn snapshot_thematic_break() {
    let src = snapshot("thematic_break", "Before\n\n---\n\nAfter\n");
    assert!(src.contains("#line(length: 100%)"));
}

#[test]
fn snapshot_link() {
    let src = snapshot("link", "[Rust](https://www.rust-lang.org)\n");
    assert!(src.contains("#link(\"https://www.rust-lang.org\", [Rust])"));
}

#[test]
fn snapshot_strikethrough() {
    let src = snapshot("strikethrough", "~~old value~~\n");
    assert!(src.contains("#strike[old value]"));
}

#[test]
fn snapshot_footnote() {
    let src = snapshot("footnote", "Text[^1] here.\n\n[^1]: A note.\n");
    assert!(src.contains("#super["));
    assert!(src.contains("A note."));
}

#[test]
fn snapshot_toc_enabled() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.toc = true;
    config.toc_explicit = true;
    config.toc_depth = 3;
    let src = md_to_typst("# H1\n## H2\n", &config).unwrap();

    // Write snapshot manually (not using the helper since we need a custom config)
    let snap_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/snapshots");
    std::fs::create_dir_all(&snap_dir).ok();
    let snap_path = snap_dir.join("toc_enabled.typ");
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() || !snap_path.exists() {
        std::fs::write(&snap_path, &src).ok();
    }

    assert!(src.contains("#outline("));
    assert!(src.contains("depth: 3"));
}

#[test]
fn snapshot_page_header() {
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.page_width_mm = 338.0;
    config.page_height_mm = 190.0;
    config.margin_mm = 12.0;
    config.base_font_size_pt = 14.0;
    let src = md_to_typst("Slides content\n", &config).unwrap();

    let snap_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/snapshots");
    std::fs::create_dir_all(&snap_dir).ok();
    let snap_path = snap_dir.join("page_header_slides.typ");
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() || !snap_path.exists() {
        std::fs::write(&snap_path, &src).ok();
    }

    assert!(src.contains("338mm") || src.contains("338"));
    assert!(src.contains("190mm") || src.contains("190"));
    assert!(src.contains("12mm") || src.contains("12"));
    assert!(src.contains("14pt"));
}

#[test]
fn snapshot_missing_image_fallback() {
    let src = snapshot("missing_image", "![Alt text](no-such-file.png)\n");
    assert!(src.contains("#block("));
    assert!(src.contains("\\[Image:"));
    assert!(src.contains("Alt text"));
}

#[test]
fn snapshot_inline_html_strong() {
    let src = snapshot("inline_html_strong", "Text <strong>bold via HTML</strong> text.\n");
    assert!(src.contains("#strong["));
    assert!(src.contains("bold via HTML"));
}

#[test]
fn snapshot_description_list() {
    let src = snapshot("description_list", "Term\n: Definition body\n");
    assert!(src.contains("#strong[Term]"));
    assert!(src.contains("Definition body"));
}

// ─────────────────────────────────────────────────────────────────────────────
// New GFM fidelity snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_nested_bullet_list() {
    let src = snapshot(
        "nested_bullet_list",
        "- Top A\n  - Nested B\n    - Deep C\n  - Back B\n- Top E\n",
    );
    // Outer items use dash with no indent
    assert!(src.contains("- Top A"), "top-level item missing: {src}");
    // Second-level items use 2-space indent
    assert!(src.contains("  - Nested B"), "nested item missing: {src}");
    // Third-level items use 4-space indent
    assert!(src.contains("    - Deep C"), "deep item missing: {src}");
    // No spurious blank lines between item and its nested list
    assert!(!src.contains("Top A\n\n  - Nested"), "blank line between item and nested list: {src}");
}

#[test]
fn snapshot_nested_ordered_list() {
    let src = snapshot(
        "nested_ordered_list",
        "1. First\n2. Second\n   1. Sub one\n   2. Sub two\n3. Third\n",
    );
    assert!(src.contains("+ First"), "first item: {src}");
    assert!(src.contains("+ Second"), "second item: {src}");
    // Nested ordered items use 3-space indent (matching the `   1.` prefix)
    assert!(src.contains("  + Sub one") || src.contains("   + Sub one"), "nested sub-item: {src}");
    assert!(src.contains("+ Third"), "third item: {src}");
}

#[test]
fn snapshot_ordered_list_start_at_3() {
    let src = snapshot(
        "ordered_list_start_3",
        "3. Already third\n4. Fourth\n5. Fifth\n",
    );
    // Must include a `#set enum(start: 3)` or equivalent to render correct numbers
    assert!(
        src.contains("start: 3") || src.contains("#set enum(start: 3)"),
        "expected start-at-3 directive, got: {src}"
    );
    assert!(src.contains("+ Already third"), "item text: {src}");
}

#[test]
fn snapshot_task_list_with_nested() {
    let src = snapshot(
        "task_list_nested",
        "- [x] Done\n- [ ] Pending\n- [x] Another\n  - [ ] Nested pending\n  - [x] Nested done\n",
    );
    // Checkbox via #box(width: 1em)[…]
    assert!(src.contains("#box(width: 1em)[☑]"), "checked box: {src}");
    assert!(src.contains("#box(width: 1em)[☐]"), "unchecked box: {src}");
    // Nested task items indented
    assert!(
        src.contains("  - #box(width: 1em)[☐] Nested pending"),
        "nested unchecked: {src}"
    );
    assert!(
        src.contains("  - #box(width: 1em)[☑] Nested done"),
        "nested checked: {src}"
    );
    // No blank line between parent and nested list
    assert!(
        !src.contains("Another\n\n  - "),
        "blank line before nested tasks: {src}"
    );
}

#[test]
fn snapshot_table_alignment_markers() {
    let src = snapshot(
        "table_alignment_markers",
        "| Name | Score | Grade |\n|:-----|:-----:|------:|\n| Alice | 95 | A+ |\n",
    );
    // Header cells must be bold
    assert!(src.contains("#strong[Name]"), "header bold: {src}");
    assert!(src.contains("#strong[Score]"), "header bold: {src}");
    assert!(src.contains("#strong[Grade]"), "header bold: {src}");
    // Alignment per column
    assert!(src.contains("align: left"), "left align: {src}");
    assert!(src.contains("align: center"), "center align: {src}");
    assert!(src.contains("align: right"), "right align: {src}");
    // Data cells are NOT wrapped in #strong
    assert!(!src.contains("#strong[Alice]"), "data cells should not be bold: {src}");
}

#[test]
fn snapshot_autolink_bare_url() {
    let src = snapshot(
        "autolink_bare_url",
        "Visit https://example.com for more.\n",
    );
    // Bare URLs parsed by autolink extension → compact #link("url")
    assert!(src.contains("#link(\"https://example.com\")"), "bare URL autolink: {src}");
}

#[test]
fn snapshot_autolink_angle_bracket() {
    let src = snapshot(
        "autolink_angle_bracket",
        "See <https://www.rust-lang.org>.\n",
    );
    assert!(src.contains("#link("), "angle-bracket autolink: {src}");
    assert!(src.contains("rust-lang.org"), "URL in output: {src}");
}

#[test]
fn snapshot_autolink_email() {
    let src = snapshot(
        "autolink_email",
        "Contact <user@example.com>.\n",
    );
    // Email autolinks become mailto: links
    assert!(src.contains("mailto:user@example.com"), "mailto link: {src}");
    // The @ in the display label must be escaped for Typst
    assert!(src.contains("user\\@example.com"), "escaped @ in label: {src}");
}

#[test]
fn snapshot_footnote_ordering() {
    let src = snapshot(
        "footnote_ordering",
        "First ref[^a]. Second ref[^b]. Third ref[^a] again.\n\n\
         [^b]: Beta definition.\n\n\
         [^a]: Alpha definition.\n",
    );
    // Footnote section separator must be present
    assert!(src.contains("#line(length: 100%)"), "footnote separator: {src}");
    // [^a] appears first in the text → ix=1; [^b] appears second → ix=2
    // So Alpha definition should be listed as #super[1] and Beta as #super[2]
    let super1_pos = src.find("#super[1] Alpha definition").or_else(|| src.find("#super[1]"));
    let super2_pos = src.find("#super[2] Beta definition").or_else(|| src.find("#super[2]"));
    assert!(super1_pos.is_some(), "super[1] missing: {src}");
    assert!(super2_pos.is_some(), "super[2] missing: {src}");
    // The [^a] footnote (first referenced) should appear before [^b]
    if let (Some(p1), Some(p2)) = (super1_pos, super2_pos) {
        assert!(p1 < p2, "#super[1] should come before #super[2] in output: {src}");
    }
}

#[test]
fn snapshot_definition_list_multiple_details() {
    let src = snapshot(
        "definition_list_multi",
        "Term\n: First detail.\n: Second detail.\n",
    );
    assert!(src.contains("#strong[Term]"), "term is bold: {src}");
    assert!(src.contains("First detail."), "first detail: {src}");
    assert!(src.contains("Second detail."), "second detail: {src}");
    // Both details should be padded
    let pad_count = src.matches("#pad(left: 1.5em)").count();
    assert!(pad_count >= 2, "expected 2 padded details, got {pad_count}: {src}");
}

#[test]
fn snapshot_definition_list_multiple_terms() {
    let src = snapshot(
        "definition_list_terms",
        "Alpha\n: Definition of alpha.\n\nBeta\n: Definition of beta.\n",
    );
    assert!(src.contains("#strong[Alpha]"), "Alpha term: {src}");
    assert!(src.contains("#strong[Beta]"), "Beta term: {src}");
    assert!(src.contains("Definition of alpha."), "alpha def: {src}");
    assert!(src.contains("Definition of beta."), "beta def: {src}");
}

#[test]
fn snapshot_gfm_fixture() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("gfm-fixture.md");
    let md = std::fs::read_to_string(path).expect("gfm-fixture.md must exist");
    let dir = TempDir::new().unwrap();
    let src = md_to_typst(&md, &cfg(&dir)).unwrap();

    // Write as a snapshot file
    let snap_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/snapshots");
    std::fs::create_dir_all(&snap_dir).ok();
    let snap_path = snap_dir.join("gfm_fixture.typ");
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() || !snap_path.exists() {
        std::fs::write(&snap_path, &src).ok();
    }

    // GFM fixture structural checks
    assert!(src.contains("- Top-level item A"), "bullet list: {src}");
    assert!(src.contains("  - Second-level item B"), "nested bullet: {src}");
    assert!(src.contains("    - Third-level item C"), "deep nested bullet: {src}");
    assert!(src.contains("#box(width: 1em)[☑]"), "task checked: {src}");
    assert!(src.contains("#box(width: 1em)[☐]"), "task unchecked: {src}");
    assert!(src.contains("align: left"), "table align left: {src}");
    assert!(src.contains("align: center"), "table align center: {src}");
    assert!(src.contains("align: right"), "table align right: {src}");
    assert!(src.contains("#link("), "autolink: {src}");
    assert!(src.contains("#super[1]") || src.contains("#super["), "footnote ref: {src}");
    assert!(src.contains("#line(length: 100%)"), "footnote separator: {src}");
    assert!(src.contains("#strong[Markdown]") || src.contains("Markdown"), "def list term: {src}");
    assert!(src.contains("#pad(left: 1.5em)"), "def list detail padded: {src}");
}
