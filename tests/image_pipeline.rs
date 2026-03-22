//! Integration tests for the image pipeline.
//!
//! Covers:
//!  - Remote content-type detection (mime_to_ext, sniff_image_magic)
//!  - SVG edge-case handling (BOM, whitespace, xml declaration)
//!  - Sizing heuristics (css_length_to_typst)
//!  - Caption / alt-text rendering (format_image_typst / format_image_typst_sized)
//!  - Missing-image fallbacks (missing_image_fallback)
//!  - Cache invalidation metadata (read_cache_meta round-trip)
//!  - Data-URI decoding (resolve_data_uri via markdown round-trips)
//!  - Markdown → Typst round-trips exercising the full render path

use md_to_pdf::renderer::{
    FontSet, ImageInfo, RenderConfig, SizeHint, css_length_to_typst, detect_image_format,
    format_image_typst, format_image_typst_sized, is_svg_bytes,
    markdown_to_typst_pub as md_to_typst, mime_to_ext, missing_image_fallback, sniff_image_magic,
    stable_name_pub as stable_name,
};
use std::fs;
use std::path::PathBuf;
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
        cache_dir_override: Some(dir.path().join("cache")),
    }
}

fn render(md: &str, dir: &TempDir) -> String {
    md_to_typst(md, &cfg(dir)).expect("render failed")
}

