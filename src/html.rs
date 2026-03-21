/// Raw HTML → Typst translation.
///
/// Comrak exposes raw HTML as opaque strings for two node types:
///   - `HtmlInline`  – a tag fragment inside a paragraph, e.g. `<br>` or `<sup>text</sup>`
///   - `HtmlBlock`   – one or more complete block-level elements
///
/// We parse these strings with a lightweight hand-rolled scanner (zero extra deps)
/// and map the most common tags to equivalent Typst markup.  Unsupported or unsafe
/// tags are stripped; their text content is passed through where applicable.
///
/// # Supported inline tags
/// `<br>`, `<wbr>`, `<b>`, `<strong>`, `<i>`, `<em>`, `<u>`, `<s>`, `<del>`,
/// `<ins>`, `<mark>`, `<small>`, `<sub>`, `<sup>`, `<code>`, `<kbd>`,
/// `<samp>`, `<var>`, `<span>`, `<a href="…">`, `<abbr>`, `<cite>`, `<dfn>`,
/// `<q>`, `<time>`, `<data>`
///
/// # Supported block tags
/// `<p>`, `<div>`, `<section>`, `<article>`, `<main>`, `<header>`,
/// `<footer>`, `<nav>`, `<aside>`, `<blockquote>`, `<hr>`, `<br>`,
/// `<pre>` (with inner `<code>`), `<ul>`, `<ol>`, `<li>`, `<dl>`, `<dt>`,
/// `<dd>`, `<img>`, `<figure>`, `<figcaption>`, `<table>`, `<thead>`,
/// `<tbody>`, `<tfoot>`, `<tr>`, `<th>`, `<td>`, `<details>`, `<summary>`

// ─── Public entry points ─────────────────────────────────────────────────────

/// Parsed representation of a single inline HTML tag fragment.
/// Used by `TypstRenderer::handle_html_inline` to drive a persistent tag stack.
#[derive(Debug)]
pub enum InlineTag {
    /// `<tag attr="val">` — opens a paired tag.
    Open { name: String, attrs: Vec<(String, String)> },
    /// `</tag>` — closes a previously opened tag.
    Close { name: String },
    /// `<tag />` or a void element like `<br>`.
    SelfClose { name: String },
    /// String that doesn't parse as a valid tag.
    Unknown,
}

/// Parse a single raw inline HTML string (e.g. `"<br>"` or `"<strong>"`) into
/// an [`InlineTag`].  Comrak guarantees each `HtmlInline` node is exactly one
/// tag fragment.
pub fn parse_inline_tag(html: &str) -> InlineTag {
    let tokens = tokenize(html.trim());
    match tokens.into_iter().next() {
        Some(Token::Open { name, attrs }) => InlineTag::Open { name, attrs },
        Some(Token::Close { name }) => InlineTag::Close { name },
        Some(Token::SelfClose { name, .. }) => InlineTag::SelfClose { name },
        _ => InlineTag::Unknown,
    }
}

/// Return `true` for HTML elements that are always void (self-closing).
pub fn is_void_inline(name: &str) -> bool {
    is_void_element(name)
}

/// Map an inline open tag to `(prefix, suffix)` Typst strings.
/// The prefix is emitted immediately; the suffix is stored on the stack and
/// emitted when the matching close tag arrives.
pub fn inline_tag_to_typst(name: &str, attrs: &[(String, String)]) -> (String, String) {
    match name {
        "b" | "strong" => ("#strong[".to_string(), "]".to_string()),
        "i" | "em"     => ("#emph[".to_string(),   "]".to_string()),
        "u"            => ("#underline[".to_string(), "]".to_string()),
        "s" | "del"    => ("#strike[".to_string(), "]".to_string()),
        "ins"          => ("#underline[".to_string(), "]".to_string()),
        "mark"         => ("#highlight[".to_string(), "]".to_string()),
        "small"        => ("#text(size: 0.8em)[".to_string(), "]".to_string()),
        "sub"          => ("#sub[".to_string(), "]".to_string()),
        "sup"          => ("#super[".to_string(), "]".to_string()),
        // Inline monospace — use raw block with backtick delimiter.
        // We use #raw() call form to avoid delimiter collision issues.
        "code" | "kbd" | "samp" | "var" => ("#raw(\"".to_string(), "\")".to_string()),
        "cite" | "dfn" | "abbr" | "acronym" | "q" | "time" | "data" => {
            ("#emph[".to_string(), "]".to_string())
        }
        "span" => {
            let style = attrs.iter().find(|(k, _)| k == "style")
                .map(|(_, v)| v.as_str())
                .unwrap_or("");
            style_to_typst_span(style)
        }
        "a" => {
            let href = attrs.iter().find(|(k, _)| k == "href")
                .map(|(_, v)| v.as_str())
                .unwrap_or("");
            if href.is_empty() {
                (String::new(), String::new())
            } else {
                (
                    format!("#link({}, [", typst_quoted_string(href)),
                    "])".to_string(),
                )
            }
        }
        // Unknown / unsupported: pass content through unchanged.
        _ => (String::new(), String::new()),
    }
}

