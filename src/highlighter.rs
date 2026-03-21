//! Syntax highlighting for fenced code blocks using syntect.
//!
//! Tokenises source code with the bundled Sublime Text syntaxes and maps each
//! token's foreground colour (and bold/italic style flags) into Typst-native
//! `#text(fill: rgb("…"), …)[…]` spans.  The whole block is wrapped in a
//! dark-background `#block(…)` with a monospace font, matching the feel of a
//! typical code-editor theme.

use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Escape characters that have special meaning inside Typst *content* mode
/// (i.e., inside `[…]` blocks).
///
/// Special characters in Typst content mode that must be escaped with `\`:
///   `\`  – escape character itself
///   `#`  – starts a code expression / function call
///   `[`  – opens a nested content block
///   `]`  – closes a content block
///   `@`  – label / citation shorthand
///   `_`  – italic emphasis delimiter (unclosed → "unclosed delimiter")
///   `*`  – bold emphasis delimiter  (unclosed → "unclosed delimiter")
///   `$`  – opens math mode         (unclosed → "unclosed delimiter")
///
/// Curly braces `{` `}` are **not** special in Typst content mode and must
/// NOT be escaped — Typst reports a syntax error if they are escaped with `\`.
fn escape_typst_content(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '#'  => out.push_str("\\#"),
            '['  => out.push_str("\\["),
            ']'  => out.push_str("\\]"),
            '@'  => out.push_str("\\@"),
            '_'  => out.push_str("\\_"),
            '*'  => out.push_str("\\*"),
            '$'  => out.push_str("\\$"),
            // All other characters (including { } < > ` ~ ^ etc.) are literal.
            other => out.push(other),
        }
    }
    out
}

/// Format an `(r, g, b)` triple as `rgb("#rrggbb")` for Typst.
fn typst_rgb(r: u8, g: u8, b: u8) -> String {
    format!("rgb(\"#{:02x}{:02x}{:02x}\")", r, g, b)
}

// ── public API ────────────────────────────────────────────────────────────────

