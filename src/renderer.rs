use anyhow::{Context, Result};
use comrak::{
    Arena, Options,
    nodes::{AstNode, ListType, NodeCodeBlock, NodeHeading, NodeLink, NodeValue},
    parse_document,
};
use std::collections::HashMap;

use crate::highlighter::highlight_code_to_typst;
use crate::html::{InlineTag, block_html_to_typst, inline_tag_to_typst, is_void_inline, parse_inline_tag};
use std::fs;
use std::path::{Path, PathBuf};
use typst::foundations::{Bytes, Datetime, Smart};
use typst::layout::PagedDocument;
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World, compile};
use typst_kit::fonts::{FontSearcher, Fonts};
use typst_pdf::{PdfOptions, pdf};
use typst_syntax::{FileId, Source, VirtualPath};

#[derive(Clone, Debug)]
pub struct FontSet {
    pub regular: String,
    pub monospace: String,
}

impl Default for FontSet {
    fn default() -> Self {
        Self {
            regular: "Libertinus Serif".to_string(),
            monospace: "DejaVu Sans Mono".to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TocEntry {
    pub level: u8,
    pub title: String,
    pub page_number: usize,
}

#[derive(Clone, Debug)]
pub struct RenderConfig {
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub margin_mm: f32,
    pub base_font_size_pt: f32,
    pub fonts: FontSet,
    pub input_path: Option<PathBuf>,
    #[allow(dead_code)]
    pub syntax_theme: String,
}

#[derive(Clone, Debug)]
pub struct RenderSummary {
    pub pages: usize,
    pub toc_entries: Vec<TocEntry>,
}

/// Read markdown from a file path or stdin.
///
/// # Errors
/// Returns an error if stdin or the input file cannot be read.
pub fn read_input(input: &str) -> Result<String> {
    if input == "-" {
        let mut buffer = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer)?;
        Ok(buffer)
    } else {
        fs::read_to_string(input).with_context(|| format!("Failed to read input file: {input}"))
    }
}

/// Render markdown to PDF using a pure Rust Typst backend.
///
/// # Errors
/// Returns an error if markdown conversion, Typst compilation, font loading,
/// or PDF writing fails.
pub fn render_markdown_to_pdf(
    markdown: &str,
    output: &Path,
    config: RenderConfig,
) -> Result<RenderSummary> {
    let typst_source = markdown_to_typst(markdown, &config)?;
    fs::write(output.with_extension("typ"), &typst_source).ok();
    let world = TypstWorld::new(typst_source)?;

    let warned = compile::<PagedDocument>(&world);
    let document = warned
        .output
        .map_err(|errors| anyhow::anyhow!(format_typst_errors(&errors)))?;

    let pdf_bytes = pdf(
        &document,
        &PdfOptions {
            ident: Smart::Auto,
            ..PdfOptions::default()
        },
    )
    .map_err(|errors| anyhow::anyhow!(format_typst_errors(&errors)))?;
    fs::write(output, pdf_bytes)
        .with_context(|| format!("Failed to write PDF to {}", output.display()))?;

    Ok(RenderSummary {
        pages: document.pages.len(),
        toc_entries: extract_toc(markdown),
    })
}

fn markdown_to_typst(markdown: &str, config: &RenderConfig) -> Result<String> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &markdown_options());
    let mut renderer = TypstRenderer::new(config);
    let body = renderer.render_children(root)?;
    let toc = generate_typst_toc(markdown);

    Ok(format!(
        "#set page(width: {page_width}mm, height: {page_height}mm, margin: {margin}mm)\n\
#set text(font: ({font},), size: {font_size}pt)\n\
#show raw: set text(font: ({mono_font},), size: {code_size}pt)\n\
#show link: set text(fill: blue)\n\
{toc}\n\
{body}\n",
        page_width = config.page_width_mm,
        page_height = config.page_height_mm,
        margin = config.margin_mm,
        font = typst_quoted_string(&config.fonts.regular),
        mono_font = typst_quoted_string(&config.fonts.monospace),
        font_size = config.base_font_size_pt,
        code_size = config.base_font_size_pt * 0.92,
        toc = toc,
        body = body,
    ))
}

fn markdown_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.tagfilter = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    options.extension.front_matter_delimiter = Some("---".to_string());
    // Enable `$...$` inline and `$$...$$` display math parsing
    options.extension.math_dollars = true;
    // Enable ```math ... ``` fenced blocks and $`...`$ syntax
    options.extension.math_code = true;
    options.parse.smart = true;
    options.render.unsafe_ = true;
    options
}