/// Convert a raw HTML block string to Typst.
/// The block typically ends with a trailing newline; we handle that gracefully.
pub fn block_html_to_typst(html: &str) -> String {
    let tokens = tokenize(html);
    let rendered = render_tokens(&tokens, Context::Block);
    if rendered.trim().is_empty() {
        String::new()
    } else {
        format!("{}\n\n", rendered.trim_end())
    }
}

// ─── Context ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    Inline,
    Block,
}

// ─── Tokeniser ───────────────────────────────────────────────────────────────

/// Minimal HTML token – just enough to drive the Typst translator.
#[derive(Debug, Clone)]
enum Token {
    /// Raw character data between tags.
    Text(String),
    /// Opening tag with name and attributes.
    Open { name: String, attrs: Vec<(String, String)> },
    /// Self-closing tag (`<br />`, `<img … />`).
    SelfClose { name: String, attrs: Vec<(String, String)> },
    /// Closing tag.
    Close { name: String },
    /// HTML comment or DOCTYPE – ignored.
    Comment,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'<' {
            // Find end of tag
            let start = i;
            // Skip past '<'
            i += 1;
            if i >= len {
                tokens.push(Token::Text("<".to_string()));
                break;
            }

            // Comment or DOCTYPE?
            if bytes[i] == b'!' {
                // scan to -->
                let end = find_comment_end(bytes, i);
                i = end;
                tokens.push(Token::Comment);
                continue;
            }

            // Closing tag?
            let closing = bytes[i] == b'/';
            if closing {
                i += 1;
            }

            // Tag name
            let name_start = i;
            while i < len && is_tag_name_char(bytes[i]) {
                i += 1;
            }
            let name = input[name_start..i].to_ascii_lowercase();
            if name.is_empty() {
                // Not a real tag – emit literal '<'
                tokens.push(Token::Text(input[start..i].to_string()));
                continue;
            }

            // Attributes
            let mut attrs: Vec<(String, String)> = Vec::new();
            let mut self_close = false;

            loop {
                skip_whitespace(bytes, &mut i);
                if i >= len || bytes[i] == b'>' {
                    if i < len { i += 1; }
                    break;
                }
                if bytes[i] == b'/' {
                    self_close = true;
                    i += 1;
                    if i < len && bytes[i] == b'>' { i += 1; }
                    break;
                }
                // Attribute name
                let attr_start = i;
                while i < len && bytes[i] != b'=' && bytes[i] != b'>' && bytes[i] != b'/' && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                let attr_name = input[attr_start..i].to_ascii_lowercase();
                skip_whitespace(bytes, &mut i);
                if i < len && bytes[i] == b'=' {
                    i += 1;
                    skip_whitespace(bytes, &mut i);
                    let value = parse_attr_value(bytes, &mut i, input);
                    attrs.push((attr_name, value));
                } else {
                    attrs.push((attr_name, String::new()));
                }
            }

            if closing {
                tokens.push(Token::Close { name });
            } else if self_close || is_void_element(&name) {
                tokens.push(Token::SelfClose { name, attrs });
            } else {
                tokens.push(Token::Open { name, attrs });
            }
        } else {
            // Text content – decode basic HTML entities
            let start = i;
            while i < len && bytes[i] != b'<' {
                i += 1;
            }
            let raw = &input[start..i];
            tokens.push(Token::Text(decode_entities(raw)));
        }
    }
    tokens
}

fn is_tag_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':'
}

fn is_void_element(name: &str) -> bool {
    matches!(name, "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
        | "link" | "meta" | "param" | "source" | "track" | "wbr")
}