/// Highlight `code` for the given `lang` and return a self-contained Typst
/// block that can be pasted into a `.typ` source file.
///
/// Rendering strategy
/// ──────────────────
/// * Each token is wrapped in `#text(fill: …)[…]`; bold / italic tokens also
///   get the appropriate weight / style arguments.
/// * Lines are joined with `\n` inside a `#raw(…)` block so that Typst keeps
///   them verbatim (no smart-quote conversion, no ligatures).
/// * We emit the whole thing as a `#block(…)` with a dark background that
///   visually matches a typical dark editor theme.
///
/// Fallback
/// ────────
/// If the language identifier is empty or unknown, the code is rendered
/// uncoloured in the standard monospace block style.
pub fn highlight_code_to_typst(code: &str, lang: &str, mono_font: &str, theme_name: &str) -> String {
    // Lazy-static these; they are expensive to load but tiny once loaded.
    use std::sync::OnceLock;
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    static TS: OnceLock<ThemeSet>  = OnceLock::new();

    let ss = SS.get_or_init(SyntaxSet::load_defaults_newlines);
    let ts = TS.get_or_init(ThemeSet::load_defaults);

    // ── choose theme ──────────────────────────────────────────────────────
    // Available built-in themes:
    //   base16-ocean.dark, base16-ocean.light, base16-eighties.dark,
    //   base16-mocha.dark, base16-ocean.dark, InspiredGitHub,
    //   Solarized (dark), Solarized (light)
    let theme = ts
        .themes
        .get(theme_name)
        .or_else(|| ts.themes.get("base16-ocean.dark"))
        .expect("syntect built-in themes must be present");

    // Use a subtle light-gray if the theme background is pure white or transparent,
    // so code blocks have a visible container even on white-paper PDFs.
    let raw_bg = theme.settings.background.unwrap_or(syntect::highlighting::Color { r: 30, g: 30, b: 30, a: 255 });
    let bg = if raw_bg.r > 240 && raw_bg.g > 240 && raw_bg.b > 240 {
        syntect::highlighting::Color { r: 246, g: 248, b: 250, a: 255 }
    } else {
        raw_bg
    };

    // ── find syntax for the language ──────────────────────────────────────
    let syntax = if lang.is_empty() {
        None
    } else {
        ss.find_syntax_by_token(lang)
            .or_else(|| ss.find_syntax_by_name(lang))
    };

    let syntax = match syntax {
        Some(s) => s,
        None => {
            // Fallback: plain monospace block, no colour
            return plain_code_block(code, mono_font, bg);
        }
    };

    // ── tokenise and render ───────────────────────────────────────────────
    let mut hl = HighlightLines::new(syntax, theme);
    let mut lines_out: Vec<String> = Vec::new();

    for line in LinesWithEndings::from(code) {
        // Strip the trailing newline – Typst par() handles line breaks via
        // the outer `linebreak()` we insert between lines.
        let stripped = line.strip_suffix('\n').unwrap_or(line);
        // Also strip Windows \r\n
        let _stripped = stripped.strip_suffix('\r').unwrap_or(stripped);

        let ranges = hl
            .highlight_line(line, ss)
            .unwrap_or_default();

        let mut tokens_out = String::new();
        for (style, token) in &ranges {
            if token.is_empty() || *token == "\n" || *token == "\r\n" {
                continue;
            }
            // strip trailing newlines from the token itself
            let token_text = token.trim_end_matches('\n').trim_end_matches('\r');
            if token_text.is_empty() {
                continue;
            }

            let fg = style.foreground;
            let fill = typst_rgb(fg.r, fg.g, fg.b);

            let is_bold   = style.font_style.contains(FontStyle::BOLD);
            let is_italic = style.font_style.contains(FontStyle::ITALIC);

            let escaped = escape_typst_content(token_text);

            let span = match (is_bold, is_italic) {
                (true,  true)  => format!("#text(fill: {fill}, weight: \"bold\", style: \"italic\")[{escaped}]"),
                (true,  false) => format!("#text(fill: {fill}, weight: \"bold\")[{escaped}]"),
                (false, true)  => format!("#text(fill: {fill}, style: \"italic\")[{escaped}]"),
                (false, false) => format!("#text(fill: {fill})[{escaped}]"),
            };
            tokens_out.push_str(&span);
        }

        // If the line was blank (e.g. an empty line in the code), emit a
        // thin space so Typst preserves the vertical gap.
        if tokens_out.is_empty() {
            lines_out.push(String::from("#h(0pt)"));
        } else {
            lines_out.push(tokens_out);
        }
    }

    // Remove a trailing blank line that syntect sometimes appends
    while lines_out.last().map_or(false, |l| l == "#h(0pt)") {
        lines_out.pop();
    }

    let body = lines_out.join("\\\n");   // Typst `\` = explicit line-break

    let bg_color = typst_rgb(bg.r, bg.g, bg.b);
    let mono_font_esc = mono_font.replace('"', "\\\"");

    format!(
        "#block(\
fill: {bg_color}, \
inset: (x: 10pt, y: 8pt), \
radius: 4pt, \
width: 100%, \
clip: true\
)[\
#set text(font: (\"{mono_font_esc}\",), size: 9pt)\n\
{body}\
]\n\n"
    )
}

// ── fallback renderer ─────────────────────────────────────────────────────────

fn plain_code_block(
    code: &str,
    mono_font: &str,
    bg: syntect::highlighting::Color,
) -> String {
    let bg_color = typst_rgb(bg.r, bg.g, bg.b);
    let mono_font_esc = mono_font.replace('"', "\\\"");

    // Use #raw("...", block: false) so Typst handles all character escaping.
    // We only need to escape `\` and `"` inside the Typst string literal.
    let trimmed = code.trim_end_matches('\n');
    let escaped_str = typst_escape_string(trimmed);

    format!(
        "#block(\
fill: {bg_color}, \
inset: (x: 10pt, y: 8pt), \
radius: 4pt, \
width: 100%, \
clip: true\
)[#set text(font: (\"{mono_font_esc}\",), size: 9pt)\
\n#raw(\"{escaped_str}\", block: false)]\n\n"
    )
}

/// Escape a string for use inside a Typst double-quoted string literal `"..."`.
/// Only `\` and `"` need escaping; newlines can be literal (Typst string
/// literals support them).
fn typst_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"'  => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out
}