/// Entry on the inline HTML stack: tracks the Typst suffix emitted on close.
struct InlineHtmlFrame {
    tag: String,
    suffix: String,
}

struct TypstRenderer {
    asset_root: PathBuf,
    cache_dir: PathBuf,
    list_stack: Vec<ListType>,
    /// syntect theme name, e.g. "base16-ocean.dark" or "InspiredGitHub"
    syntax_theme: String,
    /// Monospace font name forwarded to code blocks
    mono_font: String,
    /// Open inline-HTML tag stack so close-tags emit the right Typst suffix.
    html_inline_stack: Vec<InlineHtmlFrame>,
}

impl TypstRenderer {
    fn new(config: &RenderConfig) -> Self {
        let asset_root = config
            .input_path
            .as_ref()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let cache_dir = asset_root.join(".md-to-pdf-cache");
        let _ = fs::create_dir_all(&cache_dir);
        Self {
            asset_root,
            cache_dir,
            list_stack: Vec::new(),
            syntax_theme: config.syntax_theme.clone(),
            mono_font: config.fonts.monospace.clone(),
            html_inline_stack: Vec::new(),
        }
    }

    fn render_children<'a>(&mut self, node: &'a AstNode<'a>) -> Result<String> {
        let mut out = String::new();
        for child in node.children() {
            out.push_str(&self.render_node(child)?);
        }
        Ok(out)
    }

    fn render_node<'a>(&mut self, node: &'a AstNode<'a>) -> Result<String> {
        match &node.data.borrow().value {
            NodeValue::Document => self.render_children(node),
            NodeValue::FrontMatter(_) => Ok(String::new()),
            NodeValue::Paragraph => {
                let inline = self.render_inline_children(node)?;
                Ok(format!("{}\n\n", inline.trim()))
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                let body = self.render_inline_children(node)?;
                Ok(format!(
                    "{} {}\n\n",
                    "=".repeat(*level as usize),
                    body.trim()
                ))
            }
            NodeValue::Text(text) => Ok(escape_typst_text(text)),
            NodeValue::SoftBreak | NodeValue::LineBreak => Ok(" ".to_string()),
            NodeValue::Code(code) => Ok(format!("`{}`", escape_typst_code(&code.literal))),
            NodeValue::Strong => Ok(format!("#strong[{}]", self.render_inline_children(node)?)),
            NodeValue::Emph => Ok(format!("#emph[{}]", self.render_inline_children(node)?)),
            NodeValue::Strikethrough => {
                Ok(format!("#strike[{}]", self.render_inline_children(node)?))
            }
            NodeValue::Link(NodeLink { url, .. }) => {
                let label = self.render_inline_children(node)?;
                Ok(format!(
                    "#link({}, [{}])",
                    typst_quoted_string(url),
                    label.trim()
                ))
            }
            NodeValue::Image(NodeLink { url, .. }) => self.render_image(node, url),
            NodeValue::BlockQuote => {
                let body = self.render_children(node)?;
                Ok(format!(
                    "#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[\n{}\n]\n\n",
                    body.trim()
                ))
            }
            NodeValue::List(list) => {
                self.list_stack.push(list.list_type);
                let body = self.render_children(node)?;
                self.list_stack.pop();
                Ok(format!("{}\n", body))
            }
            NodeValue::Item(..) => self.render_list_item(node),
            NodeValue::CodeBlock(block) => Ok(self.render_code_block(block)),
            NodeValue::TaskItem(_) => Ok(String::new()),
            NodeValue::Table(..) => self.render_table(node),
            NodeValue::ThematicBreak => Ok("#line(length: 100%)\n\n".to_string()),
            NodeValue::Math(math) => {
                // Convert LaTeX math to Typst native math syntax
                let converted = latex_to_typst(math.literal.trim());
                if math.display_math {
                    // Block math: Typst `$ expr $` on its own paragraph = display equation
                    Ok(format!("$ {} $\n\n", converted))
                } else {
                    // Inline math: `$expr$` — no surrounding spaces
                    Ok(format!("${}$", converted))
                }
            }
            NodeValue::HtmlInline(html) => Ok(self.handle_html_inline(html)),
            NodeValue::HtmlBlock(html) => {
                let rendered = block_html_to_typst(&html.literal);
                if rendered.is_empty() {
                    Ok(String::new())
                } else {
                    Ok(rendered)
                }
            }
            _ => self.render_children(node),
        }
    }

    fn render_inline_children<'a>(&mut self, node: &'a AstNode<'a>) -> Result<String> {
        let mut out = String::new();
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::Paragraph => out.push_str(&self.render_inline_children(child)?),
                _ => out.push_str(&self.render_node(child)?),
            }
        }
        Ok(out)
    }

    /// Handle a raw inline HTML tag fragment.
    ///
    /// Comrak emits each tag (`<b>`, `bold text`, `</b>`) as separate AST nodes.
    /// We maintain `html_inline_stack` across calls inside a single paragraph so
    /// opening tags push a Typst prefix and closing tags pop+emit the matching suffix.
    fn handle_html_inline(&mut self, html: &str) -> String {
        match parse_inline_tag(html) {
            // Void elements (self-closing by definition)
            InlineTag::SelfClose { ref name } | InlineTag::Open { ref name, .. }
                if is_void_inline(name) =>
            {
                match name.as_str() {
                    "br" | "wbr" => "\\\n".to_string(),
                    "hr" => "#line(length: 100%)\n\n".to_string(),
                    _ => String::new(),
                }
            }
            // Paired open tag → emit prefix, push suffix onto stack
            InlineTag::Open { name, attrs } => {
                let (prefix, suffix) = inline_tag_to_typst(&name, &attrs);
                self.html_inline_stack.push(InlineHtmlFrame { tag: name, suffix });
                prefix
            }
            // Close tag → pop matching frame, emit suffix
            InlineTag::Close { name } => {
                if let Some(pos) = self.html_inline_stack.iter().rposition(|f| f.tag == name) {
                    let mut out = String::new();
                    let inner: Vec<_> = self.html_inline_stack.drain(pos + 1..).collect();
                    for f in inner.into_iter().rev() { out.push_str(&f.suffix); }
                    let frame = self.html_inline_stack.remove(pos);
                    out.push_str(&frame.suffix);
                    out
                } else {
                    String::new()
                }
            }
            // Non-void self-closing: emit as open+close with empty content
            InlineTag::SelfClose { name } => {
                let (prefix, suffix) = inline_tag_to_typst(&name, &[]);
                format!("{}{}", prefix, suffix)
            }
            // Unknown / not a real tag: escape and pass through
            InlineTag::Unknown => crate::html::escape_typst_text(html),
        }
    }

    fn render_list_item<'a>(&mut self, node: &'a AstNode<'a>) -> Result<String> {
        let marker = match self.list_stack.last().copied().unwrap_or(ListType::Bullet) {
            ListType::Bullet => "- ",
            ListType::Ordered => "+ ",
        };

        let mut parts = Vec::new();
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::TaskItem(checked) => {
                    let box_text = if checked.is_some() { "☒" } else { "☐" };
                    parts.push(format!("{} ", box_text));
                }
                _ => parts.push(self.render_node(child)?),
            }
        }

        Ok(format!("{}{}\n", marker, parts.join("").trim()))
    }

    fn render_code_block(&self, block: &NodeCodeBlock) -> String {
        let lang = block.info.split_whitespace().next().unwrap_or_default();

        // ```math ... ``` fenced blocks are display math, not code.
        if lang == "math" {
            let converted = latex_to_typst(block.literal.trim());
            return format!("$ {} $\n\n", converted);
        }

        // Delegate to the syntect-based highlighter which returns a complete
        // self-contained Typst `#block(…)` expression with per-token colours.
        highlight_code_to_typst(&block.literal, lang, &self.mono_font, &self.syntax_theme)
    }

    fn render_image<'a>(&mut self, node: &'a AstNode<'a>, url: &str) -> Result<String> {
        let alt = self.render_inline_children(node)?.trim().to_string();
        let resolved = self.resolve_image_path(url)?;
        let caption = if alt.is_empty() {
            String::new()
        } else {
            format!(", caption: [{}]", escape_typst_text(&alt))
        };
        Ok(format!(
            "#figure(image({}, width: 100%){})\n\n",
            typst_quoted_string(&resolved),
            caption
        ))
    }

    fn resolve_image_path(&self, url: &str) -> Result<String> {
        if url.starts_with("http://") || url.starts_with("https://") {
            let hashed = stable_name(url);
            let ext = guess_remote_extension(url);
            let file_name = format!("remote-image-{}.{}", hashed, ext);
            let target = self.cache_dir.join(file_name);
            if !target.exists() {
                let response = ureq::get(url)
                    .call()
                    .with_context(|| format!("Failed to download remote image: {url}"))?;
                let mut bytes = Vec::new();
                response
                    .into_reader()
                    .read_to_end(&mut bytes)
                    .with_context(|| format!("Failed to read remote image response: {url}"))?;
                fs::write(&target, bytes).with_context(|| {
                    format!("Failed to cache remote image: {}", target.display())
                })?;
            }
            return Ok(target.to_string_lossy().to_string());
        }

        let path = Path::new(url);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.asset_root.join(path)
        };
        Ok(resolved.to_string_lossy().to_string())
    }

    fn render_table<'a>(&mut self, node: &'a AstNode<'a>) -> Result<String> {
        let mut rows = Vec::new();
        for row in node.children() {
            let mut cells = Vec::new();
            for cell in row.children() {
                let text = self.render_inline_children(cell)?;
                cells.push(format!("[{}]", text.trim()));
            }
            if !cells.is_empty() {
                rows.push(cells);
            }
        }

        if rows.is_empty() {
            return Ok(String::new());
        }

        let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);
        let mut flat = Vec::new();
        for row in rows {
            for cell in row {
                flat.push(cell);
            }
        }

        Ok(format!(
            "#table(\n  columns: {},\n  stroke: luma(180),\n  inset: 6pt,\n  align: left,\n  {}\n)\n\n",
            column_count,
            flat.join(",\n  ")
        ))
    }
}

