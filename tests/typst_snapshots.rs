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

// ─────────────────────────────────────────────────────────────────────────────
// Additional snapshots for math environments
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_math_aligned_env() {
    let src = snapshot(
        "math_aligned",
        "$$\\begin{align} a &= b + c \\\\ d &= e + f \\end{align}$$\n",
    );
    // align environment produces at least something non-empty
    assert!(src.contains("$ ") && src.contains(" $"), "display math: {src}");
}

#[test]
fn snapshot_math_vmatrix() {
    let src = snapshot(
        "math_vmatrix",
        "$$\\begin{vmatrix} a & b \\\\ c & d \\end{vmatrix}$$\n",
    );
    assert!(src.contains("mat("), "mat call: {src}");
}

#[test]
fn snapshot_math_greek_full() {
    let src = snapshot(
        "math_greek_full",
        "$\\alpha \\beta \\gamma \\delta \\epsilon \\zeta \\eta \\theta$\n",
    );
    assert!(src.contains("alpha"), "alpha: {src}");
    assert!(src.contains("beta"), "beta: {src}");
    assert!(src.contains("gamma"), "gamma: {src}");
    assert!(src.contains("delta"), "delta: {src}");
}

#[test]
fn snapshot_math_operators() {
    let src = snapshot(
        "math_operators",
        "$a \\leq b \\geq c \\neq d \\approx e$\n",
    );
    assert!(src.contains("lt.eq"), "leq: {src}");
    assert!(src.contains("gt.eq"), "geq: {src}");
    assert!(src.contains("eq.not"), "neq: {src}");
    assert!(src.contains("approx"), "approx: {src}");
}

#[test]
fn snapshot_math_blackboard_bold() {
    let src = snapshot(
        "math_blackboard_bold",
        "$\\mathbb{R} \\cup \\mathbb{Z} \\subset \\mathbb{C}$\n",
    );
    assert!(src.contains("RR"), "RR: {src}");
    assert!(src.contains("ZZ"), "ZZ: {src}");
    assert!(src.contains("CC"), "CC: {src}");
}

#[test]
fn snapshot_math_sum_limits() {
    let src = snapshot(
        "math_sum_limits",
        "$$\\sum_{k=0}^{n} \\binom{n}{k} = 2^n$$\n",
    );
    assert!(src.contains("sum"), "sum: {src}");
    assert!(src.contains("binom("), "binom: {src}");
}