fn skip_whitespace(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && bytes[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

fn parse_attr_value(bytes: &[u8], i: &mut usize, input: &str) -> String {
    if *i >= bytes.len() {
        return String::new();
    }
    if bytes[*i] == b'"' || bytes[*i] == b'\'' {
        let quote = bytes[*i];
        *i += 1;
        let start = *i;
        while *i < bytes.len() && bytes[*i] != quote {
            *i += 1;
        }
        let val = input[start..*i].to_string();
        if *i < bytes.len() { *i += 1; } // skip closing quote
        decode_entities(&val)
    } else {
        let start = *i;
        while *i < bytes.len() && !bytes[*i].is_ascii_whitespace() && bytes[*i] != b'>' {
            *i += 1;
        }
        decode_entities(&input[start..*i])
    }
}

fn find_comment_end(bytes: &[u8], mut i: usize) -> usize {
    // i is currently at '!' inside '<!'
    while i < bytes.len() {
        if bytes[i] == b'>' {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

/// Decode a small subset of HTML entities commonly found in Markdown HTML snippets.
fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            for c in chars.by_ref() {
                if c == ';' { break; }
                entity.push(c);
            }
            match entity.as_str() {
                "amp"   => out.push('&'),
                "lt"    => out.push('<'),
                "gt"    => out.push('>'),
                "quot"  => out.push('"'),
                "apos"  => out.push('\''),
                "nbsp"  => out.push('\u{00A0}'),
                "mdash" => out.push('—'),
                "ndash" => out.push('–'),
                "laquo" => out.push('«'),
                "raquo" => out.push('»'),
                "copy"  => out.push('©'),
                "reg"   => out.push('®'),
                "trade" => out.push('™'),
                "hellip"=> out.push('…'),
                other => {
                    // Numeric references: &#123; or &#x7B;
                    if let Some(rest) = other.strip_prefix('#') {
                        let code_point = if let Some(hex) = rest.strip_prefix('x').or_else(|| rest.strip_prefix('X')) {
                            u32::from_str_radix(hex, 16).ok()
                        } else {
                            rest.parse::<u32>().ok()
                        };
                        if let Some(n) = code_point.and_then(char::from_u32) {
                            out.push(n);
                        } else {
                            out.push('&');
                            out.push_str(other);
                            out.push(';');
                        }
                    } else {
                        // Unknown entity – keep as-is
                        out.push('&');
                        out.push_str(other);
                        out.push(';');
                    }
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

// ─── Renderer ────────────────────────────────────────────────────────────────

/// State carried through recursive rendering.
struct RenderState {
    /// Stack of currently-open block tags that need Typst wrappers.
    open_blocks: Vec<OpenBlock>,
    /// When inside a `<pre>`, we preserve whitespace and don't escape Typst tokens.
    in_pre: bool,
    /// When inside `<details>`, we wrap in a collapsible block substitute.
    details_summary: Option<String>,
}

#[derive(Clone, Debug)]
struct OpenBlock {
    tag: String,
    /// Typst suffix emitted after children (closing bracket / call end).
    suffix: String,
}

fn render_tokens(tokens: &[Token], _ctx: Context) -> String {
    let mut state = RenderState {
        open_blocks: Vec::new(),
        in_pre: false,
        details_summary: None,
    };
    let mut out = String::new();
    render_token_slice(tokens, &mut state, &mut out);
    out
}

fn render_token_slice(tokens: &[Token], state: &mut RenderState, out: &mut String) {
    for token in tokens {
        match token {
            Token::Text(text) => {
                if state.in_pre {
                    // Inside <pre> preserve whitespace; escape only for Typst
                    out.push_str(&escape_typst_text_verbatim(text));
                } else {
                    out.push_str(&escape_typst_text(text));
                }
            }
            Token::Comment => { /* skip */ }
            Token::SelfClose { name, attrs } => {
                let rendered = render_self_closing(name, attrs, state);
                out.push_str(&rendered);
            }
            Token::Open { name, attrs } => {
                render_open_tag(name, attrs, state, out);
            }
            Token::Close { name } => {
                render_close_tag(name, state, out);
            }
        }
    }
    // Close any tags left open (malformed HTML)
    let remaining: Vec<OpenBlock> = state.open_blocks.drain(..).collect();
    for block in remaining.into_iter().rev() {
        out.push_str(&block.suffix);
    }
}

// ── Self-closing ──────────────────────────────────────────────────────────────

fn render_self_closing(name: &str, attrs: &[(String, String)], _state: &mut RenderState) -> String {
    match name {
        "br" | "wbr" => "\\\n".to_string(),
        "hr" => "#line(length: 100%)\n\n".to_string(),
        "img" => render_img_tag(attrs),
        // Everything else is silently dropped.
        _ => String::new(),
    }
}

fn render_img_tag(attrs: &[(String, String)]) -> String {
    let src = attr(attrs, "src").unwrap_or_default();
    let alt = attr(attrs, "alt").unwrap_or_default();
    let width = attr(attrs, "width");

    if src.is_empty() {
        return String::new();
    }

    let width_arg = if let Some(w) = width {
        // Might be "200px", "50%", "200" – do a best-effort conversion to Typst
        let w = w.trim_end_matches("px").trim_end_matches('%');
        if let Ok(n) = w.parse::<u32>() {
            format!(", width: {}pt", n)
        } else {
            ", width: 100%".to_string()
        }
    } else {
        ", width: 100%".to_string()
    };

    let caption = if alt.is_empty() {
        String::new()
    } else {
        format!(", caption: [{}]", escape_typst_text(&alt))
    };

    format!(
        "#figure(image({}{}){})\n\n",
        typst_quoted_string(&src),
        width_arg,
        caption
    )
}

// ── Open / close tags ─────────────────────────────────────────────────────────

fn render_open_tag(name: &str, attrs: &[(String, String)], state: &mut RenderState, out: &mut String) {
    match name {
        // ── Inline formatting ──────────────────────────────────────────────────
        "b" | "strong" => push_inline_wrap(state, out, name, "#strong[", "]"),
        "i" | "em"     => push_inline_wrap(state, out, name, "#emph[", "]"),
        "u"            => push_inline_wrap(state, out, name, "#underline[", "]"),
        "s" | "del"    => push_inline_wrap(state, out, name, "#strike[", "]"),
        "ins"          => push_inline_wrap(state, out, name, "#underline[", "]"),
        "mark"         => push_inline_wrap(state, out, name, "#highlight[", "]"),
        "small"        => push_inline_wrap(state, out, name, "#text(size: 0.8em)[", "]"),
        "sub"          => push_inline_wrap(state, out, name, "#sub[", "]"),
        "sup"          => push_inline_wrap(state, out, name, "#super[", "]"),
        "code" | "kbd" | "samp" | "var" => {
            if state.in_pre {
                // Inside <pre> the content is already verbatim — just pass through.
                push_inline_wrap(state, out, name, "", "");
            } else {
                push_inline_wrap(state, out, name, "`", "`");
            }
        }
        "abbr" | "acronym" | "cite" | "dfn" | "q" | "time" | "data" => {
            // Render as italic (closest semantic equivalent)
            push_inline_wrap(state, out, name, "#emph[", "]");
        }
        "span" => {
            // Try to extract colour or font-size from a simple style attribute.
            let style = attr(attrs, "style").unwrap_or_default();
            let (prefix, suffix) = style_to_typst_span(&style);
            push_inline_wrap(state, out, name, &prefix, &suffix);
        }
        "a" => {
            let href = attr(attrs, "href").unwrap_or_default();
            if href.is_empty() {
                push_inline_wrap(state, out, name, "", "");
            } else {
                let prefix = format!("#link({}, [", typst_quoted_string(&href));
                push_inline_wrap(state, out, name, &prefix, "])");
            }
        }

        // ── Block containers ───────────────────────────────────────────────────
        "p" => push_block_wrap(state, out, name, "", "\n\n"),
        "div" | "section" | "article" | "main" | "header" | "footer" | "nav" | "aside" => {
            push_block_wrap(state, out, name, "", "\n\n")
        }
        "blockquote" => push_block_wrap(
            state, out, name,
            "#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[\n",
            "\n]\n\n",
        ),

        // ── Preformatted ───────────────────────────────────────────────────────
        "pre" => {
            state.in_pre = true;
            push_block_wrap(
                state, out, name,
                "#block(fill: luma(245), inset: 10pt, radius: 4pt)[#text(font: (\"DejaVu Sans Mono\",), size: 10pt)[\n",
                "\n]]\n\n",
            );
        }

        // ── Lists ──────────────────────────────────────────────────────────────
        "ul" => push_block_wrap(state, out, name, "", "\n"),
        "ol" => push_block_wrap(state, out, name, "", "\n"),
        "li" => {
            // Determine ordered or unordered by checking the stack.
            let marker = if state.open_blocks.iter().rev().any(|b| b.tag == "ol") {
                "+ "
            } else {
                "- "
            };
            push_block_wrap(state, out, name, marker, "\n");
        }
        "dl" => push_block_wrap(state, out, name, "", "\n\n"),
        "dt" => push_block_wrap(state, out, name, "#strong[", "]\n"),
        "dd" => push_block_wrap(state, out, name, "#block(inset: (left: 12pt))[", "]\n"),

        // ── Table ──────────────────────────────────────────────────────────────
        // Full table structure is complex; we accumulate a simple grid.
        "table" => push_block_wrap(
            state, out, name,
            "#table(columns: 1, stroke: luma(180), inset: 6pt, align: left,\n",
            "\n)\n\n",
        ),
        "thead" | "tbody" | "tfoot" => push_block_wrap(state, out, name, "", ""),
        "tr" => push_block_wrap(state, out, name, "", ""),
        "th" => push_block_wrap(state, out, name, "[#strong[", "]],"),
        "td" => push_block_wrap(state, out, name, "[", "],"),

        // ── Figure ────────────────────────────────────────────────────────────
        "figure" => push_block_wrap(state, out, name, "", "\n\n"),
        "figcaption" => push_block_wrap(state, out, name, "#text(size: 0.9em, style: \"italic\")[", "]\n"),

        // ── Details / summary ─────────────────────────────────────────────────
        "details" => {
            state.details_summary = None;
            push_block_wrap(
                state, out, name,
                "#block(stroke: 1pt + luma(180), inset: 8pt, radius: 4pt)[\n",
                "\n]\n\n",
            );
        }
        "summary" => push_block_wrap(state, out, name, "#strong[", "]\n"),

        // ── Headings inside HTML blocks ────────────────────────────────────────
        "h1" => push_block_wrap(state, out, name, "= ", "\n\n"),
        "h2" => push_block_wrap(state, out, name, "== ", "\n\n"),
        "h3" => push_block_wrap(state, out, name, "=== ", "\n\n"),
        "h4" => push_block_wrap(state, out, name, "==== ", "\n\n"),
        "h5" => push_block_wrap(state, out, name, "===== ", "\n\n"),
        "h6" => push_block_wrap(state, out, name, "====== ", "\n\n"),

        // ── Anything else: pass through content, strip tag ────────────────────
        _ => push_inline_wrap(state, out, name, "", ""),
    }
}

fn render_close_tag(name: &str, state: &mut RenderState, out: &mut String) {
    // Pop matching block from the stack.
    if let Some(pos) = state.open_blocks.iter().rposition(|b| b.tag == name) {
        // Close any implicitly-open inner tags first.
        let inner_blocks: Vec<OpenBlock> = state.open_blocks.drain(pos + 1..).collect();
        for b in inner_blocks.into_iter().rev() {
            out.push_str(&b.suffix);
        }
        let block = state.open_blocks.remove(pos);
        // For code inside pre, the backtick wrapping is already in the block prefix/suffix,
        // so we do NOT re-apply code escaping here.
        if block.tag == "pre" {
            state.in_pre = false;
        }
        out.push_str(&block.suffix);
    }
    // Unknown closing tag without matching open → ignore.
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn push_inline_wrap(state: &mut RenderState, out: &mut String, tag: &str, prefix: &str, suffix: &str) {
    out.push_str(prefix);
    state.open_blocks.push(OpenBlock {
        tag: tag.to_string(),
        suffix: suffix.to_string(),
    });
}

fn push_block_wrap(state: &mut RenderState, out: &mut String, tag: &str, prefix: &str, suffix: &str) {
    out.push_str(prefix);
    state.open_blocks.push(OpenBlock {
        tag: tag.to_string(),
        suffix: suffix.to_string(),
    });
}

/// Parse a tiny subset of the CSS `style` attribute and return (prefix, suffix) Typst spans.
/// Only `color` and `font-size` are handled; anything else is passed through as plain.
fn style_to_typst_span(style: &str) -> (String, String) {
    let mut color: Option<String> = None;
    let mut font_size: Option<String> = None;

    for decl in style.split(';') {
        let decl = decl.trim();
        if let Some((prop, val)) = decl.split_once(':') {
            let prop = prop.trim().to_ascii_lowercase();
            let val = val.trim();
            match prop.as_str() {
                "color" => color = Some(css_color_to_typst(val)),
                "font-size" => font_size = Some(css_size_to_typst(val)),
                _ => {}
            }
        }
    }

    match (color, font_size) {
        (Some(c), Some(s)) => (
            format!("#text(fill: {c}, size: {s})["),
            "]".to_string(),
        ),
        (Some(c), None) => (
            format!("#text(fill: {c})["),
            "]".to_string(),
        ),
        (None, Some(s)) => (
            format!("#text(size: {s})["),
            "]".to_string(),
        ),
        (None, None) => (String::new(), String::new()),
    }
}

/// Convert a CSS colour value to a Typst colour expression.
/// Supports: named colours, `#rrggbb`, `#rgb`, `rgb(r,g,b)`.
fn css_color_to_typst(val: &str) -> String {
    let v = val.trim().to_ascii_lowercase();
    // Named colours
    let named = match v.as_str() {
        "red"    => Some("red"),
        "green"  => Some("green"),
        "blue"   => Some("blue"),
        "white"  => Some("white"),
        "black"  => Some("black"),
        "gray" | "grey" => Some("gray"),
        "orange" => Some("orange"),
        "purple" => Some("purple"),
        "yellow" => Some("yellow"),
        "pink"   => Some("rgb(\"#ffc0cb\")"),
        _        => None,
    };
    if let Some(n) = named {
        return n.to_string();
    }
    // Hex
    if v.starts_with('#') {
        return format!("rgb(\"{}\")", v);
    }
    // rgb(r, g, b)
    if let Some(inner) = v.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].trim().parse::<u8>(),
                parts[1].trim().parse::<u8>(),
                parts[2].trim().parse::<u8>(),
            ) {
                return format!("rgb(\"#{:02x}{:02x}{:02x}\")", r, g, b);
            }
        }
    }
    // Fallback
    format!("rgb(\"{}\")", v)
}

/// Convert a CSS size value to a Typst size expression.
fn css_size_to_typst(val: &str) -> String {
    let v = val.trim();
    if v.ends_with("px") {
        format!("{}pt", v.trim_end_matches("px"))
    } else if v.ends_with("em") || v.ends_with("rem") {
        let n = v.trim_end_matches("rem").trim_end_matches("em");
        format!("{}em", n)
    } else if v.ends_with("pt") {
        v.to_string()
    } else {
        format!("{}pt", v)
    }
}

fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<String> {
    attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
}

// ─── Typst escaping ───────────────────────────────────────────────────────────

/// Escape text for use inside Typst content blocks.
pub fn escape_typst_text(s: &str) -> String {
    s.replace('\n', " ")
        .replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('"', "\\\"")
}

/// Escape text inside `<pre>` blocks – preserves newlines as Typst linebreaks.
fn escape_typst_text_verbatim(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for line in s.split('\n') {
        let escaped = line
            .replace('\\', "\\\\")
            .replace('#', "\\#")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('"', "\\\"");
        out.push_str(&escaped);
        out.push_str("\\\n");
    }
    out
}

fn typst_quoted_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Tokeniser ─────────────────────────────────────────────────────────────

    #[test]
    fn tokenise_simple_text() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Text(t) if t == "hello world"));
    }

    #[test]
    fn tokenise_br() {
        let tokens = tokenize("<br>");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::SelfClose { name, .. } if name == "br"));
    }

    #[test]
    fn tokenise_br_self_closing() {
        let tokens = tokenize("<br />");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::SelfClose { name, .. } if name == "br"));
    }

    #[test]
    fn tokenise_open_close() {
        let tokens = tokenize("<strong>hello</strong>");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], Token::Open { name, .. } if name == "strong"));
        assert!(matches!(&tokens[1], Token::Text(t) if t == "hello"));
        assert!(matches!(&tokens[2], Token::Close { name } if name == "strong"));
    }

    #[test]
    fn tokenise_attributes() {
        let tokens = tokenize("<a href=\"https://example.com\">click</a>");
        assert_eq!(tokens.len(), 3);
        if let Token::Open { name, attrs } = &tokens[0] {
            assert_eq!(name, "a");
            assert_eq!(attrs[0], ("href".to_string(), "https://example.com".to_string()));
        } else {
            panic!("expected Open token");
        }
    }

    #[test]
    fn tokenise_single_quoted_attr() {
        let tokens = tokenize("<img src='logo.png' alt='logo'>");
        assert_eq!(tokens.len(), 1);
        if let Token::SelfClose { attrs, .. } = &tokens[0] {
            assert_eq!(attrs[0].1, "logo.png");
            assert_eq!(attrs[1].1, "logo");
        } else {
            panic!("expected SelfClose token");
        }
    }

    #[test]
    fn tokenise_html_entities_in_text() {
        let tokens = tokenize("a &amp; b &lt;c&gt;");
        assert!(matches!(&tokens[0], Token::Text(t) if t == "a & b <c>"));
    }

    #[test]
    fn tokenise_comment() {
        let tokens = tokenize("<!-- ignored -->text");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0], Token::Comment));
        assert!(matches!(&tokens[1], Token::Text(t) if t == "text"));
    }

    // ── Inline renderer (stateful API) ────────────────────────────────────────
    //
    // Comrak emits HtmlInline as individual tag fragments.  The stateful stack
    // is owned by TypstRenderer.  Here we test the lower-level building blocks:
    //   - `parse_inline_tag`  — parses one tag string into an InlineTag
    //   - `inline_tag_to_typst` — maps a tag name+attrs to (prefix, suffix)
    //
    // For full round-trip tests we simulate the renderer stack manually.

    /// Simulate the TypstRenderer inline HTML stack for tests.
    struct FakeRenderer {
        stack: Vec<(String, String)>, // (tag, suffix)
    }
    impl FakeRenderer {
        fn new() -> Self { Self { stack: Vec::new() } }
        fn handle(&mut self, html: &str) -> String {
            match parse_inline_tag(html) {
                InlineTag::SelfClose { ref name } | InlineTag::Open { ref name, .. }
                    if is_void_inline(name) =>
                {
                    match name.as_str() {
                        "br" | "wbr" => "\\\n".to_string(),
                        "hr" => "#line(length: 100%)\n\n".to_string(),
                        _ => String::new(),
                    }
                }
                InlineTag::Open { name, attrs } => {
                    let (prefix, suffix) = inline_tag_to_typst(&name, &attrs);
                    self.stack.push((name, suffix));
                    prefix
                }
                InlineTag::Close { name } => {
                    if let Some(pos) = self.stack.iter().rposition(|(t, _)| *t == name) {
                        let mut out = String::new();
                        let inner: Vec<_> = self.stack.drain(pos + 1..).collect();
                        for (_, suf) in inner.into_iter().rev() { out.push_str(&suf); }
                        let (_, suf) = self.stack.remove(pos);
                        out.push_str(&suf);
                        out
                    } else {
                        String::new()
                    }
                }
                InlineTag::SelfClose { name } => {
                    let (p, s) = inline_tag_to_typst(&name, &[]);
                    format!("{}{}", p, s)
                }
                InlineTag::Unknown => escape_typst_text(html),
            }
        }
    }

    #[test]
    fn inline_br() {
        let mut r = FakeRenderer::new();
        assert_eq!(r.handle("<br>"), "\\\n");
    }

    #[test]
    fn inline_strong() {
        let mut r = FakeRenderer::new();
        let open = r.handle("<strong>");
        let text = escape_typst_text("hello");
        let close = r.handle("</strong>");
        assert_eq!(format!("{}{}{}", open, text, close), "#strong[hello]");
    }

    #[test]
    fn inline_em() {
        let mut r = FakeRenderer::new();
        let out = format!("{}{}{}",
            r.handle("<em>"), escape_typst_text("world"), r.handle("</em>"));
        assert_eq!(out, "#emph[world]");
    }

    #[test]
    fn inline_b_and_i() {
        let mut r = FakeRenderer::new();
        let out = format!("{}{}{}{}{}{}{}",
            r.handle("<b>"), escape_typst_text("bold"), r.handle("</b>"),
            escape_typst_text(" and "),
            r.handle("<i>"), escape_typst_text("italic"), r.handle("</i>"));
        assert!(out.contains("#strong[bold]"), "got: {out}");
        assert!(out.contains("#emph[italic]"), "got: {out}");
    }

    #[test]
    fn inline_sup_sub() {
        let mut r = FakeRenderer::new();
        let sup = format!("{}{}{}", r.handle("<sup>"), escape_typst_text("2"), r.handle("</sup>"));
        assert_eq!(sup, "#super[2]");
        let sub = format!("{}{}{}", r.handle("<sub>"), escape_typst_text("n"), r.handle("</sub>"));
        assert_eq!(sub, "#sub[n]");
    }

    #[test]
    fn inline_u_and_s() {
        let mut r = FakeRenderer::new();
        let u = format!("{}{}{}", r.handle("<u>"), escape_typst_text("underline"), r.handle("</u>"));
        assert_eq!(u, "#underline[underline]");
        let s = format!("{}{}{}", r.handle("<s>"), escape_typst_text("strike"), r.handle("</s>"));
        assert_eq!(s, "#strike[strike]");
    }

    #[test]
    fn inline_mark() {
        let mut r = FakeRenderer::new();
        let out = format!("{}{}{}", r.handle("<mark>"), escape_typst_text("highlighted"), r.handle("</mark>"));
        assert_eq!(out, "#highlight[highlighted]");
    }

    #[test]
    fn inline_small() {
        let mut r = FakeRenderer::new();
        let out = format!("{}{}{}", r.handle("<small>"), escape_typst_text("tiny"), r.handle("</small>"));
        assert_eq!(out, "#text(size: 0.8em)[tiny]");
    }

    #[test]
    fn inline_code() {
        let mut r = FakeRenderer::new();
        // code uses #raw("…") form — the text content must be unescaped for raw
        let out = format!("{}{}{}", r.handle("<code>"), "foo()", r.handle("</code>"));
        assert!(out.contains("foo()"), "got: {out}");
        assert!(out.contains("#raw("), "got: {out}");
    }

    #[test]
    fn inline_a_with_href() {
        let mut r = FakeRenderer::new();
        let out = format!("{}{}{}", r.handle("<a href=\"https://rust-lang.org\">"), escape_typst_text("Rust"), r.handle("</a>"));
        assert!(out.contains("#link(\"https://rust-lang.org\", ["), "got: {out}");
        assert!(out.contains("Rust"), "got: {out}");
    }

    #[test]
    fn inline_span_color() {
        let mut r = FakeRenderer::new();
        let out = format!("{}{}{}", r.handle("<span style=\"color: red\">"), escape_typst_text("alert"), r.handle("</span>"));
        assert!(out.contains("#text(fill: red)["), "got: {out}");
        assert!(out.contains("alert"), "got: {out}");
    }

    #[test]
    fn inline_span_font_size_px() {
        let mut r = FakeRenderer::new();
        let out = format!("{}{}{}", r.handle("<span style=\"font-size: 14px\">"), escape_typst_text("small"), r.handle("</span>"));
        assert!(out.contains("#text(size: 14pt)["), "got: {out}");
    }

    #[test]
    fn inline_nested() {
        // <b><em>bold-italic</em></b>
        let mut r = FakeRenderer::new();
        let out = format!("{}{}{}{}{}",
            r.handle("<b>"),
            r.handle("<em>"),
            escape_typst_text("bold-italic"),
            r.handle("</em>"),
            r.handle("</b>"),
        );
        assert_eq!(out, "#strong[#emph[bold-italic]]");
    }

    #[test]
    fn inline_unknown_tag_passes_content() {
        let mut r = FakeRenderer::new();
        // Unknown open tag → no prefix/suffix
        let open = r.handle("<blink>");
        let text = escape_typst_text("old school");
        let close = r.handle("</blink>");
        assert_eq!(format!("{}{}{}", open, text, close), "old school");
    }

    // ── Block renderer ────────────────────────────────────────────────────────

    #[test]
    fn block_hr() {
        let out = block_html_to_typst("<hr>\n");
        assert!(out.contains("#line(length: 100%)"), "got: {out}");
    }

    #[test]
    fn block_p() {
        let out = block_html_to_typst("<p>Hello world</p>\n");
        assert!(out.contains("Hello world"), "got: {out}");
    }

    #[test]
    fn block_blockquote() {
        let out = block_html_to_typst("<blockquote>A quote</blockquote>");
        assert!(out.contains("#block(inset"), "got: {out}");
        assert!(out.contains("A quote"), "got: {out}");
    }

    #[test]
    fn block_div_passes_content() {
        let out = block_html_to_typst("<div>content here</div>");
        assert!(out.contains("content here"), "got: {out}");
    }

    #[test]
    fn block_ul_li() {
        let out = block_html_to_typst("<ul><li>Alpha</li><li>Beta</li></ul>");
        assert!(out.contains("- Alpha"), "got: {out}");
        assert!(out.contains("- Beta"), "got: {out}");
    }

    #[test]
    fn block_ol_li() {
        let out = block_html_to_typst("<ol><li>First</li><li>Second</li></ol>");
        assert!(out.contains("+ First"), "got: {out}");
        assert!(out.contains("+ Second"), "got: {out}");
    }

    #[test]
    fn block_img() {
        let out = block_html_to_typst("<img src=\"photo.png\" alt=\"A photo\">\n");
        assert!(out.contains("#figure(image(\"photo.png\""), "got: {out}");
        assert!(out.contains("caption"), "got: {out}");
    }

    #[test]
    fn block_details_summary() {
        let out = block_html_to_typst("<details><summary>Click me</summary>Hidden content</details>");
        assert!(out.contains("#block(stroke"), "got: {out}");
        assert!(out.contains("Click me"), "got: {out}");
        assert!(out.contains("Hidden content"), "got: {out}");
    }

    #[test]
    fn block_heading_tags() {
        let h1 = block_html_to_typst("<h1>Title</h1>");
        assert!(h1.contains("= Title"), "got: {h1}");
        let h3 = block_html_to_typst("<h3>Sub</h3>");
        assert!(h3.contains("=== Sub"), "got: {h3}");
    }

    #[test]
    fn block_pre_code() {
        let out = block_html_to_typst("<pre><code>fn main() {}</code></pre>");
        assert!(out.contains("DejaVu Sans Mono"), "got: {out}");
        assert!(out.contains("fn main()"), "got: {out}");
    }

    #[test]
    fn block_table() {
        let out = block_html_to_typst("<table><tr><th>Name</th><td>Alice</td></tr></table>");
        assert!(out.contains("#table("), "got: {out}");
        assert!(out.contains("Name"), "got: {out}");
        assert!(out.contains("Alice"), "got: {out}");
    }

    // ── Entity decoding ───────────────────────────────────────────────────────

    #[test]
    fn entity_amp() {
        assert_eq!(decode_entities("a &amp; b"), "a & b");
    }

    #[test]
    fn entity_lt_gt() {
        assert_eq!(decode_entities("&lt;tag&gt;"), "<tag>");
    }

    #[test]
    fn entity_nbsp() {
        assert_eq!(decode_entities("a&nbsp;b"), "a\u{00A0}b");
    }

    #[test]
    fn entity_numeric_decimal() {
        assert_eq!(decode_entities("&#65;"), "A");
    }

    #[test]
    fn entity_numeric_hex() {
        assert_eq!(decode_entities("&#x41;"), "A");
    }

    // ── CSS helpers ───────────────────────────────────────────────────────────

    #[test]
    fn css_color_named() {
        assert_eq!(css_color_to_typst("red"), "red");
        assert_eq!(css_color_to_typst("blue"), "blue");
    }

    #[test]
    fn css_color_hex() {
        assert_eq!(css_color_to_typst("#ff0000"), "rgb(\"#ff0000\")");
    }

    #[test]
    fn css_color_rgb_fn() {
        assert_eq!(css_color_to_typst("rgb(255, 0, 0)"), "rgb(\"#ff0000\")");
    }

    #[test]
    fn css_size_px() {
        assert_eq!(css_size_to_typst("16px"), "16pt");
    }

    #[test]
    fn css_size_em() {
        assert_eq!(css_size_to_typst("1.5em"), "1.5em");
    }

    #[test]
    fn css_size_pt() {
        assert_eq!(css_size_to_typst("12pt"), "12pt");
    }
}