fn extract_toc(markdown: &str) -> Vec<TocEntry> {
    markdown
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let hashes = trimmed.chars().take_while(|c| *c == '#').count();
            if (1..=6).contains(&hashes) && trimmed.chars().nth(hashes) == Some(' ') {
                Some(TocEntry {
                    level: hashes as u8,
                    title: trimmed[hashes + 1..].trim().to_string(),
                    page_number: 0,
                })
            } else {
                None
            }
        })
        .collect()
}

fn generate_typst_toc(markdown: &str) -> String {
    let entries = extract_toc(markdown);
    if entries.is_empty() {
        return String::new();
    }

    let mut lines = String::from("#pagebreak()\n= Table of Contents\n");
    for entry in entries {
        let indent = "  ".repeat(entry.level.saturating_sub(1) as usize);
        lines.push_str(&format!("{indent}- {}\n", escape_typst_text(&entry.title)));
    }
    lines.push_str("#pagebreak()\n");
    lines
}

// ─────────────────────────────────────────────────────────────────────────────
// LaTeX → Typst math translation
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a LaTeX math expression to Typst math syntax.
///
/// Typst math is similar to LaTeX but uses function-call notation for most
/// constructs.  This covers the common subset that appears in Markdown.
/// Unknown commands are passed through unchanged (with leading `\`) so they
/// fail visibly rather than producing silent wrong output.
fn latex_to_typst(latex: &str) -> String {
    let chars: Vec<char> = latex.chars().collect();
    let mut out = String::with_capacity(latex.len() + 16);
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '\\' => {
                i += 1;
                if i >= chars.len() { out.push('\\'); continue; }
                match chars[i] {
                    // Line break inside matrices / aligned envs
                    '\\' => { out.push_str("\\ "); i += 1; continue; }
                    // Spacing commands → single space
                    ',' | ':' | ';' | '!' | ' ' => { out.push(' '); i += 1; continue; }
                    // Escaped braces → literal brace
                    '{' => { out.push('{'); i += 1; continue; }
                    '}' => { out.push('}'); i += 1; continue; }
                    _ => {}
                }
                // Collect alphabetic command name
                let cmd_start = i;
                while i < chars.len() && chars[i].is_ascii_alphabetic() { i += 1; }
                let cmd: String = chars[cmd_start..i].iter().collect();
                // Skip optional trailing space after command name
                if i < chars.len() && chars[i] == ' ' && !cmd.is_empty() { i += 1; }

                // Insert a leading space if the output buffer ends with an
                // identifier char and the translated command will start with one
                // — prevents merging like `i` + `pi` → `ipi`.
                let prev_is_ident = out.chars().last()
                    .map(|c| c.is_alphanumeric() || c == '\'')
                    .unwrap_or(false);

                let translated = math_cmd(&cmd, &chars, &mut i);

                if prev_is_ident {
                    if let Some(first) = translated.chars().next() {
                        if first.is_alphanumeric() || first == '"' {
                            out.push(' ');
                        }
                    }
                }
                out.push_str(&translated);

                // Insert a trailing space when the translation ends with an
                // identifier char and the next input char is also alphanumeric
                // — prevents e.g. `gt.eq0` instead of `gt.eq 0`.
                if let (Some(last), Some(&next)) = (translated.chars().last(), chars.get(i)) {
                    if (last.is_alphanumeric() || last == '.')
                        && (next.is_alphanumeric() || next == '_')
                    {
                        out.push(' ');
                    }
                }
            }
            // Braces pass through as Typst grouping.
            // `math_cmd`'s `consume` helper already consumed the braces that
            // belonged to function arguments.
            '{' => { out.push('{'); i += 1; }
            '}' => { out.push('}'); i += 1; }
            c   => { out.push(c);  i += 1; }
        }
    }
    out
}