#[test]
fn snapshot_math_integral_limits() {
    let src = snapshot(
        "math_integral_limits",
        "$$\\int_0^{\\infty} e^{-x} \\, dx = 1$$\n",
    );
    assert!(src.contains("integral"), "integral: {src}");
    assert!(src.contains("oo"), "infinity: {src}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional snapshots for GFM elements
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_mixed_list_types() {
    let src = snapshot(
        "mixed_list_types",
        "- Category A\n  1. Sub one\n  2. Sub two\n- Category B\n  1. Sub one\n",
    );
    assert!(src.contains("- Category A"), "bullet A: {src}");
    assert!(src.contains("- Category B"), "bullet B: {src}");
    assert!(src.contains("+ Sub one") || src.contains("Sub one"), "sub item: {src}");
}

#[test]
fn snapshot_ordered_list_start_5() {
    let src = snapshot(
        "ordered_list_start_5",
        "5. Fifth\n6. Sixth\n7. Seventh\n",
    );
    assert!(
        src.contains("start: 5") || src.contains("#set enum(start: 5)"),
        "start:5 directive: {src}"
    );
    assert!(src.contains("+ Fifth"), "item: {src}");
}

#[test]
fn snapshot_table_wide() {
    let src = snapshot(
        "table_wide",
        "| ID | Name | Email | Role | Active |\n\
         |----|------|-------|------|--------|\n\
         | 1 | Alice | alice@example.com | Admin | Yes |\n\
         | 2 | Bob | bob@example.com | User | No |\n",
    );
    assert!(src.contains("#table("), "table: {src}");
    let count = src.matches("1fr").count();
    assert_eq!(count, 5, "five columns: {src}");
    assert!(src.contains("Alice"), "data: {src}");
}

#[test]
fn snapshot_table_formatted_cells() {
    let src = snapshot(
        "table_formatted_cells",
        "| Feature | Status |\n|---------|--------|\n\
         | **Bold** | ✅ |\n| *Italic* | ✅ |\n| `Code` | ✅ |\n",
    );
    assert!(src.contains("#table("), "table: {src}");
    // Cell content with formatting
    assert!(src.contains("Bold") || src.contains("#strong["), "bold in cell: {src}");
}

#[test]
fn snapshot_code_block_typescript() {
    let src = snapshot(
        "code_block_typescript",
        "```typescript\ninterface User { id: number; name: string; }\n\
         async function get(id: number): Promise<User> {\n    return fetch(`/users/${id}`);\n}\n```\n",
    );
    assert!(src.contains("#block("), "block: {src}");
    assert!(src.contains("DejaVu Sans Mono"), "mono font: {src}");
}

#[test]
fn snapshot_code_block_sql() {
    let src = snapshot(
        "code_block_sql",
        "```sql\nSELECT name, COUNT(*) FROM users GROUP BY name ORDER BY 2 DESC;\n```\n",
    );
    assert!(src.contains("#block("), "block: {src}");
    assert!(src.contains("DejaVu Sans Mono"), "mono font: {src}");
}

#[test]
fn snapshot_code_block_bash() {
    let src = snapshot(
        "code_block_bash",
        "```bash\n#!/usr/bin/env bash\nset -euo pipefail\necho 'Hello, world!'\n```\n",
    );
    assert!(src.contains("#block("), "block: {src}");
}

#[test]
fn snapshot_code_block_javascript() {
    let src = snapshot(
        "code_block_javascript",
        "```javascript\nconst greet = (name) => `Hello, ${name}!`;\nconsole.log(greet('World'));\n```\n",
    );
    assert!(src.contains("#block("), "block: {src}");
}

#[test]
fn snapshot_nested_task_list_deep() {
    let src = snapshot(
        "nested_task_list_deep",
        "- [x] Root done\n  - [ ] Level 2 pending\n    - [x] Level 3 done\n",
    );
    assert!(src.contains("☑"), "checked: {src}");
    assert!(src.contains("☐"), "unchecked: {src}");
    // Three checkbox items at varying indentation levels
    let box_count = src.matches("#box(width: 1em)").count();
    assert!(box_count >= 3, "at least 3 checkbox items: {src}");
}

#[test]
fn snapshot_multiple_footnotes_ordered() {
    let src = snapshot(
        "multiple_footnotes_ordered",
        "Alpha[^a]. Beta[^b]. Gamma[^c].\n\n\
         [^a]: A definition.\n\n\
         [^b]: B definition.\n\n\
         [^c]: C definition.\n",
    );
    // All three footnotes must appear with correct numbering
    assert!(src.contains("#super[1]"), "super 1: {src}");
    assert!(src.contains("#super[2]"), "super 2: {src}");
    assert!(src.contains("#super[3]"), "super 3: {src}");
    // Footer section present
    assert!(src.contains("#line(length: 100%)"), "separator: {src}");
    assert!(src.contains("A definition."), "A def: {src}");
    assert!(src.contains("B definition."), "B def: {src}");
    assert!(src.contains("C definition."), "C def: {src}");
}

#[test]
fn snapshot_definition_list_complex() {
    let src = snapshot(
        "definition_list_complex",
        "Term A\n: First detail for A.\n: Second detail for A.\n\n\
         Term B\n: Only detail for B.\n",
    );
    assert!(src.contains("#strong[Term A]"), "term A: {src}");
    assert!(src.contains("#strong[Term B]"), "term B: {src}");
    assert!(src.contains("First detail for A."), "first detail: {src}");
    assert!(src.contains("Second detail for A."), "second detail: {src}");
    assert!(src.contains("Only detail for B."), "B detail: {src}");
    let pad_count = src.matches("#pad(left: 1.5em)").count();
    assert!(pad_count >= 3, "three padded details: {src}");
}

#[test]
fn snapshot_blockquote_with_formatting() {
    let src = snapshot(
        "blockquote_with_formatting",
        "> **Bold** text and *italic* and `code` inside a blockquote.\n",
    );
    assert!(src.contains("#block("), "block: {src}");
    assert!(src.contains("#strong["), "strong: {src}");
    assert!(src.contains("#emph["), "emph: {src}");
}

#[test]
fn snapshot_inline_math_in_table() {
    let src = snapshot(
        "inline_math_in_table",
        "| Expression | Description |\n|------------|-------------|\n\
         | $x^2$ | quadratic |\n| $\\sqrt{x}$ | square root |\n",
    );
    assert!(src.contains("#table("), "table: {src}");
    // Math expressions should appear in cell content
    assert!(src.contains("$x^2$") || src.contains("x^2"), "x^2: {src}");
    assert!(src.contains("sqrt(") || src.contains("\\sqrt"), "sqrt: {src}");
}

#[test]
fn snapshot_comprehensive_gfm() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/samples/comprehensive_gfm.md");
    let md = std::fs::read_to_string(path).expect("comprehensive_gfm.md must exist");
    let dir = TempDir::new().unwrap();
    let mut config = cfg(&dir);
    config.toc = false; // let frontmatter control
    config.toc_explicit = false;
    let src = md_to_typst(&md, &config).unwrap();

    let snap_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/snapshots");
    std::fs::create_dir_all(&snap_dir).ok();
    let snap_path = snap_dir.join("comprehensive_gfm.typ");
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() || !snap_path.exists() {
        std::fs::write(&snap_path, &src).ok();
    }

    // All major GFM features should be present
    assert!(src.contains("#outline("), "TOC from frontmatter: {src}");
    assert!(src.contains("depth: 3"), "TOC depth: {src}");
    assert!(src.contains("Contents"), "TOC title: {src}");
    assert!(src.contains("#table("), "table: {src}");
    assert!(src.contains("#box(width: 1em)"), "task list: {src}");
    assert!(src.contains("#super["), "footnote: {src}");
    assert!(src.contains("#strong["), "def list or bold: {src}");
}

#[test]
fn snapshot_regression_dollar_signs() {
    let src = snapshot(
        "regression_dollar_signs",
        "Price: $9.99 and $100 are both dollar amounts.\n\nMath: $x^2 + y^2 = r^2$\n",
    );
    // Dollar signs in plain text must be escaped
    assert!(src.contains("\\$9.99"), "dollar escape: {src}");
    assert!(src.contains("\\$100"), "dollar escape: {src}");
    // But math dollars should NOT be escaped (they're Typst math)
    assert!(src.contains("$x^2"), "math not escaped: {src}");
}

#[test]
fn snapshot_regression_at_sign() {
    let src = snapshot(
        "regression_at_sign",
        "Email user@example.com or admin@test.org for help.\n",
    );
    // @ in plain text must be escaped
    assert!(src.contains("\\@"), "at sign escaped: {src}");
}