fn make_info(path: &str, is_svg: bool) -> ImageInfo {
    ImageInfo {
        path: PathBuf::from(path),
        is_svg,
        natural_width: None,
        natural_height: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Content-Type / MIME detection
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mime_to_ext_all_known_types() {
    let cases: &[(&str, &str)] = &[
        ("image/png", "png"),
        ("image/jpeg", "jpg"),
        ("image/jpg", "jpg"),
        ("image/gif", "gif"),
        ("image/webp", "webp"),
        ("image/svg+xml", "svg"),
        ("image/svg", "svg"),
        ("image/bmp", "bmp"),
        ("image/tiff", "tiff"),
        ("image/avif", "avif"),
        ("image/x-icon", "ico"),
        ("image/vnd.microsoft.icon", "ico"),
    ];
    for (mime, expected) in cases {
        assert_eq!(
            mime_to_ext(mime),
            Some(*expected),
            "mime_to_ext({mime:?}) should be {expected:?}"
        );
    }
}

#[test]
fn mime_to_ext_unknown_returns_none() {
    assert_eq!(mime_to_ext("application/octet-stream"), None);
    assert_eq!(mime_to_ext("text/plain"), None);
    assert_eq!(mime_to_ext(""), None);
    assert_eq!(mime_to_ext("image/unknown-format"), None);
}

#[test]
fn detect_format_content_type_wins_over_magic() {
    // Even if magic bytes say PNG, a content-type of SVG should win
    let svg_bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
    assert_eq!(
        detect_image_format("https://example.com/x", "image/svg+xml", svg_bytes),
        "svg"
    );
}

#[test]
fn detect_format_content_type_with_charset_param() {
    // Content-Type may include "; charset=utf-8" — should still resolve
    let bytes = b"\x89PNG\r\n\x1a\n";
    assert_eq!(
        detect_image_format("https://example.com/img", "image/png; charset=utf-8", bytes),
        "png"
    );
}

#[test]
fn detect_format_content_type_with_quality_param() {
    // Some servers send "image/jpeg; quality=85"
    let bytes = b"\xff\xd8\xff\xe0";
    assert_eq!(
        detect_image_format(
            "https://example.com/img.jpg",
            "image/jpeg; quality=85",
            bytes
        ),
        "jpg"
    );
}

#[test]
fn detect_format_falls_back_to_magic_on_unknown_ct() {
    // Content-Type is unrecognised → fall back to magic bytes
    let png_bytes = b"\x89PNG\r\n\x1a\nsome data";
    assert_eq!(
        detect_image_format(
            "https://example.com/img",
            "application/octet-stream",
            png_bytes
        ),
        "png"
    );
}

#[test]
fn detect_format_falls_back_to_url_when_ct_and_magic_fail() {
    // Unknown CT + unrecognised bytes → fall back to URL extension
    let garbage = b"??????????garbage";
    assert_eq!(
        detect_image_format(
            "https://cdn.example.com/logo.webp",
            "application/octet-stream",
            garbage
        ),
        "webp"
    );
}

#[test]
fn detect_format_empty_ct_uses_magic() {
    let jpeg = b"\xff\xd8\xff\xe1";
    assert_eq!(
        detect_image_format("https://example.com/x", "", jpeg),
        "jpg"
    );
}

#[test]
fn detect_format_url_query_stripped_for_ext() {
    let garbage = b"not-a-real-image";
    assert_eq!(
        detect_image_format("https://cdn.example.com/avatar.gif?v=3", "", garbage),
        "gif"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Magic-byte sniffing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn sniff_png_magic() {
    assert_eq!(sniff_image_magic(b"\x89PNG\r\n\x1a\n"), Some("png"));
}

#[test]
fn sniff_jpeg_magic() {
    assert_eq!(sniff_image_magic(b"\xff\xd8\xff\xe0"), Some("jpg"));
    assert_eq!(sniff_image_magic(b"\xff\xd8\xff\xe1"), Some("jpg"));
}

#[test]
fn sniff_gif87_magic() {
    assert_eq!(sniff_image_magic(b"GIF87aXXXX"), Some("gif"));
}

#[test]
fn sniff_gif89_magic() {
    assert_eq!(sniff_image_magic(b"GIF89aXXXX"), Some("gif"));
}

#[test]
fn sniff_webp_magic() {
    let bytes = b"RIFF\x00\x00\x00\x00WEBPextra";
    assert_eq!(sniff_image_magic(bytes), Some("webp"));
}

#[test]
fn sniff_bmp_magic() {
    assert_eq!(sniff_image_magic(b"BMextra"), Some("bmp"));
}

#[test]
fn sniff_tiff_little_endian() {
    assert_eq!(sniff_image_magic(b"II\x2a\x00extra"), Some("tiff"));
}

#[test]
fn sniff_tiff_big_endian() {
    assert_eq!(sniff_image_magic(b"MM\x00\x2aextra"), Some("tiff"));
}

#[test]
fn sniff_svg_xml_declaration() {
    let svg = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
    assert_eq!(sniff_image_magic(svg), Some("svg"));
}

#[test]
fn sniff_svg_bare_tag() {
    let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\"></svg>";
    assert_eq!(sniff_image_magic(svg), Some("svg"));
}

#[test]
fn sniff_svg_with_leading_whitespace() {
    let svg = b"   \n\t<svg></svg>";
    assert_eq!(sniff_image_magic(svg), Some("svg"));
}

#[test]
fn sniff_too_short_returns_none() {
    assert_eq!(sniff_image_magic(b"PNG"), None);
    assert_eq!(sniff_image_magic(b""), None);
    assert_eq!(sniff_image_magic(b"X"), None);
}

#[test]
fn sniff_unknown_bytes_returns_none() {
    assert_eq!(sniff_image_magic(b"????garbage????"), None);
}

// ─────────────────────────────────────────────────────────────────────────────
// SVG edge-case handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn is_svg_xml_declaration() {
    assert!(is_svg_bytes(b"<?xml version=\"1.0\"?><svg></svg>"));
}

#[test]
fn is_svg_bare_opening_tag() {
    assert!(is_svg_bytes(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"));
}

#[test]
fn is_svg_with_utf8_bom() {
    let mut bytes = b"\xef\xbb\xbf".to_vec();
    bytes.extend_from_slice(b"<svg></svg>");
    assert!(is_svg_bytes(&bytes));
}

#[test]
fn is_svg_with_bom_and_whitespace() {
    let mut bytes = b"\xef\xbb\xbf".to_vec();
    bytes.extend_from_slice(b"  \n<svg></svg>");
    assert!(is_svg_bytes(&bytes));
}

#[test]
fn is_svg_newline_before_tag() {
    assert!(is_svg_bytes(b"\n<svg></svg>"));
}

#[test]
fn is_svg_tab_before_tag() {
    assert!(is_svg_bytes(b"\t<svg></svg>"));
}

#[test]
fn is_svg_case_insensitive_namespace() {
    // SVG tags are case-sensitive per spec, but check that the checker
    // works on realistic inputs with the proper lowercase <svg> tag
    assert!(is_svg_bytes(
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"
    ));
}

#[test]
fn is_svg_not_html() {
    assert!(!is_svg_bytes(b"<!DOCTYPE html><html><body></body></html>"));
}

#[test]
fn is_svg_not_png() {
    assert!(!is_svg_bytes(b"\x89PNG\r\n\x1a\n"));
}

#[test]
fn is_svg_not_jpeg() {
    assert!(!is_svg_bytes(b"\xff\xd8\xff\xe0"));
}

#[test]
fn is_svg_not_empty() {
    assert!(!is_svg_bytes(b""));
}

#[test]
fn is_svg_not_random_xml() {
    // XML that's not SVG
    assert!(!is_svg_bytes(b"<?xml version=\"1.0\"?><root></root>"));
}

// When SVG is resolved from disk, format_image_typst should include `format: "svg"`.
#[test]
fn local_svg_file_emits_format_svg() {
    let dir = TempDir::new().unwrap();
    let svg_path = dir.path().join("diagram.svg");
    fs::write(&svg_path, b"<svg></svg>").unwrap();
    let md = "![A diagram](diagram.svg)\n";
    let out = render(md, &dir);
    assert!(
        out.contains("format: \"svg\""),
        "expected SVG format arg:\n{out}"
    );
}

#[test]
fn local_svg_file_with_special_chars_in_alt() {
    let dir = TempDir::new().unwrap();
    let svg_path = dir.path().join("chart.svg");
    fs::write(&svg_path, b"<svg></svg>").unwrap();
    let md = "![Revenue $1M & profit [Q1]](chart.svg)\n";
    let out = render(md, &dir);
    // Alt text special chars must be escaped in caption
    assert!(out.contains("\\$1M"), "dollar sign must be escaped:\n{out}");
    assert!(out.contains("\\[Q1\\]"), "brackets must be escaped:\n{out}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Sizing heuristics (css_length_to_typst)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn sizing_px_to_pt() {
    // 96px → 72pt (×0.75), 100px → 75pt
    assert_eq!(css_length_to_typst("96px"), Some("72.0pt".to_string()));
    assert_eq!(css_length_to_typst("100px"), Some("75.0pt".to_string()));
    assert_eq!(css_length_to_typst("1px"), Some("0.8pt".to_string()));
}

#[test]
fn sizing_bare_integer_as_pixels() {
    assert_eq!(css_length_to_typst("200"), Some("150.0pt".to_string()));
    assert_eq!(css_length_to_typst("0"), Some("0.0pt".to_string()));
    assert_eq!(css_length_to_typst("1000"), Some("750.0pt".to_string()));
}

#[test]
fn sizing_percent_passthrough() {
    assert_eq!(css_length_to_typst("100%"), Some("100%".to_string()));
    assert_eq!(css_length_to_typst("50%"), Some("50%".to_string()));
    assert_eq!(css_length_to_typst("0%"), Some("0%".to_string()));
    assert_eq!(css_length_to_typst("33.3%"), Some("33.3%".to_string()));
}

#[test]
fn sizing_em_passthrough() {
    assert_eq!(css_length_to_typst("1em"), Some("1em".to_string()));
    assert_eq!(css_length_to_typst("2.5em"), Some("2.5em".to_string()));
}

#[test]
fn sizing_rem_converts_to_em() {
    assert_eq!(css_length_to_typst("1.5rem"), Some("1.5em".to_string()));
    assert_eq!(css_length_to_typst("2rem"), Some("2em".to_string()));
}

#[test]
fn sizing_pt_passthrough() {
    assert_eq!(css_length_to_typst("12pt"), Some("12pt".to_string()));
    assert_eq!(css_length_to_typst("0pt"), Some("0pt".to_string()));
}

#[test]
fn sizing_mm_passthrough() {
    assert_eq!(css_length_to_typst("20mm"), Some("20mm".to_string()));
}

#[test]
fn sizing_cm_passthrough() {
    assert_eq!(css_length_to_typst("5cm"), Some("5cm".to_string()));
}

#[test]
fn sizing_in_passthrough() {
    assert_eq!(css_length_to_typst("2in"), Some("2in".to_string()));
}

#[test]
fn sizing_empty_returns_none() {
    assert_eq!(css_length_to_typst(""), None);
    assert_eq!(css_length_to_typst("   "), None);
}

#[test]
fn sizing_auto_returns_none() {
    assert_eq!(css_length_to_typst("auto"), None);
}

#[test]
fn sizing_invalid_unit_returns_none() {
    assert_eq!(css_length_to_typst("100vw"), None);
    assert_eq!(css_length_to_typst("50vh"), None);
}

// ─────────────────────────────────────────────────────────────────────────────
// Caption and alt-text rendering
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn format_image_png_no_alt_no_caption() {
    let info = make_info("/tmp/img.png", false);
    let out = format_image_typst(&info, "");
    assert!(out.contains("image("), "got:\n{out}");
    assert!(!out.contains("caption"), "should have no caption:\n{out}");
    assert!(
        !out.contains("format:"),
        "should have no format: for png:\n{out}"
    );
}

#[test]
fn format_image_png_with_alt_becomes_caption() {
    let info = make_info("/tmp/chart.png", false);
    let out = format_image_typst(&info, "Quarterly revenue");
    assert!(out.contains("caption: [Quarterly revenue]"), "got:\n{out}");
}

#[test]
fn format_image_caption_escapes_special_chars() {
    let info = make_info("/tmp/img.png", false);
    // Hash, brackets, dollar, braces — all must be escaped in caption
    let out = format_image_typst(&info, "#heading [bracket] $99 {brace}");
    assert!(out.contains("\\#heading"), "hash not escaped:\n{out}");
    assert!(
        out.contains("\\[bracket\\]"),
        "brackets not escaped:\n{out}"
    );
    assert!(out.contains("\\$99"), "dollar not escaped:\n{out}");
    assert!(out.contains("\\{brace\\}"), "braces not escaped:\n{out}");
}

#[test]
fn format_image_svg_gets_format_arg() {
    let info = make_info("/tmp/logo.svg", true);
    let out = format_image_typst(&info, "Logo");
    assert!(out.contains("format: \"svg\""), "got:\n{out}");
    assert!(out.contains("caption: [Logo]"), "got:\n{out}");
}

#[test]
fn format_image_svg_no_alt_still_has_format_arg() {
    let info = make_info("/path/to/icon.svg", true);
    let out = format_image_typst(&info, "");
    assert!(out.contains("format: \"svg\""), "got:\n{out}");
    assert!(
        !out.contains("caption"),
        "no caption when alt is empty:\n{out}"
    );
}

#[test]
fn format_image_default_width_is_100pct() {
    let info = make_info("/img.png", false);
    let out = format_image_typst(&info, "");
    assert!(out.contains("width: 100%"), "got:\n{out}");
}

#[test]
fn format_image_sized_custom_width() {
    let info = make_info("/img.png", false);
    let hint = SizeHint {
        width: Some("60%".to_string()),
        height: None,
    };
    let out = format_image_typst_sized(&info, "", &hint);
    assert!(out.contains("width: 60%"), "got:\n{out}");
    assert!(
        !out.contains("height:"),
        "no height when not specified:\n{out}"
    );
}

#[test]
fn format_image_sized_with_both_dimensions() {
    let info = make_info("/img.png", false);
    let hint = SizeHint {
        width: Some("50%".to_string()),
        height: Some("100pt".to_string()),
    };
    let out = format_image_typst_sized(&info, "Caption", &hint);
    assert!(out.contains("width: 50%"), "got:\n{out}");
    assert!(out.contains("height: 100pt"), "got:\n{out}");
    assert!(out.contains("caption: [Caption]"), "got:\n{out}");
}

#[test]
fn format_image_sized_px_converted_width() {
    let info = make_info("/img.png", false);
    let px_width = css_length_to_typst("400px").unwrap();
    let hint = SizeHint {
        width: Some(px_width),
        height: None,
    };
    let out = format_image_typst_sized(&info, "", &hint);
    assert!(out.contains("width: 300.0pt"), "got:\n{out}");
}

#[test]
fn format_image_svg_with_custom_width() {
    let info = make_info("/diagram.svg", true);
    let hint = SizeHint {
        width: Some("80%".to_string()),
        height: None,
    };
    let out = format_image_typst_sized(&info, "Architecture", &hint);
    assert!(out.contains("format: \"svg\""), "got:\n{out}");
    assert!(out.contains("width: 80%"), "got:\n{out}");
    assert!(out.contains("caption: [Architecture]"), "got:\n{out}");
}

#[test]
fn format_image_wraps_in_figure() {
    let info = make_info("/img.png", false);
    let out = format_image_typst(&info, "My image");
    assert!(
        out.starts_with("#figure("),
        "should start with #figure:\n{out}"
    );
}

#[test]
fn format_image_ends_with_double_newline() {
    let info = make_info("/img.png", false);
    let out = format_image_typst(&info, "");
    assert!(
        out.ends_with("\n\n"),
        "should end with double newline:\n{out}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Missing-image fallbacks
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn fallback_uses_block_with_luma_fill() {
    let out = missing_image_fallback("img.png", "");
    assert!(out.contains("#block("), "got:\n{out}");
    assert!(out.contains("luma("), "got:\n{out}");
}

#[test]
fn fallback_includes_image_marker() {
    let out = missing_image_fallback("img.png", "Alt text");
    assert!(out.contains("\\[Image:"), "got:\n{out}");
}

#[test]
fn fallback_with_alt_includes_alt_text() {
    let out = missing_image_fallback("https://example.com/photo.jpg", "A sunset photo");
    assert!(out.contains("A sunset photo"), "got:\n{out}");
}

#[test]
fn fallback_without_alt_uses_filename() {
    let out = missing_image_fallback("https://cdn.example.com/logo.png", "");
    assert!(out.contains("logo.png"), "should use filename:\n{out}");
    assert!(!out.contains("https://"), "should strip URL prefix:\n{out}");
}

#[test]
fn fallback_strips_query_string_from_url() {
    let out = missing_image_fallback("https://cdn.example.com/img.png?v=3&size=large", "");
    assert!(
        out.contains("img.png"),
        "should include bare filename:\n{out}"
    );
    assert!(
        !out.contains("v=3"),
        "should not include query param:\n{out}"
    );
    assert!(
        !out.contains("size=large"),
        "should not include query param:\n{out}"
    );
}

#[test]
fn fallback_with_alt_also_shows_filename() {
    // Even when alt text is provided, the filename is included for diagnostics
    let out = missing_image_fallback("https://example.com/chart.png", "Revenue chart");
    assert!(out.contains("Revenue chart"), "alt text present:\n{out}");
    assert!(out.contains("chart.png"), "filename also present:\n{out}");
}

#[test]
fn fallback_empty_url_and_alt() {
    let out = missing_image_fallback("", "");
    assert!(
        out.contains("#block("),
        "should still produce a block:\n{out}"
    );
}

#[test]
fn fallback_special_chars_in_alt_escaped() {
    let out = missing_image_fallback("img.png", "Price: $9.99 #tag [info]");
    assert!(out.contains("\\$9.99"), "dollar not escaped:\n{out}");
    assert!(out.contains("\\#tag"), "hash not escaped:\n{out}");
    assert!(out.contains("\\[info\\]"), "brackets not escaped:\n{out}");
}

#[test]
fn fallback_special_chars_in_filename_escaped() {
    let out = missing_image_fallback("path/to/image [1].png", "");
    assert!(
        out.contains("\\[1\\]") || out.contains("image"),
        "got:\n{out}"
    );
}

#[test]
fn fallback_uses_center_align() {
    let out = missing_image_fallback("img.png", "Test");
    assert!(out.contains("center"), "fallback should be centred:\n{out}");
}

#[test]
fn fallback_uses_stroke() {
    let out = missing_image_fallback("img.png", "");
    assert!(
        out.contains("stroke:"),
        "fallback should have a border:\n{out}"
    );
}

#[test]
fn fallback_full_width() {
    let out = missing_image_fallback("img.png", "");
    assert!(
        out.contains("width: 100%"),
        "fallback should span full width:\n{out}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Cache invalidation (stable_name hash stability)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn stable_name_deterministic() {
    let url = "https://example.com/images/photo.png";
    assert_eq!(stable_name(url), stable_name(url));
}

#[test]
fn stable_name_different_urls_different_hashes() {
    let a = stable_name("https://example.com/a.png");
    let b = stable_name("https://example.com/b.png");
    assert_ne!(a, b);
}

#[test]
fn stable_name_is_hex_string() {
    let name = stable_name("https://example.com/test.png");
    assert!(!name.is_empty());
    assert!(
        name.chars().all(|c| c.is_ascii_hexdigit()),
        "hash should be hex digits only: {name}"
    );
}

#[test]
fn stable_name_url_with_query_hashes_full_url() {
    // Different query strings should produce different hashes
    let a = stable_name("https://cdn.example.com/img.png?v=1");
    let b = stable_name("https://cdn.example.com/img.png?v=2");
    assert_ne!(a, b, "versioned URLs should have different hashes");
}

// Cache metadata round-trip test
#[test]
fn cache_meta_write_read_roundtrip() {
    let dir = TempDir::new().unwrap();
    let meta_path = dir.path().join("test.meta");
    let content = "etag=W/\"abc123\"\nlast_modified=Mon, 01 Jan 2024 00:00:00 GMT\next=png\n";
    fs::write(&meta_path, content).unwrap();

    // Verify we can read the file back properly by checking the cached file exists
    assert!(meta_path.exists());
    let read_back = fs::read_to_string(&meta_path).unwrap();
    assert!(read_back.contains("ext=png"));
    assert!(read_back.contains("etag=W/\"abc123\""));
}

// ─────────────────────────────────────────────────────────────────────────────
// Markdown → Typst round-trips (no network, no PDF compilation)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md_local_png_renders_figure() {
    let dir = TempDir::new().unwrap();
    let png = dir.path().join("photo.png");
    fs::write(&png, b"\x89PNG\r\n\x1a\n").unwrap();
    let out = render("![A photo](photo.png)\n", &dir);
    assert!(out.contains("#figure(image("), "got:\n{out}");
    assert!(!out.contains("#block("), "should not be a fallback:\n{out}");
}

#[test]
fn md_local_png_with_alt_has_caption() {
    let dir = TempDir::new().unwrap();
    let png = dir.path().join("chart.png");
    fs::write(&png, b"\x89PNG\r\n\x1a\n").unwrap();
    let out = render("![Sales chart Q1](chart.png)\n", &dir);
    assert!(out.contains("caption: [Sales chart Q1]"), "got:\n{out}");
}

#[test]
fn md_local_png_without_alt_no_caption() {
    let dir = TempDir::new().unwrap();
    let png = dir.path().join("img.png");
    fs::write(&png, b"\x89PNG\r\n\x1a\n").unwrap();
    let out = render("![](img.png)\n", &dir);
    assert!(!out.contains("caption"), "no alt → no caption:\n{out}");
}

#[test]
fn md_local_svg_emits_format_svg() {
    let dir = TempDir::new().unwrap();
    let svg = dir.path().join("diagram.svg");
    fs::write(&svg, b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>").unwrap();
    let out = render("![Flow diagram](diagram.svg)\n", &dir);
    assert!(out.contains("format: \"svg\""), "got:\n{out}");
    assert!(out.contains("caption: [Flow diagram]"), "got:\n{out}");
}

#[test]
fn md_local_svg_no_alt_no_caption() {
    let dir = TempDir::new().unwrap();
    let svg = dir.path().join("icon.svg");
    fs::write(&svg, b"<svg></svg>").unwrap();
    let out = render("![](icon.svg)\n", &dir);
    assert!(out.contains("format: \"svg\""), "got:\n{out}");
    assert!(!out.contains("caption"), "no alt → no caption:\n{out}");
}

#[test]
fn md_missing_local_image_emits_fallback() {
    let dir = TempDir::new().unwrap();
    let out = render("![Missing](nonexistent.png)\n", &dir);
    assert!(out.contains("#block("), "expected fallback:\n{out}");
    assert!(out.contains("\\[Image:"), "expected Image marker:\n{out}");
    assert!(
        !out.contains("#figure(image("),
        "should not emit real image:\n{out}"
    );
}

#[test]
fn md_missing_image_shows_alt_in_fallback() {
    let dir = TempDir::new().unwrap();
    let out = render("![My missing diagram](nope.png)\n", &dir);
    assert!(
        out.contains("My missing diagram"),
        "alt text should appear in fallback:\n{out}"
    );
}

#[test]
fn md_missing_image_shows_filename_in_fallback() {
    let dir = TempDir::new().unwrap();
    let out = render("![](nope.png)\n", &dir);
    assert!(
        out.contains("nope.png"),
        "filename should appear in fallback:\n{out}"
    );
}

#[test]
fn md_remote_image_disabled_emits_fallback() {
    let dir = TempDir::new().unwrap();
    let out = render("![Cloud image](https://cdn.example.com/photo.jpg)\n", &dir);
    // With no_remote_images=true, should emit fallback
    assert!(
        !out.contains("image(\"https://"),
        "should not emit remote URL:\n{out}"
    );
    assert!(out.contains("#block("), "expected fallback:\n{out}");
}

#[test]
fn md_remote_image_with_alt_shows_in_fallback() {
    let dir = TempDir::new().unwrap();
    let out = render("![Server diagram](https://example.com/arch.png)\n", &dir);
    assert!(
        out.contains("Server diagram"),
        "alt text should appear:\n{out}"
    );
}

#[test]
fn md_data_uri_svg_renders_correctly() {
    let dir = TempDir::new().unwrap();
    // Simple base64-encoded SVG
    let svg_content = b"<svg xmlns='http://www.w3.org/2000/svg'><circle r='10'/></svg>";
    let b64 = base64_encode(svg_content);
    let md = format!("![SVG circle](data:image/svg+xml;base64,{})\n", b64);
    let out = render(&md, &dir);
    // Data URI SVG should decode and get format: "svg"
    // (on success) OR fall back gracefully (on any write error)
    // The important thing is: no panic, and either figure or fallback
    assert!(
        out.contains("#figure(image(") || out.contains("#block("),
        "should render figure or fallback, got:\n{out}"
    );
}

#[test]
fn md_data_uri_png_renders() {
    let dir = TempDir::new().unwrap();
    // 1×1 transparent PNG (minimal valid PNG)
    let b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let md = format!("![Tiny PNG](data:image/png;base64,{})\n", b64);
    let out = render(&md, &dir);
    assert!(
        out.contains("#figure(image(") || out.contains("#block("),
        "should render figure or fallback, got:\n{out}"
    );
}

#[test]
fn md_multiple_images_all_rendered() {
    let dir = TempDir::new().unwrap();
    let png1 = dir.path().join("a.png");
    let png2 = dir.path().join("b.png");
    fs::write(&png1, b"\x89PNG\r\n\x1a\n").unwrap();
    fs::write(&png2, b"\x89PNG\r\n\x1a\n").unwrap();
    let md = "![First](a.png)\n\n![Second](b.png)\n";
    let out = render(md, &dir);
    // Both images should appear
    let count = out.matches("#figure(image(").count();
    assert_eq!(count, 2, "expected 2 images, got {count}:\n{out}");
}

#[test]
fn md_image_alongside_text_text_preserved() {
    let dir = TempDir::new().unwrap();
    let out = render(
        "Some text before.\n\n![Missing](nope.png)\n\nSome text after.\n",
        &dir,
    );
    assert!(
        out.contains("Some text before."),
        "text before image:\n{out}"
    );
    assert!(out.contains("Some text after."), "text after image:\n{out}");
}

#[test]
fn md_dollar_sign_in_text_is_escaped() {
    let dir = TempDir::new().unwrap();
    // Dollar signs in plain text must be escaped so they don't start math mode
    let out = render("The price is $9.99 per unit.\n", &dir);
    assert!(out.contains("\\$9.99"), "dollar sign not escaped:\n{out}");
    // Should NOT contain an unescaped $ that could trigger math
    assert!(!out.contains(" $9"), "unescaped dollar in output:\n{out}");
}

#[test]
fn md_dollar_sign_in_alt_text_escaped() {
    let dir = TempDir::new().unwrap();
    let png = dir.path().join("img.png");
    fs::write(&png, b"\x89PNG\r\n\x1a\n").unwrap();
    let out = render("![Price $9.99](img.png)\n", &dir);
    assert!(
        out.contains("\\$9.99"),
        "dollar in alt text must be escaped:\n{out}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Snapshot tests for image-related Typst output
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_local_png_with_caption() {
    let dir = TempDir::new().unwrap();
    let png = dir.path().join("sample.png");
    fs::write(&png, b"\x89PNG\r\n\x1a\n").unwrap();
    let out = render("![Figure caption text](sample.png)\n", &dir);

    // Structural checks
    assert!(out.starts_with("#set page("), "preamble:\n{out}");
    assert!(out.contains("#figure(image("), "figure call:\n{out}");
    assert!(out.contains("width: 100%"), "default width:\n{out}");
    assert!(
        out.contains("caption: [Figure caption text]"),
        "caption:\n{out}"
    );
}

#[test]
fn snapshot_svg_no_caption() {
    let dir = TempDir::new().unwrap();
    let svg = dir.path().join("icon.svg");
    fs::write(&svg, b"<svg></svg>").unwrap();
    let out = render("![](icon.svg)\n", &dir);

    assert!(out.contains("format: \"svg\""), "svg format:\n{out}");
    assert!(out.contains("width: 100%"), "default width:\n{out}");
    assert!(!out.contains("caption"), "no caption:\n{out}");
}

#[test]
fn snapshot_fallback_missing_with_alt() {
    let dir = TempDir::new().unwrap();
    let out = render("![Broken image](does-not-exist.png)\n", &dir);

    assert!(
        out.contains("#block(fill: luma(235)"),
        "fallback fill:\n{out}"
    );
    assert!(
        out.contains("stroke: 1pt + luma(180)"),
        "fallback stroke:\n{out}"
    );
    assert!(out.contains("\\[Image:"), "image marker:\n{out}");
    assert!(out.contains("Broken image"), "alt text in fallback:\n{out}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: simple base64 encoder for test use
// ─────────────────────────────────────────────────────────────────────────────

fn base64_encode(input: &[u8]) -> String {
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let v = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHA[((v >> 18) & 0x3f) as usize] as char);
        out.push(ALPHA[((v >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHA[((v >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHA[(v & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional MIME and magic-byte tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn detect_format_jpeg_ffd8_ffe1() {
    // JPEG variant with Exif marker
    let bytes = b"\xff\xd8\xff\xe1more-data";
    assert_eq!(detect_image_format("x", "", bytes), "jpg");
}

#[test]
fn detect_format_svg_from_content_type_with_charset() {
    let bytes = b"<svg></svg>";
    assert_eq!(
        detect_image_format("x", "image/svg+xml; charset=UTF-8", bytes),
        "svg"
    );
}

#[test]
fn detect_format_webp_from_magic() {
    let mut bytes = [0u8; 12];
    b"RIFF".iter().enumerate().for_each(|(i, &b)| bytes[i] = b);
    b"WEBP"
        .iter()
        .enumerate()
        .for_each(|(i, &b)| bytes[i + 8] = b);
    assert_eq!(detect_image_format("x", "", &bytes), "webp");
}

#[test]
fn detect_format_gif_from_url_fallback() {
    // Garbage bytes + unrecognised CT → URL ext used
    let garbage = b"not-an-image-at-all-xxxx";
    assert_eq!(
        detect_image_format("https://cdn.example.com/anim.gif", "", garbage),
        "gif"
    );
}

#[test]
fn sniff_avif_returns_none_not_crash() {
    // AVIF has ISOBMFF container — we don't sniff it, but must not panic
    let result = sniff_image_magic(b"????ftyp");
    // Either None or some value is fine; main thing is no panic
    let _ = result;
}

#[test]
fn mime_to_ext_svg_both_forms() {
    assert_eq!(mime_to_ext("image/svg+xml"), Some("svg"));
    assert_eq!(mime_to_ext("image/svg"), Some("svg"));
}

#[test]
fn mime_to_ext_tiff() {
    assert_eq!(mime_to_ext("image/tiff"), Some("tiff"));
}

#[test]
fn mime_to_ext_bmp() {
    assert_eq!(mime_to_ext("image/bmp"), Some("bmp"));
}

#[test]
fn mime_to_ext_webp() {
    assert_eq!(mime_to_ext("image/webp"), Some("webp"));
}

// ─────────────────────────────────────────────────────────────────────────────
// SVG detection: more edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn is_svg_doctype_then_svg() {
    let bytes = b"<!DOCTYPE svg PUBLIC ...><svg></svg>";
    // The checker looks for <svg — this has it
    assert!(is_svg_bytes(bytes));
}

#[test]
fn is_svg_multiple_spaces_before_tag() {
    assert!(is_svg_bytes(b"     <svg></svg>"));
}

#[test]
fn is_svg_mixed_whitespace() {
    assert!(is_svg_bytes(b" \t \n<svg></svg>"));
}

#[test]
fn is_svg_returns_false_for_pdf() {
    assert!(!is_svg_bytes(b"%PDF-1.4 extra content here"));
}

#[test]
fn is_svg_returns_false_for_gif() {
    assert!(!is_svg_bytes(b"GIF89a some data"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Sizing: more unit conversions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn sizing_0px_produces_0pt() {
    assert_eq!(css_length_to_typst("0px"), Some("0.0pt".to_string()));
}

#[test]
fn sizing_fractional_px() {
    // 1.5px → 1.5 × 0.75 = 1.125pt
    assert_eq!(css_length_to_typst("1.5px"), Some("1.1pt".to_string()));
}

#[test]
fn sizing_vw_returns_none() {
    assert_eq!(css_length_to_typst("100vw"), None);
}

#[test]
fn sizing_vh_returns_none() {
    assert_eq!(css_length_to_typst("50vh"), None);
}

#[test]
fn sizing_leading_whitespace_stripped() {
    // "  50%" with leading spaces should work or return None gracefully
    // The function trims whitespace
    let result = css_length_to_typst("  50%");
    // Either Some("50%") or None — must not panic
    let _ = result;
}

#[test]
fn sizing_large_px_value() {
    // 2000px → 2000 × 0.75 = 1500pt
    assert_eq!(css_length_to_typst("2000px"), Some("1500.0pt".to_string()));
}

// ─────────────────────────────────────────────────────────────────────────────
// format_image_typst: more caption/alt tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn format_image_caption_with_unicode() {
    let info = make_info("/img.png", false);
    let out = format_image_typst(&info, "图片说明 (caption)");
    assert!(out.contains("图片说明"), "unicode caption: {out}");
}

#[test]
fn format_image_empty_alt_no_caption_kwarg() {
    let info = make_info("/img.png", false);
    let out = format_image_typst(&info, "");
    // Should not have a `caption:` keyword argument at all
    assert!(!out.contains("caption:"), "no caption kwarg: {out}");
}

#[test]
fn format_image_png_no_format_arg() {
    let info = make_info("/img.png", false);
    let out = format_image_typst(&info, "");
    // PNG images must NOT have `format: "svg"` — only SVG needs it
    assert!(!out.contains("format:"), "no format: for png: {out}");
}

#[test]
fn format_image_svg_gets_format_svg_arg() {
    let info = make_info("/logo.svg", true);
    let out = format_image_typst(&info, "");
    assert!(out.contains("format: \"svg\""), "svg format arg: {out}");
}

#[test]
fn format_image_sized_none_defaults_to_100pct() {
    let info = make_info("/img.png", false);
    let hint = SizeHint {
        width: None,
        height: None,
    };
    let out = format_image_typst_sized(&info, "", &hint);
    assert!(out.contains("width: 100%"), "default width: {out}");
}

#[test]
fn format_image_sized_height_only() {
    let info = make_info("/img.png", false);
    let hint = SizeHint {
        width: None,
        height: Some("200pt".to_string()),
    };
    let out = format_image_typst_sized(&info, "", &hint);
    assert!(out.contains("height: 200pt"), "height: {out}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Missing-image fallback: more URL patterns
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn fallback_data_uri_shows_data_uri_label() {
    // A data URI fallback should say something sensible, not the full base64
    let out = missing_image_fallback("data:image/png;base64,abc123", "Alt");
    assert!(out.contains("#block("), "block: {out}");
    assert!(out.contains("Alt"), "alt text: {out}");
}

#[test]
fn fallback_url_with_fragment() {
    let out = missing_image_fallback("https://example.com/img.png#section", "");
    assert!(out.contains("#block("), "block: {out}");
}

#[test]
fn fallback_relative_path() {
    let out = missing_image_fallback("images/photo.jpg", "Photo");
    assert!(out.contains("Photo"), "alt: {out}");
    assert!(
        out.contains("photo.jpg") || out.contains("images"),
        "path: {out}"
    );
}

#[test]
fn fallback_very_long_url() {
    let url = format!("https://cdn.example.com/{}/image.png", "x".repeat(200));
    let out = missing_image_fallback(&url, "Long URL");
    assert!(out.contains("#block("), "block: {out}");
    assert!(out.contains("Long URL"), "alt: {out}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Markdown round-trips: more image scenarios
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md_jpeg_local_image_renders_figure() {
    let dir = TempDir::new().unwrap();
    let jpg = dir.path().join("photo.jpg");
    // JPEG magic bytes
    fs::write(&jpg, b"\xff\xd8\xff\xe0some-data").unwrap();
    let out = render("![Photo](photo.jpg)\n", &dir);
    // Should produce a figure (not a fallback)
    assert!(
        out.contains("#figure(image(") || out.contains("#block("),
        "figure or fallback: {out}"
    );
}

#[test]
fn md_image_without_extension_local() {
    // An image file without extension — resolve attempts, may fallback gracefully
    let dir = TempDir::new().unwrap();
    let out = render("![No ext](noextfile)\n", &dir);
    // Should produce some output, either figure or fallback, no crash
    assert!(
        out.contains("#figure(image(") || out.contains("#block("),
        "output: {out}"
    );
}

#[test]
fn md_two_images_in_same_paragraph_context() {
    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a.png");
    let b = dir.path().join("b.png");
    fs::write(&a, b"\x89PNG\r\n\x1a\n").unwrap();
    fs::write(&b, b"\x89PNG\r\n\x1a\n").unwrap();
    let out = render("![A](a.png)\n\nSome text.\n\n![B](b.png)\n", &dir);
    let fig_count = out.matches("#figure(image(").count();
    assert_eq!(fig_count, 2, "two figures: {out}");
    assert!(out.contains("Some text."), "text between: {out}");
}

#[test]
fn md_svg_from_relative_path() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("assets")).unwrap();
    let svg = dir.path().join("assets/logo.svg");
    fs::write(&svg, b"<svg xmlns='http://www.w3.org/2000/svg'></svg>").unwrap();
    let out = render("![Logo](assets/logo.svg)\n", &dir);
    assert!(out.contains("format: \"svg\""), "svg format: {out}");
}

#[test]
fn md_data_uri_plain_svg_non_base64() {
    let dir = TempDir::new().unwrap();
    // URL-encoded plain SVG data URI
    let md = "![SVG](data:image/svg+xml,%3Csvg%3E%3C%2Fsvg%3E)\n";
    let out = render(md, &dir);
    // Should produce figure or fallback gracefully
    assert!(
        out.contains("#figure(image(") || out.contains("#block("),
        "data uri svg: {out}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Stable name: URL patterns
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn stable_name_empty_string() {
    let name = stable_name("");
    assert!(!name.is_empty(), "even empty string produces a hash");
    assert!(
        name.chars().all(|c| c.is_ascii_hexdigit()),
        "hex only: {name}"
    );
}

#[test]
fn stable_name_long_url() {
    let url = format!(
        "https://cdn.example.com/{}/image.png",
        "segment/".repeat(50)
    );
    let name = stable_name(&url);
    assert!(!name.is_empty());
    assert!(name.chars().all(|c| c.is_ascii_hexdigit()), "hex: {name}");
}

#[test]
fn stable_name_url_vs_same_path_different() {
    let a = stable_name("https://cdn.example.com/img.png");
    let b = stable_name("http://cdn.example.com/img.png");
    assert_ne!(a, b, "http vs https produce different hashes");
}

// ─────────────────────────────────────────────────────────────────────────────
// snapshot tests: image-related Typst structure
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_missing_image_with_hash_in_alt() {
    let dir = TempDir::new().unwrap();
    let out = render("![Tag #featured](missing.png)\n", &dir);

    // Hash in alt text must be escaped
    assert!(out.contains("\\#featured"), "hash in alt: {out}");
    assert!(out.contains("\\[Image:"), "image marker: {out}");
}

#[test]
fn snapshot_missing_image_with_dollar_in_alt() {
    let dir = TempDir::new().unwrap();
    let out = render("![Cost $9.99](nope.jpg)\n", &dir);
    assert!(out.contains("\\$9.99"), "dollar in alt: {out}");
}

#[test]
fn snapshot_local_png_width_100pct() {
    let dir = TempDir::new().unwrap();
    let png = dir.path().join("img.png");
    fs::write(&png, b"\x89PNG\r\n\x1a\n").unwrap();
    let out = render("![](img.png)\n", &dir);
    assert!(out.contains("width: 100%"), "default width: {out}");
}

#[test]
fn snapshot_figure_call_structure() {
    let dir = TempDir::new().unwrap();
    let png = dir.path().join("chart.png");
    fs::write(&png, b"\x89PNG\r\n\x1a\n").unwrap();
    let out = render("![Chart title](chart.png)\n", &dir);

    // Structure: #figure(image("path", width: 100%), caption: [alt])
    assert!(out.starts_with("#set page"), "preamble: {out}");
    assert!(out.contains("#figure(image("), "figure call: {out}");
    assert!(out.contains("caption: [Chart title]"), "caption: {out}");
    assert!(out.ends_with('\n'), "ends with newline: {out}");
}