/// Translate a single LaTeX command name to its Typst equivalent.
/// `chars` and `i` allow consuming brace-delimited arguments.
fn math_cmd(cmd: &str, chars: &[char], i: &mut usize) -> String {
    // Consume one brace-group {…} or single token, returning it translated.
    let consume = |chars: &[char], i: &mut usize| -> String {
        while *i < chars.len() && chars[*i] == ' ' { *i += 1; }
        if *i >= chars.len() { return String::new(); }
        if chars[*i] == '{' {
            *i += 1;
            let mut depth = 1usize;
            let mut inner = String::new();
            while *i < chars.len() {
                match chars[*i] {
                    '{' => { depth += 1; inner.push('{'); *i += 1; }
                    '}' => {
                        depth -= 1;
                        if depth == 0 { *i += 1; break; }
                        inner.push('}'); *i += 1;
                    }
                    c => { inner.push(c); *i += 1; }
                }
            }
            latex_to_typst(&inner)
        } else {
            let c = chars[*i]; *i += 1;
            if c == '\\' {
                let start = *i;
                while *i < chars.len() && chars[*i].is_ascii_alphabetic() { *i += 1; }
                let sub: String = chars[start..*i].iter().collect();
                math_cmd(&sub, chars, i)
            } else { c.to_string() }
        }
    };

    match cmd {
        // ── fractions ─────────────────────────────────────────────────
        "frac"|"dfrac"|"tfrac"|"cfrac" => {
            let n = consume(chars, i); let d = consume(chars, i);
            format!("frac({}, {})", n, d)
        }
        "binom" => {
            let n = consume(chars, i); let k = consume(chars, i);
            format!("binom({}, {})", n, k)
        }

        // ── roots ─────────────────────────────────────────────────────
        "sqrt" => {
            while *i < chars.len() && chars[*i] == ' ' { *i += 1; }
            if *i < chars.len() && chars[*i] == '[' {
                *i += 1;
                let mut n = String::new();
                while *i < chars.len() && chars[*i] != ']' { n.push(chars[*i]); *i += 1; }
                if *i < chars.len() { *i += 1; }
                format!("root({}, {})", latex_to_typst(&n), consume(chars, i))
            } else {
                format!("sqrt({})", consume(chars, i))
            }
        }

        // ── text in math ──────────────────────────────────────────────
        "text"|"mathrm"|"mathit"|"mathsf"|"mathtt"|"operatorname" =>
            format!("\"{}\"", consume(chars, i).replace('"', "\\\"")),
        "mathbf"|"textbf"|"boldsymbol"|"bm" =>
            format!("bold({})", consume(chars, i)),
        "textrm"|"textnormal" =>
            format!("\"{}\"", consume(chars, i).replace('"', "\\\"")),
        // Blackboard bold: \mathbb{R} → RR, \mathbb{Z} → ZZ, etc.
        "mathbb" => {
            let inner = consume(chars, i);
            match inner.trim() {
                "R" => "RR".into(),
                "Z" => "ZZ".into(),
                "N" => "NN".into(),
                "Q" => "QQ".into(),
                "C" => "CC".into(),
                "H" => "HH".into(),
                other => format!("bb({})", other),
            }
        }

        // ── accents ───────────────────────────────────────────────────
        "hat"|"widehat"        => format!("hat({})",         consume(chars, i)),
        "tilde"|"widetilde"    => format!("tilde({})",       consume(chars, i)),
        "bar"|"overline"       => format!("overline({})",    consume(chars, i)),
        "underline"            => format!("underline({})",   consume(chars, i)),
        "vec"                  => format!("arrow({})",       consume(chars, i)),
        "dot"                  => format!("dot({})",         consume(chars, i)),
        "ddot"                 => format!("dot.double({})",  consume(chars, i)),
        "underbrace"           => format!("underbrace({})",  consume(chars, i)),
        "overbrace"            => format!("overbrace({})",   consume(chars, i)),

        // ── trig / log functions ──────────────────────────────────────
        "sin"=>"sin".into(), "cos"=>"cos".into(), "tan"=>"tan".into(),
        "sec"=>"sec".into(), "csc"=>"csc".into(), "cot"=>"cot".into(),
        "arcsin"=>"arcsin".into(), "arccos"=>"arccos".into(), "arctan"=>"arctan".into(),
        "sinh"=>"sinh".into(), "cosh"=>"cosh".into(), "tanh"=>"tanh".into(),
        "ln"=>"ln".into(), "log"=>"log".into(), "exp"=>"exp".into(),
        "lim"=>"lim".into(), "limsup"=>"limsup".into(), "liminf"=>"liminf".into(),
        "sup"=>"sup".into(), "inf"=>"inf".into(),
        "max"=>"max".into(), "min"=>"min".into(),
        "arg"=>"arg".into(), "det"=>"det".into(), "dim"=>"dim".into(),
        "gcd"=>"gcd".into(), "hom"=>"hom".into(), "ker"=>"ker".into(),

        // ── sums / integrals ──────────────────────────────────────────
        "sum"=>"sum".into(), "prod"=>"prod".into(),
        "int"=>"integral".into(), "iint"=>"integral.double".into(),
        "iiint"=>"integral.triple".into(), "oint"=>"integral.cont".into(),
        "bigcup"=>"union.big".into(), "bigcap"=>"sect.big".into(),

        // ── Greek (lowercase) ─────────────────────────────────────────
        "alpha"=>"alpha".into(), "beta"=>"beta".into(), "gamma"=>"gamma".into(),
        "delta"=>"delta".into(), "epsilon"=>"epsilon".into(),
        "varepsilon"=>"epsilon.alt".into(), "zeta"=>"zeta".into(),
        "eta"=>"eta".into(), "theta"=>"theta".into(), "vartheta"=>"theta.alt".into(),
        "iota"=>"iota".into(), "kappa"=>"kappa".into(), "lambda"=>"lambda".into(),
        "mu"=>"mu".into(), "nu"=>"nu".into(), "xi"=>"xi".into(),
        "pi"=>"pi".into(), "varpi"=>"pi.alt".into(),
        "rho"=>"rho".into(), "varrho"=>"rho.alt".into(),
        "sigma"=>"sigma".into(), "varsigma"=>"sigma.alt".into(),
        "tau"=>"tau".into(), "upsilon"=>"upsilon".into(),
        "phi"=>"phi.alt".into(), "varphi"=>"phi".into(),
        "chi"=>"chi".into(), "psi"=>"psi".into(), "omega"=>"omega".into(),
        // Greek (uppercase)
        "Gamma"=>"Gamma".into(), "Delta"=>"Delta".into(), "Theta"=>"Theta".into(),
        "Lambda"=>"Lambda".into(), "Xi"=>"Xi".into(), "Pi"=>"Pi".into(),
        "Sigma"=>"Sigma".into(), "Upsilon"=>"Upsilon".into(),
        "Phi"=>"Phi".into(), "Psi"=>"Psi".into(), "Omega"=>"Omega".into(),

        // ── binary operators / relations ──────────────────────────────
        "cdot"=>"dot.c".into(), "cdots"=>"dots.c".into(),
        "ldots"|"dots"=>"dots".into(), "vdots"=>"dots.v".into(), "ddots"=>"dots.down".into(),
        "times"=>"times".into(), "div"=>"div".into(),
        "pm"=>"plus.minus".into(), "mp"=>"minus.plus".into(),
        "leq"|"le"=>"lt.eq".into(), "geq"|"ge"=>"gt.eq".into(),
        "neq"|"ne"=>"eq.not".into(), "approx"=>"approx".into(),
        "sim"=>"tilde.op".into(), "simeq"=>"tilde.eq".into(),
        "cong"=>"tilde.equiv".into(), "equiv"=>"equiv".into(),
        "propto"=>"prop".into(), "ll"=>"lt.double".into(), "gg"=>"gt.double".into(),
        "in"=>"in".into(), "notin"=>"in.not".into(),
        "subset"=>"subset".into(), "subseteq"=>"subset.eq".into(),
        "supset"=>"supset".into(), "supseteq"=>"supset.eq".into(),
        "cup"=>"union".into(), "cap"=>"sect".into(),
        "setminus"=>"without".into(), "emptyset"|"varnothing"=>"nothing".into(),
        "forall"=>"forall".into(), "exists"=>"exists".into(), "nexists"=>"exists.not".into(),
        "neg"|"lnot"=>"not".into(), "land"|"wedge"=>"and".into(), "lor"|"vee"=>"or".into(),
        "oplus"=>"plus.circle".into(), "otimes"=>"times.circle".into(),
        "circ"=>"circle.small".into(), "bullet"=>"bullet".into(),

        // ── arrows ────────────────────────────────────────────────────
        "to"|"rightarrow"=>"->".into(), "leftarrow"=>"<-".into(),
        "Rightarrow"=>"=>".into(), "Leftarrow"=>"<=".into(),
        "leftrightarrow"=>"<->".into(), "Leftrightarrow"=>"<=>".into(),
        "mapsto"=>"|->".into(), "uparrow"=>"arrow.t".into(), "downarrow"=>"arrow.b".into(),
        "updownarrow"=>"arrow.t.b".into(),
        "longrightarrow"=>"-->".into(), "longleftarrow"=>"<--".into(),

        // ── misc symbols ──────────────────────────────────────────────
        "partial"=>"diff".into(), "nabla"=>"nabla".into(), "infty"=>"oo".into(),
        "hbar"=>"planck.reduce".into(), "ell"=>"ell".into(),
        "Re"=>"Re".into(), "Im"=>"Im".into(), "aleph"=>"aleph".into(),
        "prime"=>"'".into(), "dagger"=>"dagger".into(), "ddagger"=>"dagger.double".into(),
        "star"=>"star".into(), "ast"=>"ast".into(),

        // ── delimiters (auto-sized in Typst) ──────────────────────────
        "left"|"right" => {
            while *i < chars.len() && chars[*i] == ' ' { *i += 1; }
            if *i < chars.len() {
                let d = chars[*i]; *i += 1;
                if d == '.' { String::new() } else { d.to_string() }
            } else { String::new() }
        }
        "langle"=>"angle.l".into(), "rangle"=>"angle.r".into(),
        "lfloor"=>"floor.l".into(), "rfloor"=>"floor.r".into(),
        "lceil"=>"ceil.l".into(),   "rceil"=>"ceil.r".into(),
        "lVert"|"rVert"=>"||".into(), "lvert"|"rvert"=>"|".into(),

        // ── environments ──────────────────────────────────────────────
        "begin" => { let env = consume(chars, i); math_env(&env, chars, i) }
        "end"   => { consume(chars, i); String::new() }

        // ── layout / spacing ──────────────────────────────────────────
        "quad"=>"quad".into(), "qquad"=>"wide".into(),
        "hspace"|"vspace" => { consume(chars, i); " ".into() }
        "displaystyle"|"textstyle"|"scriptstyle"|"scriptscriptstyle" => String::new(),
        "limits"|"nolimits"|"nonumber"|"notag" => String::new(),
        "label"|"tag" => { consume(chars, i); String::new() }

        // ── unknown: pass through so it fails visibly ──────────────────
        other => format!("\\{}", other),
    }
}

/// Handle `\begin{env}...\end{env}` environments.
fn math_env(env: &str, chars: &[char], i: &mut usize) -> String {
    let begin_m = format!("\\begin{{{}}}", env);
    let end_m   = format!("\\end{{{}}}", env);
    let remaining: String = chars[*i..].iter().collect();
    let mut inner = String::new();
    let mut depth = 1usize;
    let mut j = 0usize;

    while j < remaining.len() {
        if remaining[j..].starts_with(begin_m.as_str()) {
            depth += 1;
            inner.push_str(&begin_m);
            j += begin_m.len();
        } else if remaining[j..].starts_with(end_m.as_str()) {
            depth -= 1;
            if depth == 0 { j += end_m.len(); break; }
            inner.push_str(&end_m);
            j += end_m.len();
        } else {
            let ch = remaining[j..].chars().next().unwrap_or('\0');
            inner.push(ch);
            j += ch.len_utf8();
        }
    }
    *i += remaining[..j].chars().count();

    match env {
        "matrix"  => format!("mat(delim: #none, {})", math_matrix_body(&inner)),
        "pmatrix" => format!("mat({})",                math_matrix_body(&inner)),
        "bmatrix" => format!("mat(delim: \"[\", {})",  math_matrix_body(&inner)),
        "Bmatrix" => format!("mat(delim: \"{{\", {})", math_matrix_body(&inner)),
        "vmatrix" => format!("mat(delim: \"|\", {})",  math_matrix_body(&inner)),
        "Vmatrix" => format!("mat(delim: \"||\", {})", math_matrix_body(&inner)),
        "cases"   => math_cases(&inner),
        "align"|"align*"|"aligned" => {
            inner.split("\\\\")
                 .map(|r| latex_to_typst(r.trim()))
                 .collect::<Vec<_>>()
                 .join(" \\ ")
        }
        "equation"|"equation*" => latex_to_typst(inner.trim()),
        _ => latex_to_typst(inner.trim()),
    }
}

/// Translate `cases` environment body.
/// LaTeX: `expr & \text{if} cond \\` rows
/// Typst: `cases(expr &"if" cond, ...)`
fn math_cases(inner: &str) -> String {
    let rows: Vec<String> = inner.split("\\\\")
        .filter(|r| !r.trim().is_empty())
        .map(|row| {
            let parts: Vec<&str> = row.splitn(2, '&').collect();
            if parts.len() == 2 {
                let val = latex_to_typst(parts[0].trim());
                let cond_raw = parts[1].trim();
                let stripped = strip_text_if(cond_raw);
                let cond = latex_to_typst(stripped.trim());
                if cond.is_empty() || cond == "\"otherwise\"" || cond == "\"else\"" {
                    format!("{} &\"otherwise\"", val)
                } else {
                    format!("{} &\"if\" {}", val, cond)
                }
            } else {
                latex_to_typst(row.trim())
            }
        })
        .collect();
    format!("cases({})", rows.join(", "))
}

/// Strip leading `\text{if}`, `\text{otherwise}`, etc. from a cases condition
/// so the surrounding `"if"` keyword in Typst doesn't duplicate it.
fn strip_text_if(s: &str) -> &str {
    let s = s.trim();
    for p in &[r"\text{if }", r"\text{if}", r"\text{ if }", r"\text{ if}",
               r"\text{otherwise}", r"\text{else}"] {
        if s.starts_with(p) { return &s[p.len()..]; }
    }
    s
}

/// Convert a matrix body (rows: `\\`, cols: `&`) to Typst `mat(...)` args.
/// Output: rows separated by `;`, columns by `,`.
fn math_matrix_body(body: &str) -> String {
    body.split("\\\\")
        .filter(|r| !r.trim().is_empty())
        .map(|row| {
            row.split('&')
               .map(|c| latex_to_typst(c.trim()))
               .collect::<Vec<_>>()
               .join(", ")
        })
        .collect::<Vec<_>>()
        .join("; ")
}

// ─────────────────────────────────────────────────────────────────────────────

fn escape_typst_text(s: &str) -> String {
    s.replace('\n', " ")
        .replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('"', "\\\"")
}

fn escape_typst_code(s: &str) -> String {
    s.replace('`', "\\`")
}

fn typst_quoted_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn stable_name(s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn guess_remote_extension(url: &str) -> &'static str {
    let lower = url.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "jpg"
    } else if lower.ends_with(".gif") {
        "gif"
    } else if lower.ends_with(".webp") {
        "webp"
    } else if lower.ends_with(".svg") {
        "svg"
    } else {
        "img"
    }
}

fn format_typst_errors(errors: &[typst::diag::SourceDiagnostic]) -> String {
    errors
        .iter()
        .map(|error| error.message.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

struct TypstWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<typst_kit::fonts::FontSlot>,
    main: FileId,
    sources: HashMap<FileId, Source>,
    root: PathBuf,
}

impl TypstWorld {
    fn new(main_source: String) -> Result<Self> {
        let root = std::env::current_dir()?;
        let mut searcher = FontSearcher::new();
        let Fonts { book, fonts } = searcher.search();
        let main = FileId::new(None, VirtualPath::new("/main.typ"));
        let mut sources = HashMap::new();
        sources.insert(main, Source::new(main, main_source));

        Ok(Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts,
            main,
            sources,
            root,
        })
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> typst::diag::FileResult<Source> {
        if let Some(source) = self.sources.get(&id) {
            return Ok(source.clone());
        }
        let path = id.vpath().resolve(&self.root).ok_or_else(|| {
            typst::diag::FileError::NotFound(id.vpath().as_rootless_path().to_path_buf())
        })?;
        let text = fs::read_to_string(&path).map_err(|_| {
            typst::diag::FileError::NotFound(id.vpath().as_rootless_path().to_path_buf())
        })?;
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> typst::diag::FileResult<Bytes> {
        let path = id.vpath().resolve(&self.root).ok_or_else(|| {
            typst::diag::FileError::NotFound(id.vpath().as_rootless_path().to_path_buf())
        })?;
        let bytes = fs::read(&path).map_err(|_| {
            typst::diag::FileError::NotFound(id.vpath().as_rootless_path().to_path_buf())
        })?;
        Ok(Bytes::new(bytes))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).and_then(|slot| slot.get())
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}
