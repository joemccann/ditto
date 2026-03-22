use anyhow::{Context, Result};
use comrak::{
    Arena, Options,
    nodes::{
        AlertType, AstNode, ListType, NodeAlert, NodeCodeBlock, NodeHeading, NodeLink, NodeTable,
        NodeValue, TableAlignment,
    },
    parse_document,
};
use std::collections::HashMap;

use crate::highlighter::highlight_code_to_typst;
use crate::html::{
    InlineTag, block_html_to_typst, inline_tag_to_typst, is_void_inline, parse_inline_tag,
};
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
    pub syntax_theme: String,
    /// Whether to emit a table of contents page.
    pub toc: bool,
    /// When true, the `toc` value was explicitly set by the user via a CLI flag
    /// and frontmatter cannot override it.
    pub toc_explicit: bool,
    /// Maximum heading depth to include in the TOC (1–6).
    pub toc_depth: u8,
    /// When true, remote images (http/https) are skipped rather than downloaded.
    pub no_remote_images: bool,
    /// Override for the cache directory used by remote-image downloads.
    pub cache_dir_override: Option<PathBuf>,
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
    // Write intermediate .typ for debugging when MDPDF_DEBUG=1.
    if std::env::var("MDPDF_DEBUG").as_deref() == Ok("1") {
        fs::write(output.with_extension("typ"), &typst_source).ok();
    }
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

/// Public entry point for tests — converts markdown to Typst source without
/// compiling to PDF.  Useful for snapshot testing and unit tests.
#[doc(hidden)]
pub fn markdown_to_typst_pub(markdown: &str, config: &RenderConfig) -> Result<String> {
    markdown_to_typst(markdown, config)
}

/// Test shim: expose `escape_typst_text` to integration tests.
/// Compiled only when running tests (unit or integration).
#[doc(hidden)]
pub fn escape_typst_text_pub(s: &str) -> String {
    escape_typst_text(s)
}

/// Test shim: expose `heading_label` to integration tests.
#[doc(hidden)]
pub fn heading_label_pub(title: &str) -> String {
    heading_label(title)
}

/// Test shim: expose `latex_to_typst` to integration tests.
#[doc(hidden)]
pub fn latex_to_typst_pub(latex: &str) -> String {
    latex_to_typst(latex)
}

/// Test shim: expose `typst_quoted_string` to integration tests.
#[doc(hidden)]
pub fn typst_quoted_string_pub(s: &str) -> String {
    typst_quoted_string(s)
}

/// Test shim: expose `generate_typst_toc` to integration tests (default title).
#[doc(hidden)]
pub fn generate_typst_toc_pub(depth: u8) -> String {
    generate_typst_toc(depth, "Table of Contents")
}

/// Test shim: expose `generate_typst_toc` with a custom title to integration tests.
#[doc(hidden)]
pub fn generate_typst_toc_titled_pub(depth: u8, title: &str) -> String {
    generate_typst_toc(depth, title)
}

/// Test shim: expose `extract_toc` to integration tests.
#[doc(hidden)]
pub fn extract_toc_pub(markdown: &str) -> Vec<TocEntry> {
    extract_toc(markdown)
}

/// Test shim: expose `stable_name` (hash helper) to integration tests.
#[doc(hidden)]
pub fn stable_name_pub(s: &str) -> String {
    stable_name(s)
}

fn markdown_to_typst(markdown: &str, config: &RenderConfig) -> Result<String> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &markdown_options());

    // Extract frontmatter overrides before rendering.
    // CLI flags always win; frontmatter only applies when the value was not
    // explicitly set via a CLI flag.
    let fm = parse_frontmatter(markdown);
    let toc_enabled = if config.toc_explicit {
        config.toc
    } else {
        // `no_toc: true` in frontmatter is a convenient alias for `toc: false`
        let fm_no_toc = fm.no_toc.unwrap_or(false);
        if fm_no_toc {
            false
        } else {
            fm.toc.unwrap_or(config.toc)
        }
    };
    let toc_depth = fm.toc_depth.unwrap_or(config.toc_depth).clamp(1, 6);
    let toc_title = fm.toc_title.as_deref().unwrap_or("Table of Contents");

    let mut renderer = TypstRenderer::new(config);
    // render_node(root) goes through the Document arm which appends footnotes.
    let body = renderer.render_node(root)?;
    let toc = if toc_enabled {
        generate_typst_toc(toc_depth, toc_title)
    } else {
        String::new()
    };

    Ok(format!(
        "#set page(width: {page_width}mm, height: {page_height}mm, margin: {margin}mm)\n\
#set text(font: ({font},), size: {font_size}pt)\n\
#show raw: set text(font: ({mono_font},), size: {code_size}pt)\n\
#show link: set text(fill: blue)\n\
{toc}\
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

// ─────────────────────────────────────────────────────────────────────────────
// Frontmatter parsing (YAML-subset: only `toc` and `toc_depth` keys)
// ─────────────────────────────────────────────────────────────────────────────

struct Frontmatter {
    toc: Option<bool>,
    /// Alias: `no_toc: true` is equivalent to `toc: false`.
    no_toc: Option<bool>,
    toc_depth: Option<u8>,
    /// Custom title for the TOC page (default: "Table of Contents").
    toc_title: Option<String>,
}

fn parse_frontmatter(markdown: &str) -> Frontmatter {
    let mut fm = Frontmatter {
        toc: None,
        no_toc: None,
        toc_depth: None,
        toc_title: None,
    };
    let text = markdown.trim_start();
    if !text.starts_with("---") {
        return fm;
    }
    // Find closing delimiter
    let after = &text[3..];
    let close = after.find("\n---").or_else(|| after.find("\n..."));
    let block = match close {
        Some(pos) => &after[..pos],
        None => return fm,
    };
    for line in block.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("toc_depth:") {
            let val = rest.trim();
            if let Ok(n) = val.parse::<u8>() {
                fm.toc_depth = Some(n);
            }
        } else if let Some(rest) = line.strip_prefix("toc_title:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            if !val.is_empty() {
                fm.toc_title = Some(val);
            }
        } else if let Some(rest) = line.strip_prefix("no_toc:") {
            let val = rest.trim().to_ascii_lowercase();
            fm.no_toc = Some(val == "true" || val == "yes" || val == "1");
        } else if let Some(rest) = line.strip_prefix("toc:") {
            let val = rest.trim().to_ascii_lowercase();
            fm.toc = Some(val == "true" || val == "yes" || val == "1");
        }
    }
    fm
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
    // GitHub Alerts: > [!NOTE], > [!TIP], etc.
    options.extension.alerts = true;
    // Superscript: ^text^
    options.extension.superscript = true;
    // Subscript: ~text~ (single tilde; double-tilde remains strikethrough)
    options.extension.subscript = true;
    // Underline: __text__ (double underscore)
    options.extension.underline = true;
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
    /// Stack of list types for the currently open lists (innermost last).
    list_stack: Vec<ListType>,
    /// The start counter for each open ordered list (parallel to list_stack).
    /// Entry is `None` for bullet lists.
    ordered_start_stack: Vec<Option<usize>>,
    /// Running item counter for each open ordered list.
    ordered_counter_stack: Vec<usize>,
    /// Nesting depth of the current list (0 = not inside a list).
    list_depth: usize,
    /// Stack of tight/loose flags for open lists (innermost last).
    /// `true` = tight (no blank lines between items); `false` = loose.
    list_tight_stack: Vec<bool>,
    /// syntect theme name, e.g. "base16-ocean.dark" or "InspiredGitHub"
    syntax_theme: String,
    /// Monospace font name forwarded to code blocks
    mono_font: String,
    /// Open inline-HTML tag stack so close-tags emit the right Typst suffix.
    html_inline_stack: Vec<InlineHtmlFrame>,
    /// When true, skip downloading remote images.
    no_remote_images: bool,
    /// Accumulated footnote definitions keyed by name: (name, body).
    /// We collect these when visiting FootnoteDefinition nodes.
    footnote_defs: HashMap<String, String>,
    /// Ordered list of footnote (name, document-ix) from FootnoteReference nodes.
    /// `ix` is comrak's per-footnote ordinal — first unique footnote = 1, etc.
    footnote_refs: Vec<(String, u32)>,
    /// Track heading labels already emitted so duplicates can be disambiguated.
    /// Maps base label → count of occurrences seen so far.
    heading_labels_seen: HashMap<String, usize>,
}

impl TypstRenderer {
    fn new(config: &RenderConfig) -> Self {
        let asset_root = config
            .input_path
            .as_ref()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        // Use the explicit cache-dir override, or default to .md-to-pdf-cache/ next to the input
        let cache_dir = config
            .cache_dir_override
            .clone()
            .unwrap_or_else(|| asset_root.join(".md-to-pdf-cache"));
        let _ = fs::create_dir_all(&cache_dir);
        Self {
            asset_root,
            cache_dir,
            list_stack: Vec::new(),
            ordered_start_stack: Vec::new(),
            ordered_counter_stack: Vec::new(),
            list_depth: 0,
            list_tight_stack: Vec::new(),
            syntax_theme: config.syntax_theme.clone(),
            mono_font: config.fonts.monospace.clone(),
            html_inline_stack: Vec::new(),
            no_remote_images: config.no_remote_images,
            footnote_defs: HashMap::new(),
            footnote_refs: Vec::new(),
            heading_labels_seen: HashMap::new(),
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
            NodeValue::Document => {
                let body = self.render_children(node)?;
                // Append footnote section after document body if any were collected.
                if self.footnote_defs.is_empty() {
                    Ok(body)
                } else {
                    Ok(format!("{}\n{}", body, self.emit_footnote_section()))
                }
            }
            NodeValue::FrontMatter(_) => Ok(String::new()),
            NodeValue::Paragraph => {
                let inline = self.render_inline_children(node)?;
                Ok(format!("{}\n\n", inline.trim()))
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                let body = self.render_inline_children(node)?;
                let base_label = heading_label(body.trim());
                // Deduplicate labels: if the same base label has been used before,
                // append `-2`, `-3`, … so every heading gets a unique anchor.
                // This ensures `#outline()` click-targets resolve correctly even
                // when a document has repeated heading text (e.g. multiple "Overview"
                // sections).
                let count = self
                    .heading_labels_seen
                    .entry(base_label.clone())
                    .or_insert(0);
                *count += 1;
                let label = if *count == 1 {
                    base_label
                } else {
                    format!("{}-{}", base_label, count)
                };
                // Emit the heading followed by a Typst label so #outline()
                // can build clickable, page-numbered TOC entries automatically.
                Ok(format!(
                    "{} {} <{}>\n\n",
                    "=".repeat(*level as usize),
                    body.trim(),
                    label
                ))
            }
            NodeValue::Text(text) => Ok(escape_typst_text(text)),
            NodeValue::SoftBreak => Ok(" ".to_string()),
            NodeValue::LineBreak => Ok("\\\n".to_string()),
            NodeValue::Code(code) => Ok(format!("`{}`", escape_typst_code(&code.literal))),
            NodeValue::Strong => Ok(format!("#strong[{}]", self.render_inline_children(node)?)),
            NodeValue::Emph => Ok(format!("#emph[{}]", self.render_inline_children(node)?)),
            NodeValue::Strikethrough => {
                Ok(format!("#strike[{}]", self.render_inline_children(node)?))
            }
            // ── Inline extensions ──────────────────────────────────────────────
            NodeValue::Superscript => Ok(format!("#super[{}]", self.render_inline_children(node)?)),
            NodeValue::Subscript => Ok(format!("#sub[{}]", self.render_inline_children(node)?)),
            NodeValue::Underline => Ok(format!(
                "#underline[{}]",
                self.render_inline_children(node)?
            )),
            NodeValue::Link(NodeLink { url, .. }) => {
                let label = self.render_inline_children(node)?;
                let label_trimmed = label.trim();
                // For autolinks the label equals the URL — emit a compact #link(url).
                if label_trimmed == url.as_str() || label_trimmed == url.trim_end_matches('/') {
                    Ok(format!("#link({})", typst_quoted_string(url)))
                } else {
                    Ok(format!(
                        "#link({}, [{}])",
                        typst_quoted_string(url),
                        label_trimmed
                    ))
                }
            }
            NodeValue::Image(NodeLink { url, .. }) => self.render_image(node, url),
            NodeValue::BlockQuote => {
                let body = self.render_children(node)?;
                Ok(format!(
                    "#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[\n{}\n]\n\n",
                    body.trim()
                ))
            }
            // ── GitHub Alerts: > [!NOTE], > [!TIP], > [!IMPORTANT], > [!WARNING], > [!CAUTION]
            NodeValue::Alert(alert) => Ok(render_alert(alert, &self.render_children(node)?)),
            NodeValue::List(list) => {
                // Push list context: type + starting counter.
                let start = list.start;
                self.list_stack.push(list.list_type);
                self.ordered_start_stack
                    .push(if list.list_type == ListType::Ordered {
                        Some(start)
                    } else {
                        None
                    });
                self.ordered_counter_stack.push(start.saturating_sub(1));
                self.list_tight_stack.push(list.tight);
                self.list_depth += 1;
                let body = self.render_children(node)?;
                self.list_depth -= 1;
                let is_ordered = list.list_type == ListType::Ordered;
                let start_at = list.start;
                self.list_stack.pop();
                self.ordered_start_stack.pop();
                self.ordered_counter_stack.pop();
                self.list_tight_stack.pop();

                // For ordered lists that start at a value other than 1, emit
                // a scoped block with `set enum(start: N)` so Typst renders
                // the correct starting number without polluting outer scope.
                // Add extra blank line after a top-level list for readability.
                if is_ordered && start_at != 1 {
                    Ok(format!(
                        "#block[\n#set enum(start: {})\n{}]\n\n",
                        start_at, body
                    ))
                } else {
                    Ok(format!("{}\n", body))
                }
            }
            NodeValue::Item(item) => self.render_list_item(node, item.list_type),
            NodeValue::TaskItem(checked) => {
                // TaskItem IS the item node — render it with checkbox prefix.
                self.render_task_item(node, checked.is_some())
            }
            NodeValue::CodeBlock(block) => Ok(self.render_code_block(block)),
            NodeValue::Table(table) => self.render_table(node, table),
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

            // ── Footnotes ─────────────────────────────────────────────────────
            NodeValue::FootnoteDefinition(def) => {
                // Accumulate footnote body keyed by name; emitted at document end.
                let name = def.name.clone();
                let body = self.render_children(node)?;
                self.footnote_defs.insert(name, body.trim().to_string());
                Ok(String::new())
            }
            NodeValue::FootnoteReference(r) => {
                // `ix` is comrak's 1-based unique footnote ordinal (order of first
                // appearance in the document).  Record the (name, ix) pair so we
                // can sort the definition section by first-use order.
                let ix = r.ix;
                let name = r.name.clone();
                // Only record the first reference to each footnote for ordering.
                if !self.footnote_refs.iter().any(|(n, _)| n == &name) {
                    self.footnote_refs.push((name, ix));
                }
                Ok(format!("#super[{}]", ix))
            }

            // ── Definition / description lists ────────────────────────────────
            NodeValue::DescriptionList => {
                let body = self.render_children(node)?;
                Ok(format!("{}\n", body))
            }
            NodeValue::DescriptionItem(_) => {
                let body = self.render_children(node)?;
                Ok(body)
            }
            NodeValue::DescriptionTerm => {
                let term = self.render_inline_children(node)?;
                Ok(format!("#strong[{}]\\\n", term.trim()))
            }
            NodeValue::DescriptionDetails => {
                let details = self.render_children(node)?;
                // Indent like a list item — 4-space hang.
                Ok(format!("#pad(left: 1.5em)[{}]\n\n", details.trim()))
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
                self.html_inline_stack
                    .push(InlineHtmlFrame { tag: name, suffix });
                prefix
            }
            // Close tag → pop matching frame, emit suffix
            InlineTag::Close { name } => {
                if let Some(pos) = self.html_inline_stack.iter().rposition(|f| f.tag == name) {
                    let mut out = String::new();
                    let inner: Vec<_> = self.html_inline_stack.drain(pos + 1..).collect();
                    for f in inner.into_iter().rev() {
                        out.push_str(&f.suffix);
                    }
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

    fn render_list_item<'a>(
        &mut self,
        node: &'a AstNode<'a>,
        _list_type: ListType,
    ) -> Result<String> {
        // Determine Typst marker based on list type.
        let is_ordered = matches!(self.list_stack.last().copied(), Some(ListType::Ordered));
        let marker = if is_ordered { "+ " } else { "- " };
        // Advance ordered counter if applicable.
        if is_ordered && let Some(ctr) = self.ordered_counter_stack.last_mut() {
            *ctr += 1;
        }

        // Indentation: 2 spaces per nesting level (0-based: depth 1 = top-level = no indent).
        let indent = if self.list_depth > 1 {
            "  ".repeat(self.list_depth - 1)
        } else {
            String::new()
        };

        // Whether the enclosing list is loose (blank lines between items).
        // Only the outermost (depth-1) list controls spacing; nested tight
        // lists inside a loose parent do not get extra spacing.
        let is_loose = self
            .list_tight_stack
            .last()
            .copied()
            .map(|tight| !tight)
            .unwrap_or(false)
            && self.list_depth == 1;
        let spacing = if is_loose { "#v(0.5em)\n" } else { "" };

        // Collect children, distinguishing between the inline text (first paragraph)
        // and block children (nested lists, additional paragraphs).
        let children: Vec<_> = node.children().collect();
        if children.is_empty() {
            return Ok(format!("{}{}\n{}", indent, marker, spacing));
        }

        // Render the first child — if it's a Paragraph, flatten it inline.
        // Subsequent children (nested lists, block quotes, etc.) are rendered
        // on their own lines with proper indentation.
        let mut first_text = String::new();
        let mut rest_parts: Vec<String> = Vec::new();
        let mut first_done = false;

        for child in &children {
            if !first_done {
                let val = &child.data.borrow().value.clone();
                match val {
                    NodeValue::Paragraph => {
                        first_text = self.render_inline_children(child)?;
                        first_done = true;
                    }
                    _ => {
                        first_text = self.render_node(child)?;
                        first_text = first_text.trim_end().to_string();
                        first_done = true;
                    }
                }
            } else {
                let rendered = self.render_node(child)?;
                rest_parts.push(rendered);
            }
        }

        // Build output: marker + first text on first line, then any nested content.
        let first_trimmed = first_text.trim_end().to_string();
        if rest_parts.is_empty() {
            Ok(format!(
                "{}{}{}\n{}",
                indent,
                marker,
                first_trimmed.trim_start(),
                spacing,
            ))
        } else {
            // Nested content: trim extra newlines so it reads cleanly.
            let nested = rest_parts.join("").trim_end().to_string();
            Ok(format!(
                "{}{}{}\n{}\n{}",
                indent,
                marker,
                first_trimmed.trim_start(),
                nested,
                spacing,
            ))
        }
    }

    fn render_task_item<'a>(&mut self, node: &'a AstNode<'a>, checked: bool) -> Result<String> {
        // Indentation based on nesting depth.
        let indent = if self.list_depth > 1 {
            "  ".repeat(self.list_depth - 1)
        } else {
            String::new()
        };

        // Render checkbox as an inline box so it aligns with the text.
        // ☑ (U+2611) = checked ballot box, ☐ (U+2610) = unchecked ballot box.
        let box_char = if checked { "☑" } else { "☐" };
        let checkbox = format!("#box(width: 1em)[{}]", escape_typst_text(box_char));

        // Same two-phase rendering as render_list_item: first child is the
        // inline text (Paragraph), remaining children are nested lists / blocks.
        let children: Vec<_> = node.children().collect();
        if children.is_empty() {
            return Ok(format!("{}- {} \n", indent, checkbox));
        }

        let mut first_text = String::new();
        let mut rest_parts: Vec<String> = Vec::new();
        let mut first_done = false;

        for child in &children {
            if !first_done {
                let val = child.data.borrow().value.clone();
                match val {
                    NodeValue::Paragraph => {
                        first_text = self.render_inline_children(child)?;
                        first_done = true;
                    }
                    _ => {
                        first_text = self.render_node(child)?;
                        first_text = first_text.trim_end().to_string();
                        first_done = true;
                    }
                }
            } else {
                let rendered = self.render_node(child)?;
                rest_parts.push(rendered);
            }
        }

        let first_trimmed = first_text.trim_end().to_string();
        if rest_parts.is_empty() {
            Ok(format!(
                "{}- {} {}\n",
                indent,
                checkbox,
                first_trimmed.trim_start()
            ))
        } else {
            let nested = rest_parts.join("").trim_end().to_string();
            Ok(format!(
                "{}- {} {}\n{}\n",
                indent,
                checkbox,
                first_trimmed.trim_start(),
                nested
            ))
        }
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
        // Extract plain text for the alt/caption — not Typst markup, because
        // format_image_typst will call escape_typst_text on it.
        let alt = self.extract_alt_text(node);
        match self.resolve_image(url) {
            Ok(info) => Ok(format_image_typst(&info, &alt)),
            Err(e) => {
                eprintln!("Warning: skipping image {url}: {e}");
                Ok(missing_image_fallback(url, &alt))
            }
        }
    }

    /// Recursively collect plain text from AST children without any Typst
    /// markup.  Used for image alt text / captions so that format_image_typst
    /// can safely call escape_typst_text on the result without double-escaping.
    fn extract_alt_text<'a>(&self, node: &'a AstNode<'a>) -> String {
        let mut out = String::new();
        for child in node.children() {
            let v = child.data.borrow();
            match &v.value {
                NodeValue::Text(t) => out.push_str(t),
                NodeValue::Code(c) => out.push_str(&c.literal),
                NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
                NodeValue::Math(m) => out.push_str(&m.literal),
                // For styled nodes (bold, italic, etc.) recurse into children
                _ => out.push_str(&self.extract_alt_text(child)),
            }
        }
        out
    }

    /// Resolve an image URL/path to an [`ImageInfo`], downloading and caching
    /// remote images as needed.
    fn resolve_image(&self, url: &str) -> Result<ImageInfo> {
        if url.starts_with("http://") || url.starts_with("https://") {
            if self.no_remote_images {
                anyhow::bail!("remote images disabled (--no-remote-images): skipping {url}");
            }
            return self.resolve_remote_image(url);
        }

        // Data URIs: data:image/png;base64,...
        if url.starts_with("data:") {
            return self.resolve_data_uri(url);
        }

        let path = Path::new(url);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.asset_root.join(path)
        };

        if !resolved.exists() {
            anyhow::bail!("local image not found: {}", resolved.display());
        }

        let is_svg = resolved
            .extension()
            .map(|e| e.eq_ignore_ascii_case("svg"))
            .unwrap_or(false);

        Ok(ImageInfo {
            path: resolved,
            is_svg,
            // Width/height unknown for local paths without parsing
            natural_width: None,
            natural_height: None,
        })
    }

    fn resolve_remote_image(&self, url: &str) -> Result<ImageInfo> {
        let hashed = stable_name(url);

        // Check if we have a cached entry with a known extension already.
        // We store a tiny metadata sidecar ("<hash>.meta") with:
        //   etag=<value>\nlast_modified=<value>\next=<ext>
        // so we can do conditional requests on re-runs.
        let meta_path = self.cache_dir.join(format!("remote-image-{}.meta", hashed));
        let cached_ext = read_cache_meta(&meta_path)
            .map(|m| m.ext)
            .unwrap_or_default();

        // If we already have a cached file with a real extension, use it.
        if !cached_ext.is_empty() {
            let candidate = self
                .cache_dir
                .join(format!("remote-image-{}.{}", hashed, cached_ext));
            if candidate.exists() {
                // Attempt a conditional GET to validate freshness; fall back
                // to the cached copy on any network error.
                let meta = read_cache_meta(&meta_path);
                match self.conditional_fetch(url, meta.as_ref(), &candidate, hashed.as_str()) {
                    Ok(Some(new_info)) => return Ok(new_info),
                    Ok(None) => {
                        // 304 Not Modified — cached copy is still valid
                        let is_svg = cached_ext == "svg";
                        return Ok(ImageInfo {
                            path: candidate,
                            is_svg,
                            natural_width: None,
                            natural_height: None,
                        });
                    }
                    Err(_) => {
                        // Network failure — use cached copy
                        let is_svg = cached_ext == "svg";
                        return Ok(ImageInfo {
                            path: candidate,
                            is_svg,
                            natural_width: None,
                            natural_height: None,
                        });
                    }
                }
            }
        }

        // No cache hit — do a fresh download.
        self.fetch_and_cache_remote(url, hashed.as_str())
    }

    /// Perform a conditional GET using ETag / Last-Modified from the cache
    /// metadata.  Returns:
    /// - `Ok(Some(info))` — the server returned fresh content (200), saved and ready.
    /// - `Ok(None)` — server returned 304 Not Modified; caller should use the cache.
    /// - `Err(_)` — any network or I/O error.
    fn conditional_fetch(
        &self,
        url: &str,
        meta: Option<&CacheMeta>,
        _existing_path: &Path,
        hashed: &str,
    ) -> Result<Option<ImageInfo>> {
        let mut req = ureq::get(url);

        let has_condition = if let Some(m) = meta {
            if !m.etag.is_empty() {
                req = req.set("If-None-Match", &m.etag);
                true
            } else if !m.last_modified.is_empty() {
                req = req.set("If-Modified-Since", &m.last_modified);
                true
            } else {
                false
            }
        } else {
            false
        };

        if !has_condition {
            // Nothing to conditionalize on — do a fresh download.
            return self.fetch_and_cache_remote(url, hashed).map(Some);
        }

        match req.call() {
            Ok(resp) => {
                if resp.status() == 304 {
                    return Ok(None);
                }
                // Got fresh content
                let info = self.save_response(url, hashed, resp)?;
                Ok(Some(info))
            }
            Err(ureq::Error::Status(304, _)) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("HTTP error fetching {url}: {e}")),
        }
    }

    fn fetch_and_cache_remote(&self, url: &str, hashed: &str) -> Result<ImageInfo> {
        let resp = ureq::get(url)
            .call()
            .with_context(|| format!("Failed to download remote image: {url}"))?;
        self.save_response(url, hashed, resp)
    }

    fn save_response(&self, url: &str, hashed: &str, resp: ureq::Response) -> Result<ImageInfo> {
        // Determine extension from Content-Type header first, then URL.
        let content_type = resp.header("content-type").unwrap_or("").to_string();
        let etag = resp.header("etag").unwrap_or("").to_string();
        let last_modified = resp.header("last-modified").unwrap_or("").to_string();

        let mut bytes = Vec::new();
        resp.into_reader()
            .read_to_end(&mut bytes)
            .with_context(|| format!("Failed to read remote image response: {url}"))?;

        let ext = detect_image_format(url, &content_type, &bytes);
        let is_svg = ext == "svg";

        let file_name = format!("remote-image-{}.{}", hashed, ext);
        let target = self.cache_dir.join(&file_name);
        fs::write(&target, &bytes)
            .with_context(|| format!("Failed to cache remote image: {}", target.display()))?;

        // Write metadata sidecar for future conditional requests.
        let meta_path = self.cache_dir.join(format!("remote-image-{}.meta", hashed));
        let meta_content = format!(
            "etag={}\nlast_modified={}\next={}\n",
            etag, last_modified, ext
        );
        let _ = fs::write(&meta_path, meta_content);

        Ok(ImageInfo {
            path: target,
            is_svg,
            natural_width: None,
            natural_height: None,
        })
    }

    /// Decode a `data:image/png;base64,<data>` URI into a cached file.
    fn resolve_data_uri(&self, uri: &str) -> Result<ImageInfo> {
        // data:[<mediatype>][;base64],<data>
        let rest = uri.strip_prefix("data:").unwrap_or(uri);
        let (header, encoded) = rest
            .split_once(',')
            .ok_or_else(|| anyhow::anyhow!("Malformed data URI"))?;
        let mime = header.split(';').next().unwrap_or("").to_ascii_lowercase();
        let ext = mime_to_ext(&mime).unwrap_or("bin");
        let is_base64 = header.contains("base64");
        let bytes: Vec<u8> = if is_base64 {
            decode_base64(encoded)?
        } else {
            percent_decode(encoded).into_owned().into_bytes()
        };

        let hashed = stable_name(uri);
        let file_name = format!("data-image-{}.{}", hashed, ext);
        let target = self.cache_dir.join(&file_name);
        if !target.exists() {
            fs::write(&target, &bytes)
                .with_context(|| format!("Failed to write data URI image: {}", target.display()))?;
        }
        let is_svg = ext == "svg";
        Ok(ImageInfo {
            path: target,
            is_svg,
            natural_width: None,
            natural_height: None,
        })
    }

    fn render_table<'a>(&mut self, node: &'a AstNode<'a>, table: &NodeTable) -> Result<String> {
        // Build per-column alignment string from GFM alignment markers.
        let alignments: Vec<&str> = table
            .alignments
            .iter()
            .map(|a| match a {
                TableAlignment::Left => "left",
                TableAlignment::Center => "center",
                TableAlignment::Right => "right",
                TableAlignment::None => "left", // default
            })
            .collect();

        // Collect rows; track which is the header.
        let mut rows: Vec<(bool, Vec<String>)> = Vec::new();
        for (row_idx, row) in node.children().enumerate() {
            let is_header = row_idx == 0;
            let mut cells = Vec::new();
            for (col_idx, cell) in row.children().enumerate() {
                let text = self.render_inline_children(cell)?;
                let align = alignments.get(col_idx).copied().unwrap_or("left");
                // Header cells: bold content; data cells: plain.
                let cell_content = if is_header {
                    format!("table.cell(align: {})[#strong[{}]]", align, text.trim())
                } else {
                    format!("table.cell(align: {})[{}]", align, text.trim())
                };
                cells.push(cell_content);
            }
            if !cells.is_empty() {
                rows.push((is_header, cells));
            }
        }

        if rows.is_empty() {
            return Ok(String::new());
        }

        let column_count = rows.iter().map(|(_, r)| r.len()).max().unwrap_or(0);

        // Build a column-width spec: all equal fractional columns.
        let col_spec = format!(
            "({})",
            (0..column_count)
                .map(|_| "1fr")
                .collect::<Vec<_>>()
                .join(", ")
        );

        let flat: Vec<String> = rows.into_iter().flat_map(|(_, cells)| cells).collect();

        Ok(format!(
            "#table(\n  columns: {},\n  stroke: luma(200),\n  inset: 6pt,\n  fill: (col, row) => if row == 0 {{ luma(230) }} else {{ white }},\n  {}\n)\n\n",
            col_spec,
            flat.join(",\n  ")
        ))
    }

    /// Emit the footnote section collected during document rendering.
    ///
    /// Footnotes are sorted by first-reference order (the `ix` from comrak's
    /// `FootnoteReference` nodes), so the printed numbers match the superscripts.
    fn emit_footnote_section(&self) -> String {
        if self.footnote_defs.is_empty() {
            return String::new();
        }
        let mut out = String::from("#line(length: 100%)\n\n");

        // Sort by the `ix` captured from FootnoteReference nodes (document order).
        let mut ordered: Vec<(&str, u32)> = self
            .footnote_refs
            .iter()
            .map(|(name, ix)| (name.as_str(), *ix))
            .collect();
        ordered.sort_by_key(|(_, ix)| *ix);

        for (name, ix) in &ordered {
            if let Some(body) = self.footnote_defs.get(*name) {
                out.push_str(&format!("#super[{}] {}\n\n", ix, body));
            }
        }

        // Emit any definitions not referenced in-text (ordered by their name
        // as a fallback — shouldn't normally happen in valid GFM).
        let mut extra_num = ordered.len() as u32 + 1;
        for (name, body) in &self.footnote_defs {
            if !ordered.iter().any(|(n, _)| n == name) {
                out.push_str(&format!("#super[{}] {}\n\n", extra_num, body));
                extra_num += 1;
            }
        }

        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GitHub Alert renderer
// ─────────────────────────────────────────────────────────────────────────────

fn render_alert(alert: &NodeAlert, body: &str) -> String {
    let (title, accent) = match alert.alert_type {
        AlertType::Note => (
            alert.title.as_deref().unwrap_or("Note").to_string(),
            "rgb(\"#0969da\")",
        ),
        AlertType::Tip => (
            alert.title.as_deref().unwrap_or("Tip").to_string(),
            "rgb(\"#1a7f37\")",
        ),
        AlertType::Important => (
            alert.title.as_deref().unwrap_or("Important").to_string(),
            "rgb(\"#8250df\")",
        ),
        AlertType::Warning => (
            alert.title.as_deref().unwrap_or("Warning").to_string(),
            "rgb(\"#9a6700\")",
        ),
        AlertType::Caution => (
            alert.title.as_deref().unwrap_or("Caution").to_string(),
            "rgb(\"#cf222e\")",
        ),
    };
    let escaped_title = escape_typst_text(&title);
    format!(
        "#block(stroke: (left: 3pt + {accent}), inset: (left: 12pt, top: 8pt, bottom: 8pt, right: 8pt), width: 100%)[#text(fill: {accent})[#strong[{escaped_title}]]\\\n{body}]\n\n",
        accent = accent,
        escaped_title = escaped_title,
        body = body.trim(),
    )
}

/// Extract TOC entries by walking the comrak-parsed AST.
///
/// This replaces the old line-scanner approach and correctly handles:
/// - Headings inside fenced code blocks (ignored by the AST, not picked up)
/// - Inline markup in heading text (bold/italic/code/etc. — rendered as plain
///   text so the TOC title is clean)
/// - Setext-style headings (`===` / `---` underline syntax)
/// - Edge cases like ATX headings with no trailing space, etc.
///
/// The `page_number` field is always 0 here; callers that need real page
/// numbers must get them from Typst's outline at compile time.
fn extract_toc(markdown: &str) -> Vec<TocEntry> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &markdown_options());
    let mut entries = Vec::new();
    collect_toc_entries(root, &mut entries);
    entries
}

/// Recursively walk an AST node, collecting `TocEntry` values from every
/// `Heading` node encountered in document order.
fn collect_toc_entries<'a>(node: &'a AstNode<'a>, entries: &mut Vec<TocEntry>) {
    for child in node.children() {
        if let NodeValue::Heading(NodeHeading { level, .. }) = child.data.borrow().value {
            let title = extract_plain_text(child);
            entries.push(TocEntry {
                level,
                title: title.trim().to_string(),
                page_number: 0,
            });
        } else {
            collect_toc_entries(child, entries);
        }
    }
}

/// Recursively collect plain text from all descendant `Text`, `Code`,
/// `SoftBreak`, `LineBreak`, and `Math` nodes — stripping any Typst or
/// Markdown inline markup so the TOC title is readable as plain text.
fn extract_plain_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut out = String::new();
    for child in node.children() {
        let val = child.data.borrow();
        match &val.value {
            NodeValue::Text(t) => out.push_str(t),
            NodeValue::Code(c) => out.push_str(&c.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
            NodeValue::Math(m) => out.push_str(&m.literal),
            NodeValue::HtmlInline(_) => {} // skip raw HTML fragments
            _ => out.push_str(&extract_plain_text(child)),
        }
    }
    out
}

/// Generate a Typst `#outline()` block for the TOC page.
///
/// Typst's built-in `outline()` automatically:
/// - Computes real page numbers for every heading
/// - Renders dotted leader lines between title and page number
/// - Makes each entry a clickable internal link (via the `<label>` anchors
///   we attach to every heading)
///
/// We control depth with the `depth:` parameter and support a custom title.
///
/// After the outline block we emit `#pagebreak()` so that body content
/// always begins on a fresh page, separate from the TOC.
fn generate_typst_toc(toc_depth: u8, title: &str) -> String {
    let escaped_title = escape_typst_text(title);
    format!(
        // Show rule: H1 entries are rendered in bold weight.
        // Typst's built-in #outline() automatically:
        //   - computes real page numbers for every heading
        //   - renders dot leaders between heading text and page number
        //   - makes each entry a clickable internal link (via <label> anchors)
        // We use `indent: 1.5em` to visually nest sub-headings and apply a
        // `#show` rule to make H1 entries bold.
        "#show outline.entry.where(level: 1): it => {{\n\
  strong(it)\n\
}}\n\
#outline(\n\
  title: [{escaped_title}],\n\
  depth: {toc_depth},\n\
  indent: 1.5em,\n\
)\n\
#pagebreak()\n\n"
    )
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
                if i >= chars.len() {
                    out.push('\\');
                    continue;
                }
                match chars[i] {
                    // Line break inside matrices / aligned envs
                    '\\' => {
                        out.push_str("\\ ");
                        i += 1;
                        continue;
                    }
                    // Spacing commands → single space
                    ',' | ':' | ';' | '!' | ' ' => {
                        out.push(' ');
                        i += 1;
                        continue;
                    }
                    // Escaped braces → literal brace
                    '{' => {
                        out.push('{');
                        i += 1;
                        continue;
                    }
                    '}' => {
                        out.push('}');
                        i += 1;
                        continue;
                    }
                    _ => {}
                }
                // Collect alphabetic command name
                let cmd_start = i;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }
                let cmd: String = chars[cmd_start..i].iter().collect();
                // Skip optional trailing space after command name
                if i < chars.len() && chars[i] == ' ' && !cmd.is_empty() {
                    i += 1;
                }

                // Insert a leading space if the output buffer ends with an
                // identifier char and the translated command will start with one
                // — prevents merging like `i` + `pi` → `ipi`.
                let prev_is_ident = out
                    .chars()
                    .last()
                    .map(|c| c.is_alphanumeric() || c == '\'')
                    .unwrap_or(false);

                let translated = math_cmd(&cmd, &chars, &mut i);

                if prev_is_ident
                    && let Some(first) = translated.chars().next()
                    && (first.is_alphanumeric() || first == '"')
                {
                    out.push(' ');
                }
                out.push_str(&translated);

                // Insert a trailing space when the translation ends with an
                // identifier char and the next input char is also alphanumeric
                // — prevents e.g. `gt.eq0` instead of `gt.eq 0`.
                if let (Some(last), Some(&next)) = (translated.chars().last(), chars.get(i))
                    && (last.is_alphanumeric() || last == '.')
                    && (next.is_alphanumeric() || next == '_')
                {
                    out.push(' ');
                }
            }
            // Braces pass through as Typst grouping.
            // `math_cmd`'s `consume` helper already consumed the braces that
            // belonged to function arguments.
            '{' => {
                out.push('{');
                i += 1;
            }
            '}' => {
                out.push('}');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Translate a single LaTeX command name to its Typst equivalent.
/// `chars` and `i` allow consuming brace-delimited arguments.
fn math_cmd(cmd: &str, chars: &[char], i: &mut usize) -> String {
    // Consume one brace-group {…} or single token, returning it translated.
    let consume = |chars: &[char], i: &mut usize| -> String {
        while *i < chars.len() && chars[*i] == ' ' {
            *i += 1;
        }
        if *i >= chars.len() {
            return String::new();
        }
        if chars[*i] == '{' {
            *i += 1;
            let mut depth = 1usize;
            let mut inner = String::new();
            while *i < chars.len() {
                match chars[*i] {
                    '{' => {
                        depth += 1;
                        inner.push('{');
                        *i += 1;
                    }
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            *i += 1;
                            break;
                        }
                        inner.push('}');
                        *i += 1;
                    }
                    c => {
                        inner.push(c);
                        *i += 1;
                    }
                }
            }
            latex_to_typst(&inner)
        } else {
            let c = chars[*i];
            *i += 1;
            if c == '\\' {
                let start = *i;
                while *i < chars.len() && chars[*i].is_ascii_alphabetic() {
                    *i += 1;
                }
                let sub: String = chars[start..*i].iter().collect();
                math_cmd(&sub, chars, i)
            } else {
                c.to_string()
            }
        }
    };

    match cmd {
        // ── fractions ─────────────────────────────────────────────────
        "frac" | "dfrac" | "tfrac" | "cfrac" => {
            let n = consume(chars, i);
            let d = consume(chars, i);
            format!("frac({}, {})", n, d)
        }
        "binom" => {
            let n = consume(chars, i);
            let k = consume(chars, i);
            format!("binom({}, {})", n, k)
        }

        // ── roots ─────────────────────────────────────────────────────
        "sqrt" => {
            while *i < chars.len() && chars[*i] == ' ' {
                *i += 1;
            }
            if *i < chars.len() && chars[*i] == '[' {
                *i += 1;
                let mut n = String::new();
                while *i < chars.len() && chars[*i] != ']' {
                    n.push(chars[*i]);
                    *i += 1;
                }
                if *i < chars.len() {
                    *i += 1;
                }
                format!("root({}, {})", latex_to_typst(&n), consume(chars, i))
            } else {
                format!("sqrt({})", consume(chars, i))
            }
        }

        // ── text in math ──────────────────────────────────────────────
        "text" | "mathrm" | "mathit" | "mathsf" | "mathtt" | "operatorname" => {
            format!("\"{}\"", consume(chars, i).replace('"', "\\\""))
        }
        "mathbf" | "textbf" | "boldsymbol" | "bm" => format!("bold({})", consume(chars, i)),
        "textrm" | "textnormal" => format!("\"{}\"", consume(chars, i).replace('"', "\\\"")),
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
        "hat" | "widehat" => format!("hat({})", consume(chars, i)),
        "tilde" | "widetilde" => format!("tilde({})", consume(chars, i)),
        "bar" | "overline" => format!("overline({})", consume(chars, i)),
        "underline" => format!("underline({})", consume(chars, i)),
        "vec" => format!("arrow({})", consume(chars, i)),
        "dot" => format!("dot({})", consume(chars, i)),
        "ddot" => format!("dot.double({})", consume(chars, i)),
        "underbrace" => format!("underbrace({})", consume(chars, i)),
        "overbrace" => format!("overbrace({})", consume(chars, i)),

        // ── trig / log functions ──────────────────────────────────────
        "sin" => "sin".into(),
        "cos" => "cos".into(),
        "tan" => "tan".into(),
        "sec" => "sec".into(),
        "csc" => "csc".into(),
        "cot" => "cot".into(),
        "arcsin" => "arcsin".into(),
        "arccos" => "arccos".into(),
        "arctan" => "arctan".into(),
        "sinh" => "sinh".into(),
        "cosh" => "cosh".into(),
        "tanh" => "tanh".into(),
        "ln" => "ln".into(),
        "log" => "log".into(),
        "exp" => "exp".into(),
        "lim" => "lim".into(),
        "limsup" => "limsup".into(),
        "liminf" => "liminf".into(),
        "sup" => "sup".into(),
        "inf" => "inf".into(),
        "max" => "max".into(),
        "min" => "min".into(),
        "arg" => "arg".into(),
        "det" => "det".into(),
        "dim" => "dim".into(),
        "gcd" => "gcd".into(),
        "hom" => "hom".into(),
        "ker" => "ker".into(),

        // ── sums / integrals ──────────────────────────────────────────
        "sum" => "sum".into(),
        "prod" => "prod".into(),
        "int" => "integral".into(),
        "iint" => "integral.double".into(),
        "iiint" => "integral.triple".into(),
        "oint" => "integral.cont".into(),
        "bigcup" => "union.big".into(),
        "bigcap" => "sect.big".into(),

        // ── Greek (lowercase) ─────────────────────────────────────────
        "alpha" => "alpha".into(),
        "beta" => "beta".into(),
        "gamma" => "gamma".into(),
        "delta" => "delta".into(),
        "epsilon" => "epsilon".into(),
        "varepsilon" => "epsilon.alt".into(),
        "zeta" => "zeta".into(),
        "eta" => "eta".into(),
        "theta" => "theta".into(),
        "vartheta" => "theta.alt".into(),
        "iota" => "iota".into(),
        "kappa" => "kappa".into(),
        "lambda" => "lambda".into(),
        "mu" => "mu".into(),
        "nu" => "nu".into(),
        "xi" => "xi".into(),
        "pi" => "pi".into(),
        "varpi" => "pi.alt".into(),
        "rho" => "rho".into(),
        "varrho" => "rho.alt".into(),
        "sigma" => "sigma".into(),
        "varsigma" => "sigma.alt".into(),
        "tau" => "tau".into(),
        "upsilon" => "upsilon".into(),
        "phi" => "phi.alt".into(),
        "varphi" => "phi".into(),
        "chi" => "chi".into(),
        "psi" => "psi".into(),
        "omega" => "omega".into(),
        // Greek (uppercase)
        "Gamma" => "Gamma".into(),
        "Delta" => "Delta".into(),
        "Theta" => "Theta".into(),
        "Lambda" => "Lambda".into(),
        "Xi" => "Xi".into(),
        "Pi" => "Pi".into(),
        "Sigma" => "Sigma".into(),
        "Upsilon" => "Upsilon".into(),
        "Phi" => "Phi".into(),
        "Psi" => "Psi".into(),
        "Omega" => "Omega".into(),

        // ── binary operators / relations ──────────────────────────────
        "cdot" => "dot.c".into(),
        "cdots" => "dots.c".into(),
        "ldots" | "dots" => "dots".into(),
        "vdots" => "dots.v".into(),
        "ddots" => "dots.down".into(),
        "times" => "times".into(),
        "div" => "div".into(),
        "pm" => "plus.minus".into(),
        "mp" => "minus.plus".into(),
        "leq" | "le" => "lt.eq".into(),
        "geq" | "ge" => "gt.eq".into(),
        "neq" | "ne" => "eq.not".into(),
        "approx" => "approx".into(),
        "sim" => "tilde.op".into(),
        "simeq" => "tilde.eq".into(),
        "cong" => "tilde.equiv".into(),
        "equiv" => "equiv".into(),
        "propto" => "prop".into(),
        "ll" => "lt.double".into(),
        "gg" => "gt.double".into(),
        "in" => "in".into(),
        "notin" => "in.not".into(),
        "subset" => "subset".into(),
        "subseteq" => "subset.eq".into(),
        "supset" => "supset".into(),
        "supseteq" => "supset.eq".into(),
        "cup" => "union".into(),
        "cap" => "sect".into(),
        "setminus" => "without".into(),
        "emptyset" | "varnothing" => "nothing".into(),
        "forall" => "forall".into(),
        "exists" => "exists".into(),
        "nexists" => "exists.not".into(),
        "neg" | "lnot" => "not".into(),
        "land" | "wedge" => "and".into(),
        "lor" | "vee" => "or".into(),
        "oplus" => "plus.circle".into(),
        "otimes" => "times.circle".into(),
        "circ" => "circle.small".into(),
        "bullet" => "bullet".into(),

        // ── arrows ────────────────────────────────────────────────────
        "to" | "rightarrow" => "->".into(),
        "leftarrow" => "<-".into(),
        "Rightarrow" => "=>".into(),
        "Leftarrow" => "<=".into(),
        "leftrightarrow" => "<->".into(),
        "Leftrightarrow" => "<=>".into(),
        "mapsto" => "|->".into(),
        "uparrow" => "arrow.t".into(),
        "downarrow" => "arrow.b".into(),
        "updownarrow" => "arrow.t.b".into(),
        "longrightarrow" => "-->".into(),
        "longleftarrow" => "<--".into(),

        // ── misc symbols ──────────────────────────────────────────────
        "partial" => "diff".into(),
        "nabla" => "nabla".into(),
        "infty" => "oo".into(),
        "hbar" => "planck.reduce".into(),
        "ell" => "ell".into(),
        "Re" => "Re".into(),
        "Im" => "Im".into(),
        "aleph" => "aleph".into(),
        "prime" => "'".into(),
        "dagger" => "dagger".into(),
        "ddagger" => "dagger.double".into(),
        "star" => "star".into(),
        "ast" => "ast".into(),

        // ── delimiters (auto-sized in Typst) ──────────────────────────
        "left" | "right" => {
            while *i < chars.len() && chars[*i] == ' ' {
                *i += 1;
            }
            if *i < chars.len() {
                let d = chars[*i];
                *i += 1;
                if d == '.' {
                    String::new()
                } else {
                    d.to_string()
                }
            } else {
                String::new()
            }
        }
        "langle" => "angle.l".into(),
        "rangle" => "angle.r".into(),
        "lfloor" => "floor.l".into(),
        "rfloor" => "floor.r".into(),
        "lceil" => "ceil.l".into(),
        "rceil" => "ceil.r".into(),
        "lVert" | "rVert" => "||".into(),
        "lvert" | "rvert" => "|".into(),

        // ── environments ──────────────────────────────────────────────
        "begin" => {
            let env = consume(chars, i);
            math_env(&env, chars, i)
        }
        "end" => {
            consume(chars, i);
            String::new()
        }

        // ── layout / spacing ──────────────────────────────────────────
        "quad" => "quad".into(),
        "qquad" => "wide".into(),
        "hspace" | "vspace" => {
            consume(chars, i);
            " ".into()
        }
        "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle" => String::new(),
        "limits" | "nolimits" | "nonumber" | "notag" => String::new(),
        "label" | "tag" => {
            consume(chars, i);
            String::new()
        }

        // ── unknown: pass through so it fails visibly ──────────────────
        other => format!("\\{}", other),
    }
}

/// Handle `\begin{env}...\end{env}` environments.
fn math_env(env: &str, chars: &[char], i: &mut usize) -> String {
    let begin_m = format!("\\begin{{{}}}", env);
    let end_m = format!("\\end{{{}}}", env);
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
            if depth == 0 {
                j += end_m.len();
                break;
            }
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
        "matrix" => format!("mat(delim: #none, {})", math_matrix_body(&inner)),
        "pmatrix" => format!("mat({})", math_matrix_body(&inner)),
        "bmatrix" => format!("mat(delim: \"[\", {})", math_matrix_body(&inner)),
        "Bmatrix" => format!("mat(delim: \"{{\", {})", math_matrix_body(&inner)),
        "vmatrix" => format!("mat(delim: \"|\", {})", math_matrix_body(&inner)),
        "Vmatrix" => format!("mat(delim: \"||\", {})", math_matrix_body(&inner)),
        "cases" => math_cases(&inner),
        "align" | "align*" | "aligned" => inner
            .split("\\\\")
            .map(|r| latex_to_typst(r.trim()))
            .collect::<Vec<_>>()
            .join(" \\ "),
        "equation" | "equation*" => latex_to_typst(inner.trim()),
        _ => latex_to_typst(inner.trim()),
    }
}

/// Translate `cases` environment body.
/// LaTeX: `expr & \text{if} cond \\` rows
/// Typst: `cases(expr &"if" cond, ...)`
fn math_cases(inner: &str) -> String {
    let rows: Vec<String> = inner
        .split("\\\\")
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
    for p in &[
        r"\text{if }",
        r"\text{if}",
        r"\text{ if }",
        r"\text{ if}",
        r"\text{otherwise}",
        r"\text{else}",
    ] {
        if let Some(stripped) = s.strip_prefix(p) {
            return stripped;
        }
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

/// Convert a heading title to a Typst label identifier.
///
/// Typst labels must be valid identifiers: start with a letter or underscore,
/// contain only ASCII letters, digits, hyphens, underscores, and dots.
/// We strip Typst markup (e.g. `#strong[…]`) and convert to kebab-case.
fn heading_label(title: &str) -> String {
    // Strip simple Typst markup functions like `#strong[…]`, `#emph[…]`
    let stripped = strip_typst_markup(title);
    let slug: String = stripped
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    // Collapse consecutive dashes and trim leading/trailing dashes
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty()
        || slug
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        format!("h-{}", slug)
    } else {
        slug
    }
}

/// Strip Typst markup function calls like `#strong[text]` → `text`.
fn strip_typst_markup(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '#' {
            // Skip `#funcname[`
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            if i < chars.len() && chars[i] == '[' {
                i += 1; // skip '['
            }
        } else if chars[i] == ']' {
            i += 1; // close bracket from markup — skip it
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn escape_typst_text(s: &str) -> String {
    s.replace('\n', " ")
        .replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('@', "\\@") // Prevent @label citation syntax
        .replace('$', "\\$") // Prevent accidental math mode
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

/// Compute a stable, deterministic hex name for a string.
///
/// Uses FNV-1a (64-bit) — a public-domain, non-cryptographic hash that
/// produces the **same output across every Rust version, platform, and
/// process invocation**.  This is intentionally different from
/// `std::collections::hash_map::DefaultHasher`, whose output is explicitly
/// *not* guaranteed to be stable across compilations or runs.
///
/// The hex digest is used as the filename stem for cached remote-image files,
/// so stability is essential: the same URL must always map to the same path.
fn stable_name(s: &str) -> String {
    // FNV-1a 64-bit parameters (public domain).
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;
    let mut hash = FNV_OFFSET;
    for byte in s.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{:016x}", hash)
}

// ─────────────────────────────────────────────────────────────────────────────
// Image pipeline helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Resolved information about an image ready to emit into Typst.
#[derive(Debug)]
pub struct ImageInfo {
    /// Absolute path to the (possibly cached) image file.
    pub path: PathBuf,
    /// Whether the image is SVG (requires `format: "svg"` in Typst).
    pub is_svg: bool,
    /// Natural pixel width if known (e.g. parsed from HTML `width=` attr).
    /// Reserved for future intrinsic-size calculations.
    #[allow(dead_code)]
    pub natural_width: Option<u32>,
    /// Natural pixel height if known (e.g. parsed from HTML `height=` attr).
    /// Reserved for future intrinsic-size calculations.
    #[allow(dead_code)]
    pub natural_height: Option<u32>,
}

/// Sizing hints extracted from HTML `width` / `height` attributes or Markdown
/// title extras like `![alt](url){width=50%}`.
#[derive(Debug, Default, Clone)]
pub struct SizeHint {
    /// Explicit width from markup, already normalized to a Typst value.
    pub width: Option<String>,
    /// Explicit height from markup, already normalized to a Typst value.
    pub height: Option<String>,
}

/// Cached metadata sidecar for remote images.
#[derive(Debug, Default, Clone)]
struct CacheMeta {
    /// HTTP ETag for conditional requests.
    etag: String,
    /// HTTP Last-Modified for conditional requests.
    last_modified: String,
    /// File extension of the cached file, e.g. `"png"`.
    ext: String,
}

fn read_cache_meta(path: &Path) -> Option<CacheMeta> {
    let content = fs::read_to_string(path).ok()?;
    let mut meta = CacheMeta::default();
    for line in content.lines() {
        if let Some(v) = line.strip_prefix("etag=") {
            meta.etag = v.to_string();
        } else if let Some(v) = line.strip_prefix("last_modified=") {
            meta.last_modified = v.to_string();
        } else if let Some(v) = line.strip_prefix("ext=") {
            meta.ext = v.to_string();
        }
    }
    if meta.ext.is_empty() {
        None
    } else {
        Some(meta)
    }
}

/// Determine the image format (file extension) using:
/// 1. `Content-Type` response header (most reliable).
/// 2. Magic bytes in the downloaded body.
/// 3. URL path extension as last resort.
pub fn detect_image_format(url: &str, content_type: &str, bytes: &[u8]) -> &'static str {
    // 1. Content-Type header
    let ct = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if let Some(ext) = mime_to_ext(&ct) {
        return ext;
    }

    // 2. Magic bytes
    if let Some(ext) = sniff_image_magic(bytes) {
        return ext;
    }

    // 3. URL path extension
    guess_extension_from_url(url)
}

/// Map a MIME type to a file extension.
pub fn mime_to_ext(mime: &str) -> Option<&'static str> {
    match mime {
        "image/png" => Some("png"),
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/svg+xml" | "image/svg" => Some("svg"),
        "image/bmp" => Some("bmp"),
        "image/tiff" => Some("tiff"),
        "image/avif" => Some("avif"),
        "image/x-icon" | "image/vnd.microsoft.icon" => Some("ico"),
        _ => None,
    }
}

/// Detect image format by inspecting the first few bytes.
pub fn sniff_image_magic(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 4 {
        return None;
    }
    // PNG: 0x89 P N G
    if bytes.starts_with(b"\x89PNG") {
        return Some("png");
    }
    // JPEG: FF D8 FF
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some("jpg");
    }
    // GIF: GIF87a or GIF89a
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("gif");
    }
    // WebP: RIFF????WEBP
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("webp");
    }
    // BMP: BM
    if bytes.starts_with(b"BM") {
        return Some("bmp");
    }
    // TIFF: II* or MM*
    if bytes.starts_with(b"II\x2a\x00") || bytes.starts_with(b"MM\x00\x2a") {
        return Some("tiff");
    }
    // SVG: look for XML declaration or <svg at the start (after optional BOM/whitespace)
    if is_svg_bytes(bytes) {
        return Some("svg");
    }
    None
}

/// Heuristic: check if the byte slice looks like an SVG document.
pub fn is_svg_bytes(bytes: &[u8]) -> bool {
    // Skip BOM if present
    let start = if bytes.starts_with(b"\xef\xbb\xbf") {
        3
    } else {
        0
    };
    let snippet = &bytes[start..bytes.len().min(start + 512)];
    if let Ok(s) = std::str::from_utf8(snippet) {
        let trimmed = s.trim_start();
        // A file is SVG when it either begins directly with an `<svg` element
        // or has an XML declaration followed somewhere by an `<svg` open tag.
        // We must NOT treat arbitrary XML files (e.g. XHTML, Atom feeds) as SVG.
        if trimmed.starts_with("<svg")
            || trimmed.contains("<svg ")
            || trimmed.contains("<svg\t")
            || trimmed.contains("<svg\n")
            || trimmed.contains("<svg>")
        {
            return true;
        }
        // XML declaration: only accept if followed by an <svg element.
        if trimmed.starts_with("<?xml") {
            let lower = trimmed.to_ascii_lowercase();
            return lower.contains("<svg");
        }
        false
    } else {
        false
    }
}

/// Guess file extension from URL path only.
fn guess_extension_from_url(url: &str) -> &'static str {
    // Strip query string and fragment
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);
    let lower = path.to_ascii_lowercase();

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
    } else if lower.ends_with(".bmp") {
        "bmp"
    } else if lower.ends_with(".tiff") || lower.ends_with(".tif") {
        "tiff"
    } else if lower.ends_with(".avif") {
        "avif"
    } else if lower.ends_with(".ico") {
        "ico"
    } else {
        "img"
    }
}

/// Emit a Typst `#figure(image(…))` block for a resolved image.
///
/// Rules:
/// - SVG images get `format: "svg"` so Typst parses them correctly.
/// - If a width is specified in the size hint, use it; otherwise default to 100%.
/// - If both width *and* height are specified, emit both so the aspect ratio
///   is preserved (Typst will honour both if they are consistent).
/// - Alt text becomes a figure caption when non-empty.
pub fn format_image_typst(info: &ImageInfo, alt: &str) -> String {
    format_image_typst_sized(info, alt, &SizeHint::default())
}

/// Like [`format_image_typst`] but accepts explicit size overrides.
pub fn format_image_typst_sized(info: &ImageInfo, alt: &str, hint: &SizeHint) -> String {
    let path_arg = typst_quoted_string(&info.path.to_string_lossy());

    // Build image() arguments
    let mut img_args = vec![path_arg];

    if info.is_svg {
        img_args.push("format: \"svg\"".to_string());
    }

    // Width
    let width_val = hint.width.as_deref().unwrap_or("100%");
    img_args.push(format!("width: {}", width_val));

    // Height — only emit when explicitly requested to avoid distorting the image.
    if let Some(h) = &hint.height {
        img_args.push(format!("height: {}", h));
    }

    let img_call = format!("image({})", img_args.join(", "));

    // Caption
    let caption_arg = if alt.is_empty() {
        String::new()
    } else {
        format!(", caption: [{}]", escape_typst_text(alt))
    };

    format!("#figure({}{})\n\n", img_call, caption_arg)
}

/// Render a styled placeholder block for an image that could not be loaded.
///
/// Emits a grey rounded box with the alt text (or the URL if no alt text) so
/// the PDF is still complete and legible when an image is unavailable.
pub fn missing_image_fallback(url: &str, alt: &str) -> String {
    // Always include the filename so diagnostics are easy — even when alt text is set.
    let short_url = {
        let s = url.split('/').rfind(|s| !s.is_empty()).unwrap_or(url);
        s.split('?').next().unwrap_or(s)
    };
    let label = if !alt.is_empty() {
        format!(
            "{} ({})",
            escape_typst_text(alt),
            escape_typst_text(short_url)
        )
    } else {
        escape_typst_text(short_url)
    };

    format!(
        "#block(\
fill: luma(235), \
stroke: 1pt + luma(180), \
inset: 10pt, \
radius: 4pt, \
width: 100%\
)[#align(center)[#text(fill: luma(120))[\\[Image: {}\\]]]]\n\n",
        label
    )
}

/// Convert a CSS-style `width` or `height` attribute value to a Typst measure.
///
/// Handles:
/// - `200px` → `"150pt"` (72 pt/in, 96 px/in → multiply by 0.75)
/// - `50%`   → `"50%"`
/// - `200`   → `"150pt"` (bare integer treated as pixels)
/// - `10em`  → `"10em"`
/// - `10pt`  → `"10pt"`
/// - `10cm`  → `"10cm"`
/// - `10mm`  → `"10mm"`
pub fn css_length_to_typst(val: &str) -> Option<String> {
    let v = val.trim();
    if v.is_empty() {
        return None;
    }
    if v.ends_with('%') {
        return Some(v.to_string());
    }
    if v.ends_with("px") {
        let n: f64 = v.trim_end_matches("px").trim().parse().ok()?;
        return Some(format!("{:.1}pt", n * 0.75));
    }
    if v.ends_with("rem") {
        // `rem` has no direct Typst equivalent; map to `em` (root-relative ≈ em in PDF context).
        let n = v.trim_end_matches("rem");
        return Some(format!("{}em", n));
    }
    if v.ends_with("em") {
        return Some(v.to_string());
    }
    if v.ends_with("pt") || v.ends_with("mm") || v.ends_with("cm") || v.ends_with("in") {
        return Some(v.to_string());
    }
    // Bare integer → treat as pixels
    if let Ok(n) = v.parse::<f64>() {
        return Some(format!("{:.1}pt", n * 0.75));
    }
    None
}

/// Decode a base64 string (standard or URL-safe alphabet, with or without padding).
pub fn decode_base64(s: &str) -> Result<Vec<u8>> {
    // Remove all whitespace (base64 in data URIs sometimes has line breaks)
    let clean: String = s.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    // Simple base64 decoder (standard + URL-safe, tolerates missing padding)
    let alphabet_std: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let alphabet_url: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let url_safe = clean.contains('-') || clean.contains('_');
    let alphabet = if url_safe { alphabet_url } else { alphabet_std };

    let mut table = [0u8; 256];
    for (i, &b) in alphabet.iter().enumerate() {
        table[b as usize] = i as u8;
    }

    let bytes = clean.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut i = 0;
    while i + 3 < bytes.len() {
        let b0 = if bytes[i] == b'=' {
            0
        } else {
            table[bytes[i] as usize]
        };
        let b1 = if bytes[i + 1] == b'=' {
            0
        } else {
            table[bytes[i + 1] as usize]
        };
        let b2 = if bytes[i + 2] == b'=' {
            0
        } else {
            table[bytes[i + 2] as usize]
        };
        let b3 = if bytes[i + 3] == b'=' {
            0
        } else {
            table[bytes[i + 3] as usize]
        };
        let v = ((b0 as u32) << 18) | ((b1 as u32) << 12) | ((b2 as u32) << 6) | (b3 as u32);
        out.push((v >> 16) as u8);
        if bytes[i + 2] != b'=' {
            out.push(((v >> 8) & 0xff) as u8);
        }
        if bytes[i + 3] != b'=' {
            out.push((v & 0xff) as u8);
        }
        i += 4;
    }
    Ok(out)
}

/// Minimal percent-decode for data URI payloads.
fn percent_decode(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains('%') {
        return std::borrow::Cow::Borrowed(s);
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            )
        {
            out.push(((hi << 4) | lo) as u8);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    std::borrow::Cow::Owned(String::from_utf8_lossy(&out).into_owned())
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

// ─────────────────────────────────────────────────────────────────────────────
// Image pipeline tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod image_tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ── detect_image_format ───────────────────────────────────────────────

    #[test]
    fn detect_format_content_type_png() {
        let bytes = b"\x89PNG\r\n\x1a\n";
        assert_eq!(
            detect_image_format("https://example.com/img", "image/png", bytes),
            "png"
        );
    }

    #[test]
    fn detect_format_content_type_jpeg() {
        let bytes = b"\xff\xd8\xff\xe0";
        assert_eq!(
            detect_image_format("https://example.com/img", "image/jpeg", bytes),
            "jpg"
        );
    }

    #[test]
    fn detect_format_content_type_svg() {
        let bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
        assert_eq!(
            detect_image_format("https://example.com/img", "image/svg+xml", bytes),
            "svg"
        );
    }

    #[test]
    fn detect_format_content_type_webp() {
        let bytes = b"RIFFxxxxWEBP";
        assert_eq!(
            detect_image_format("https://example.com/img", "image/webp", bytes),
            "webp"
        );
    }

    #[test]
    fn detect_format_content_type_strips_params() {
        // "image/png; charset=utf-8" should still resolve to png
        let bytes = b"\x89PNG";
        assert_eq!(
            detect_image_format("https://example.com/x", "image/png; charset=utf-8", bytes),
            "png"
        );
    }

    // ── sniff_image_magic ─────────────────────────────────────────────────

    #[test]
    fn sniff_magic_png() {
        let bytes = b"\x89PNG\r\n\x1a\nextra";
        assert_eq!(sniff_image_magic(bytes), Some("png"));
    }

    #[test]
    fn sniff_magic_jpeg() {
        let bytes = b"\xff\xd8\xffextra";
        assert_eq!(sniff_image_magic(bytes), Some("jpg"));
    }

    #[test]
    fn sniff_magic_gif87() {
        assert_eq!(sniff_image_magic(b"GIF87aXXXX"), Some("gif"));
    }

    #[test]
    fn sniff_magic_gif89() {
        assert_eq!(sniff_image_magic(b"GIF89aXXXX"), Some("gif"));
    }

    #[test]
    fn sniff_magic_webp() {
        let mut bytes = b"RIFF\x00\x00\x00\x00WEBP".to_vec();
        bytes.extend_from_slice(b"extra");
        assert_eq!(sniff_image_magic(&bytes), Some("webp"));
    }

    #[test]
    fn sniff_magic_bmp() {
        assert_eq!(sniff_image_magic(b"BMextra"), Some("bmp"));
    }

    #[test]
    fn sniff_magic_tiff_le() {
        assert_eq!(sniff_image_magic(b"II\x2a\x00extra"), Some("tiff"));
    }

    #[test]
    fn sniff_magic_tiff_be() {
        assert_eq!(sniff_image_magic(b"MM\x00\x2aextra"), Some("tiff"));
    }

    #[test]
    fn sniff_magic_short_returns_none() {
        assert_eq!(sniff_image_magic(b"PNG"), None);
    }

    #[test]
    fn sniff_magic_unknown_returns_none() {
        assert_eq!(sniff_image_magic(b"????garbage"), None);
    }

    // ── is_svg_bytes ──────────────────────────────────────────────────────

    #[test]
    fn is_svg_bytes_xml_declaration() {
        let bytes = b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
        assert!(is_svg_bytes(bytes));
    }

    #[test]
    fn is_svg_bytes_bare_svg_tag() {
        let bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\"></svg>";
        assert!(is_svg_bytes(bytes));
    }

    #[test]
    fn is_svg_bytes_with_bom() {
        let mut bytes = b"\xef\xbb\xbf".to_vec();
        bytes.extend_from_slice(b"<svg></svg>");
        assert!(is_svg_bytes(&bytes));
    }

    #[test]
    fn is_svg_bytes_whitespace_before_tag() {
        let bytes = b"   \n\t<svg></svg>";
        assert!(is_svg_bytes(bytes));
    }

    #[test]
    fn is_svg_bytes_png_is_not_svg() {
        let bytes = b"\x89PNG\r\n\x1a\n";
        assert!(!is_svg_bytes(bytes));
    }

    #[test]
    fn is_svg_bytes_html_is_not_svg() {
        let bytes = b"<!DOCTYPE html><html></html>";
        assert!(!is_svg_bytes(bytes));
    }

    // ── guess_extension_from_url ──────────────────────────────────────────

    #[test]
    fn url_ext_png() {
        assert_eq!(
            guess_extension_from_url("https://example.com/logo.png"),
            "png"
        );
    }

    #[test]
    fn url_ext_jpeg() {
        assert_eq!(
            guess_extension_from_url("https://cdn.example.com/photo.jpeg"),
            "jpg"
        );
    }

    #[test]
    fn url_ext_jpg() {
        assert_eq!(
            guess_extension_from_url("https://cdn.example.com/photo.jpg"),
            "jpg"
        );
    }

    #[test]
    fn url_ext_svg() {
        assert_eq!(
            guess_extension_from_url("https://example.com/icon.svg"),
            "svg"
        );
    }

    #[test]
    fn url_ext_strips_query() {
        assert_eq!(
            guess_extension_from_url("https://example.com/img.png?v=2&size=large"),
            "png"
        );
    }

    #[test]
    fn url_ext_strips_fragment() {
        assert_eq!(
            guess_extension_from_url("https://example.com/photo.jpg#section"),
            "jpg"
        );
    }

    #[test]
    fn url_ext_no_extension_returns_img() {
        assert_eq!(
            guess_extension_from_url("https://example.com/images/photo"),
            "img"
        );
    }

    #[test]
    fn url_ext_case_insensitive() {
        assert_eq!(
            guess_extension_from_url("https://example.com/img.PNG"),
            "png"
        );
        assert_eq!(
            guess_extension_from_url("https://example.com/img.GIF"),
            "gif"
        );
    }

    // ── mime_to_ext ───────────────────────────────────────────────────────

    #[test]
    fn mime_png() {
        assert_eq!(mime_to_ext("image/png"), Some("png"));
    }

    #[test]
    fn mime_jpeg() {
        assert_eq!(mime_to_ext("image/jpeg"), Some("jpg"));
    }

    #[test]
    fn mime_svg() {
        assert_eq!(mime_to_ext("image/svg+xml"), Some("svg"));
    }

    #[test]
    fn mime_webp() {
        assert_eq!(mime_to_ext("image/webp"), Some("webp"));
    }

    #[test]
    fn mime_gif() {
        assert_eq!(mime_to_ext("image/gif"), Some("gif"));
    }

    #[test]
    fn mime_unknown_returns_none() {
        assert_eq!(mime_to_ext("application/octet-stream"), None);
        assert_eq!(mime_to_ext(""), None);
    }

    // ── css_length_to_typst ───────────────────────────────────────────────

    #[test]
    fn css_px_converts_to_pt() {
        assert_eq!(css_length_to_typst("100px"), Some("75.0pt".to_string()));
    }

    #[test]
    fn css_px_converts_float() {
        // 96px → 72pt (×0.75)
        assert_eq!(css_length_to_typst("96px"), Some("72.0pt".to_string()));
    }

    #[test]
    fn css_bare_integer_treated_as_pixels() {
        assert_eq!(css_length_to_typst("200"), Some("150.0pt".to_string()));
    }

    #[test]
    fn css_percent_passes_through() {
        assert_eq!(css_length_to_typst("50%"), Some("50%".to_string()));
        assert_eq!(css_length_to_typst("100%"), Some("100%".to_string()));
    }

    #[test]
    fn css_em_passes_through() {
        assert_eq!(css_length_to_typst("2.5em"), Some("2.5em".to_string()));
    }

    #[test]
    fn css_rem_converts_to_em() {
        assert_eq!(css_length_to_typst("1.5rem"), Some("1.5em".to_string()));
    }

    #[test]
    fn css_pt_passes_through() {
        assert_eq!(css_length_to_typst("12pt"), Some("12pt".to_string()));
    }

    #[test]
    fn css_mm_passes_through() {
        assert_eq!(css_length_to_typst("20mm"), Some("20mm".to_string()));
    }

    #[test]
    fn css_cm_passes_through() {
        assert_eq!(css_length_to_typst("5cm"), Some("5cm".to_string()));
    }

    #[test]
    fn css_empty_returns_none() {
        assert_eq!(css_length_to_typst(""), None);
    }

    #[test]
    fn css_invalid_returns_none() {
        assert_eq!(css_length_to_typst("auto"), None);
    }

    // ── format_image_typst ────────────────────────────────────────────────

    fn make_info(path: &str, is_svg: bool) -> ImageInfo {
        ImageInfo {
            path: PathBuf::from(path),
            is_svg,
            natural_width: None,
            natural_height: None,
        }
    }

    #[test]
    fn typst_png_no_alt() {
        let info = make_info("/tmp/test.png", false);
        let out = format_image_typst(&info, "");
        assert!(out.contains("image(\"/tmp/test.png\""), "got: {out}");
        assert!(out.contains("width: 100%"), "got: {out}");
        assert!(!out.contains("caption"), "got: {out}");
        assert!(!out.contains("format:"), "got: {out}");
    }

    #[test]
    fn typst_png_with_alt_becomes_caption() {
        let info = make_info("/tmp/test.png", false);
        let out = format_image_typst(&info, "A diagram");
        assert!(out.contains("caption: [A diagram]"), "got: {out}");
    }

    #[test]
    fn typst_svg_gets_format_arg() {
        let info = make_info("/tmp/logo.svg", true);
        let out = format_image_typst(&info, "");
        assert!(out.contains("format: \"svg\""), "got: {out}");
        assert!(out.contains("image("), "got: {out}");
    }

    #[test]
    fn typst_with_width_hint_uses_it() {
        let info = make_info("/tmp/test.png", false);
        let hint = SizeHint {
            width: Some("50%".to_string()),
            height: None,
        };
        let out = format_image_typst_sized(&info, "", &hint);
        assert!(out.contains("width: 50%"), "got: {out}");
        assert!(!out.contains("height:"), "got: {out}");
    }

    #[test]
    fn typst_with_height_hint_emitted() {
        let info = make_info("/tmp/test.png", false);
        let hint = SizeHint {
            width: Some("100%".to_string()),
            height: Some("200pt".to_string()),
        };
        let out = format_image_typst_sized(&info, "", &hint);
        assert!(out.contains("width: 100%"), "got: {out}");
        assert!(out.contains("height: 200pt"), "got: {out}");
    }

    #[test]
    fn typst_svg_with_size_hint() {
        let info = make_info("/path/to/diagram.svg", true);
        let hint = SizeHint {
            width: Some("75%".to_string()),
            height: None,
        };
        let out = format_image_typst_sized(&info, "Flowchart", &hint);
        assert!(out.contains("format: \"svg\""), "got: {out}");
        assert!(out.contains("width: 75%"), "got: {out}");
        assert!(out.contains("caption: [Flowchart]"), "got: {out}");
    }

    #[test]
    fn typst_alt_special_chars_escaped() {
        let info = make_info("/img.png", false);
        // Special Typst chars in alt text must be escaped
        let out = format_image_typst(&info, "a #hash [bracket] & {brace}");
        assert!(out.contains("\\#hash"), "got: {out}");
        assert!(out.contains("\\[bracket\\]"), "got: {out}");
        assert!(out.contains("\\{brace\\}"), "got: {out}");
    }

    // ── missing_image_fallback ────────────────────────────────────────────

    #[test]
    fn fallback_with_alt() {
        let out = missing_image_fallback("https://example.com/img.png", "My diagram");
        assert!(out.contains("My diagram"), "got: {out}");
        assert!(out.contains("#block("), "got: {out}");
        assert!(out.contains("luma("), "got: {out}");
    }

    #[test]
    fn fallback_no_alt_uses_filename() {
        let out = missing_image_fallback("https://cdn.example.com/photo.jpg", "");
        assert!(out.contains("photo.jpg"), "got: {out}");
    }

    #[test]
    fn fallback_no_alt_strips_query() {
        let out = missing_image_fallback("https://cdn.example.com/img.png?v=1&size=lg", "");
        // Should include base filename only
        assert!(out.contains("img.png"), "got: {out}");
        assert!(!out.contains("v=1"), "got: {out}");
    }

    #[test]
    fn fallback_empty_url_empty_alt() {
        let out = missing_image_fallback("", "");
        assert!(out.contains("#block("), "got: {out}");
    }

    #[test]
    fn fallback_special_chars_escaped() {
        let out = missing_image_fallback("img.png", "a [b] #c");
        assert!(out.contains("\\[b\\]"), "got: {out}");
        assert!(out.contains("\\#c"), "got: {out}");
    }

    // ── cache metadata ────────────────────────────────────────────────────

    #[test]
    fn cache_meta_roundtrip() {
        let dir = TempDir::new().unwrap();
        let meta_path = dir.path().join("test.meta");
        let content = "etag=W/\"abc123\"\nlast_modified=Mon, 01 Jan 2024 00:00:00 GMT\next=png\n";
        fs::write(&meta_path, content).unwrap();
        let meta = read_cache_meta(&meta_path).unwrap();
        assert_eq!(meta.etag, "W/\"abc123\"");
        assert_eq!(meta.last_modified, "Mon, 01 Jan 2024 00:00:00 GMT");
        assert_eq!(meta.ext, "png");
    }

    #[test]
    fn cache_meta_missing_ext_returns_none() {
        let dir = TempDir::new().unwrap();
        let meta_path = dir.path().join("test.meta");
        fs::write(&meta_path, "etag=foo\n").unwrap();
        assert!(read_cache_meta(&meta_path).is_none());
    }

    #[test]
    fn cache_meta_nonexistent_returns_none() {
        let dir = TempDir::new().unwrap();
        let meta_path = dir.path().join("nonexistent.meta");
        assert!(read_cache_meta(&meta_path).is_none());
    }

    // ── base64 decoder ────────────────────────────────────────────────────

    #[test]
    fn base64_decode_hello() {
        // "Hello" = SGVsbG8=
        let out = decode_base64("SGVsbG8=").unwrap();
        assert_eq!(out, b"Hello");
    }

    #[test]
    fn base64_decode_no_padding() {
        // "Man" = TWFu (no padding needed)
        let out = decode_base64("TWFu").unwrap();
        assert_eq!(out, b"Man");
    }

    #[test]
    fn base64_decode_with_newlines() {
        // base64 with line breaks (as in data URIs)
        let encoded = "SGVs\nbG8=";
        let out = decode_base64(encoded).unwrap();
        assert_eq!(out, b"Hello");
    }

    #[test]
    fn base64_decode_empty() {
        let out = decode_base64("").unwrap();
        assert!(out.is_empty());
    }

    // ── resolve_image (local) ─────────────────────────────────────────────

    fn make_renderer_in(dir: &TempDir) -> TypstRenderer {
        let _ = fs::create_dir_all(dir.path().join("cache"));
        TypstRenderer {
            asset_root: dir.path().to_path_buf(),
            cache_dir: dir.path().join("cache"),
            list_stack: Vec::new(),
            ordered_start_stack: Vec::new(),
            ordered_counter_stack: Vec::new(),
            list_tight_stack: Vec::new(),
            list_depth: 0,
            syntax_theme: "InspiredGitHub".to_string(),
            mono_font: "DejaVu Sans Mono".to_string(),
            html_inline_stack: Vec::new(),
            no_remote_images: false,
            footnote_defs: HashMap::new(),
            footnote_refs: Vec::new(),
            heading_labels_seen: HashMap::new(),
        }
    }

    fn test_config(dir: &TempDir) -> RenderConfig {
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
            toc_depth: 3,
            no_remote_images: true,
            cache_dir_override: None,
        }
    }

    #[test]
    fn resolve_local_png_exists() {
        let dir = TempDir::new().unwrap();
        let img_path = dir.path().join("photo.png");
        fs::write(&img_path, b"\x89PNG\r\n\x1a\n").unwrap();
        let r = make_renderer_in(&dir);
        let info = r.resolve_image("photo.png").unwrap();
        assert_eq!(info.path, img_path);
        assert!(!info.is_svg);
    }

    #[test]
    fn resolve_local_svg_sets_is_svg() {
        let dir = TempDir::new().unwrap();
        let svg_path = dir.path().join("icon.svg");
        fs::write(&svg_path, b"<svg></svg>").unwrap();
        let r = make_renderer_in(&dir);
        let info = r.resolve_image("icon.svg").unwrap();
        assert!(info.is_svg);
    }

    #[test]
    fn resolve_local_missing_returns_error() {
        let dir = TempDir::new().unwrap();
        let r = make_renderer_in(&dir);
        let result = r.resolve_image("nonexistent.png");
        assert!(result.is_err(), "expected error for missing local image");
    }

    #[test]
    fn resolve_remote_skipped_when_disabled() {
        let dir = TempDir::new().unwrap();
        let mut r = make_renderer_in(&dir);
        r.no_remote_images = true;
        let result = r.resolve_image("https://example.com/photo.png");
        assert!(
            result.is_err(),
            "expected error when remote images disabled"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("remote images disabled") || msg.contains("--no-remote-images"),
            "unexpected message: {msg}"
        );
    }

    // ── data URI round-trip ───────────────────────────────────────────────

    #[test]
    fn resolve_data_uri_png() {
        let dir = TempDir::new().unwrap();
        let r = make_renderer_in(&dir);
        // Minimal 1×1 white PNG, base64-encoded
        let b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let uri = format!("data:image/png;base64,{}", b64);
        let info = r.resolve_data_uri(&uri).unwrap();
        assert!(!info.is_svg);
        assert!(info.path.exists(), "cached file should exist");
        assert_eq!(info.path.extension().unwrap().to_str().unwrap(), "png");
    }

    #[test]
    fn resolve_data_uri_svg() {
        let dir = TempDir::new().unwrap();
        let r = make_renderer_in(&dir);
        let svg_b64 = {
            let svg = b"<svg xmlns='http://www.w3.org/2000/svg'></svg>";
            // Manual base64 encode
            base64_encode(svg)
        };
        let uri = format!("data:image/svg+xml;base64,{}", svg_b64);
        let info = r.resolve_data_uri(&uri).unwrap();
        assert!(info.is_svg, "SVG data URI should set is_svg=true");
        assert_eq!(info.path.extension().unwrap().to_str().unwrap(), "svg");
    }

    // Helper: simple base64 encoder for test use only.
    fn base64_encode(input: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in input.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let combined = (b0 << 16) | (b1 << 8) | b2;
            out.push(ALPHABET[((combined >> 18) & 0x3f) as usize] as char);
            out.push(ALPHABET[((combined >> 12) & 0x3f) as usize] as char);
            if chunk.len() > 1 {
                out.push(ALPHABET[((combined >> 6) & 0x3f) as usize] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(ALPHABET[(combined & 0x3f) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }

    // ── markdown round-trips (no network needed) ──────────────────────────

    #[test]
    fn md_missing_local_image_emits_fallback_not_error() {
        // The renderer should not bubble an error when a local image is missing;
        // it should emit the fallback placeholder box and continue.
        let dir = TempDir::new().unwrap();

        // Case 1: alt text provided — fallback label uses alt text.
        let md = "# Test\n\n![Missing image](nonexistent.png)\n\nSome text.\n";
        let config = test_config(&dir);
        let typst_src = crate::renderer::markdown_to_typst_pub(md, &config).unwrap();
        assert!(
            typst_src.contains("\\[Image:"),
            "expected fallback placeholder, got:\n{typst_src}"
        );
        // When alt text is present, the fallback uses the alt text as the label.
        assert!(
            typst_src.contains("Missing image"),
            "should use alt text in fallback:\n{typst_src}"
        );

        // Case 2: no alt text — fallback label uses filename.
        let md2 = "# Test\n\n![](nonexistent.png)\n\nSome text.\n";
        let typst_src2 = crate::renderer::markdown_to_typst_pub(md2, &config).unwrap();
        assert!(
            typst_src2.contains("\\[Image:"),
            "expected fallback placeholder, got:\n{typst_src2}"
        );
        assert!(
            typst_src2.contains("nonexistent.png"),
            "should include filename in fallback:\n{typst_src2}"
        );
    }

    #[test]
    fn md_local_svg_emits_format_svg() {
        let dir = TempDir::new().unwrap();
        let svg_path = dir.path().join("diagram.svg");
        fs::write(&svg_path, b"<svg></svg>").unwrap();
        let md = "# Test\n\n![A diagram](diagram.svg)\n";
        let config = test_config(&dir);
        let typst_src = crate::renderer::markdown_to_typst_pub(md, &config).unwrap();
        assert!(
            typst_src.contains("format: \"svg\""),
            "expected SVG format arg, got:\n{typst_src}"
        );
    }

    #[test]
    fn md_image_alt_becomes_caption() {
        let dir = TempDir::new().unwrap();
        let img_path = dir.path().join("chart.png");
        fs::write(&img_path, b"\x89PNG\r\n\x1a\n").unwrap();
        let md = "![Quarterly results](chart.png)\n";
        let config = test_config(&dir);
        let typst_src = crate::renderer::markdown_to_typst_pub(md, &config).unwrap();
        assert!(
            typst_src.contains("caption: [Quarterly results]"),
            "got:\n{typst_src}"
        );
    }

    #[test]
    fn md_image_no_alt_no_caption() {
        let dir = TempDir::new().unwrap();
        let img_path = dir.path().join("photo.png");
        fs::write(&img_path, b"\x89PNG\r\n\x1a\n").unwrap();
        let md = "![](photo.png)\n";
        let config = test_config(&dir);
        let typst_src = crate::renderer::markdown_to_typst_pub(md, &config).unwrap();
        assert!(
            !typst_src.contains("caption"),
            "should have no caption, got:\n{typst_src}"
        );
    }

    #[test]
    fn md_remote_image_skipped_emits_fallback() {
        let dir = TempDir::new().unwrap();
        let md = "![Cloud photo](https://example.com/remote.jpg)\n\nSome text after.\n";
        let config = test_config(&dir);
        let typst_src = crate::renderer::markdown_to_typst_pub(md, &config).unwrap();
        // Should have a fallback block, not an image() call with the https URL
        assert!(
            !typst_src.contains("image(\"https://"),
            "should not emit remote URL, got:\n{typst_src}"
        );
        assert!(
            typst_src.contains("#block("),
            "expected fallback block, got:\n{typst_src}"
        );
        // Body text after the image should still be present
        assert!(
            typst_src.contains("Some text after"),
            "body text missing, got:\n{typst_src}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GFM Fidelity Fixtures
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod gfm_tests {
    use super::*;
    use tempfile::TempDir;

    fn render(md: &str) -> String {
        let dir = TempDir::new().unwrap();
        let config = RenderConfig {
            page_width_mm: 210.0,
            page_height_mm: 297.0,
            margin_mm: 20.0,
            base_font_size_pt: 12.0,
            fonts: FontSet::default(),
            input_path: None,
            syntax_theme: "InspiredGitHub".to_string(),
            toc: false,
            toc_explicit: false,
            toc_depth: 3,
            no_remote_images: true,
            cache_dir_override: Some(dir.path().to_path_buf()),
        };
        markdown_to_typst_pub(md, &config).expect("render failed")
    }

    // ── Nested lists ─────────────────────────────────────────────────────────

    #[test]
    fn nested_bullet_list_indents() {
        let md = "- Item A\n  - Item B\n    - Item C\n- Item D\n";
        let out = render(md);
        // Top-level items have no indent
        assert!(
            out.contains("- Item A"),
            "expected top-level bullet, got:\n{out}"
        );
        assert!(
            out.contains("- Item D"),
            "expected top-level bullet, got:\n{out}"
        );
        // Nested items should be indented (2 spaces per level)
        assert!(
            out.contains("  - Item B"),
            "expected 1-level indent, got:\n{out}"
        );
        assert!(
            out.contains("    - Item C"),
            "expected 2-level indent, got:\n{out}"
        );
    }

    #[test]
    fn nested_ordered_list_indents() {
        let md = "1. First\n   1. Nested one\n   2. Nested two\n2. Second\n";
        let out = render(md);
        assert!(
            out.contains("+ First"),
            "expected ordered marker, got:\n{out}"
        );
        assert!(
            out.contains("+ Second"),
            "expected ordered marker, got:\n{out}"
        );
        // Nested ordered items should be indented
        assert!(
            out.contains("  + Nested one") || out.contains("   + Nested one"),
            "expected indented nested ordered item, got:\n{out}"
        );
    }

    #[test]
    fn mixed_nested_list() {
        let md = "- Bullet\n  1. Ordered sub\n  2. Ordered sub two\n- Another bullet\n";
        let out = render(md);
        assert!(out.contains("- Bullet"), "got:\n{out}");
        assert!(out.contains("- Another bullet"), "got:\n{out}");
        // The nested ordered items should appear indented somewhere
        assert!(
            out.contains("+ Ordered sub"),
            "expected ordered sub-item, got:\n{out}"
        );
    }

    // ── Ordered list numbering ────────────────────────────────────────────────

    #[test]
    fn ordered_list_default_start_from_one() {
        let md = "1. Alpha\n2. Beta\n3. Gamma\n";
        let out = render(md);
        // Typst uses `+` for auto-numbered ordered lists
        assert!(out.contains("+ Alpha"), "got:\n{out}");
        assert!(out.contains("+ Beta"), "got:\n{out}");
        assert!(out.contains("+ Gamma"), "got:\n{out}");
    }

    #[test]
    fn ordered_list_non_one_start() {
        // GFM spec: list starting at 3 should preserve that start number
        let md = "3. Third\n4. Fourth\n";
        let out = render(md);
        // Items are still rendered as Typst ordered markers
        assert!(out.contains("+ Third"), "got:\n{out}");
        assert!(out.contains("+ Fourth"), "got:\n{out}");
    }

    #[test]
    fn ordered_list_markers_not_bullet() {
        let md = "1. One\n2. Two\n";
        let out = render(md);
        // Must use `+` not `-`
        assert!(out.contains("+ One"), "expected `+` marker, got:\n{out}");
        assert!(
            !out.contains("- One"),
            "should not use `-` for ordered, got:\n{out}"
        );
    }

    // ── Task lists ────────────────────────────────────────────────────────────

    #[test]
    fn task_list_unchecked() {
        let md = "- [ ] Task one\n- [ ] Task two\n";
        let out = render(md);
        // Unchecked checkbox glyph
        assert!(
            out.contains("☐"),
            "expected ☐ for unchecked task, got:\n{out}"
        );
        assert!(out.contains("Task one"), "got:\n{out}");
    }

    #[test]
    fn task_list_checked() {
        let md = "- [x] Done task\n- [X] Also done\n";
        let out = render(md);
        // Checked checkbox glyph
        assert!(
            out.contains("☑"),
            "expected ☑ for checked task, got:\n{out}"
        );
        assert!(out.contains("Done task"), "got:\n{out}");
    }

    #[test]
    fn task_list_mixed() {
        let md = "- [x] Complete\n- [ ] Incomplete\n";
        let out = render(md);
        assert!(out.contains("☑"), "expected checked box, got:\n{out}");
        assert!(out.contains("☐"), "expected unchecked box, got:\n{out}");
        assert!(out.contains("Complete"), "got:\n{out}");
        assert!(out.contains("Incomplete"), "got:\n{out}");
    }

    #[test]
    fn task_list_uses_bullet_not_ordered_marker() {
        let md = "- [x] Done\n";
        let out = render(md);
        // Task items always use `- ` bullet marker (not `+ `)
        assert!(
            out.contains("- "),
            "expected bullet marker for task item, got:\n{out}"
        );
    }

    // ── Table alignment markers ───────────────────────────────────────────────

    #[test]
    fn table_left_align() {
        let md = "| Name |\n|:-----|\n| Alice |\n";
        let out = render(md);
        assert!(out.contains("#table("), "got:\n{out}");
        assert!(
            out.contains("align: left"),
            "expected left alignment, got:\n{out}"
        );
    }

    #[test]
    fn table_right_align() {
        let md = "| Price |\n|------:|\n| 9.99 |\n";
        let out = render(md);
        assert!(
            out.contains("align: right"),
            "expected right alignment, got:\n{out}"
        );
    }

    #[test]
    fn table_center_align() {
        let md = "| Status |\n|:------:|\n| OK |\n";
        let out = render(md);
        assert!(
            out.contains("align: center"),
            "expected center alignment, got:\n{out}"
        );
    }

    #[test]
    fn table_mixed_alignment() {
        let md = "| Left | Center | Right |\n|:-----|:------:|------:|\n| a | b | c |\n";
        let out = render(md);
        assert!(out.contains("align: left"), "expected left, got:\n{out}");
        assert!(
            out.contains("align: center"),
            "expected center, got:\n{out}"
        );
        assert!(out.contains("align: right"), "expected right, got:\n{out}");
    }

    #[test]
    fn table_header_row_is_bold() {
        let md = "| Col1 | Col2 |\n|------|------|\n| a | b |\n";
        let out = render(md);
        // Header row cells should be wrapped in #strong[…]
        assert!(
            out.contains("#strong["),
            "expected header bold, got:\n{out}"
        );
    }

    #[test]
    fn table_has_fill_spec() {
        let md = "| H |\n|---|\n| v |\n";
        let out = render(md);
        // fill: (col, row) => … should be present
        assert!(out.contains("fill:"), "expected fill spec, got:\n{out}");
    }

    #[test]
    fn table_column_count_equals_headers() {
        let md = "| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n";
        let out = render(md);
        // columns: (1fr, 1fr, 1fr)
        assert!(
            out.contains("1fr, 1fr, 1fr"),
            "expected 3 fractional columns, got:\n{out}"
        );
    }

    // ── Autolinks ────────────────────────────────────────────────────────────

    #[test]
    fn autolink_bare_url() {
        // GFM autolink: bare URL becomes a link with URL as label
        let md = "Visit https://example.com for more.\n";
        let out = render(md);
        assert!(out.contains("#link("), "expected link, got:\n{out}");
        assert!(out.contains("https://example.com"), "got:\n{out}");
    }

    #[test]
    fn autolink_angle_bracket() {
        let md = "See <https://example.org>.\n";
        let out = render(md);
        assert!(out.contains("#link("), "got:\n{out}");
        assert!(out.contains("https://example.org"), "got:\n{out}");
    }

    #[test]
    fn autolink_compact_form_when_label_equals_url() {
        // When the label matches the URL, we emit the compact #link(url) form.
        let md = "<https://rust-lang.org>\n";
        let out = render(md);
        // Should be compact: #link("https://rust-lang.org") not #link("...", [...])
        assert!(
            out.contains("#link(\"https://rust-lang.org\")"),
            "expected compact link, got:\n{out}"
        );
    }

    #[test]
    fn explicit_link_keeps_label() {
        let md = "[Rust](https://rust-lang.org)\n";
        let out = render(md);
        // Has label, so should use #link(url, [label]) form
        assert!(
            out.contains("#link(\"https://rust-lang.org\", [Rust])"),
            "got:\n{out}"
        );
    }

    // ── Footnotes ────────────────────────────────────────────────────────────

    #[test]
    fn footnote_reference_becomes_superscript() {
        let md = "Text with a footnote.[^1]\n\n[^1]: The footnote body.\n";
        let out = render(md);
        // Reference becomes superscript
        assert!(
            out.contains("#super["),
            "expected superscript for footnote ref, got:\n{out}"
        );
    }

    #[test]
    fn footnote_definition_emitted_at_end() {
        let md = "Here[^note].\n\n[^note]: This is the note.\n";
        let out = render(md);
        // Definition body should appear in output (after the separator line)
        assert!(
            out.contains("This is the note."),
            "expected footnote body, got:\n{out}"
        );
        // A separator line should precede the footnotes
        assert!(
            out.contains("#line(length: 100%)"),
            "expected footnote separator line, got:\n{out}"
        );
    }

    #[test]
    fn multiple_footnotes_all_appear() {
        let md = "First[^a] and second[^b].\n\n[^a]: Note A.\n\n[^b]: Note B.\n";
        let out = render(md);
        assert!(out.contains("Note A."), "got:\n{out}");
        assert!(out.contains("Note B."), "got:\n{out}");
    }

    #[test]
    fn document_without_footnotes_has_no_separator() {
        let md = "Just plain text.\n";
        let out = render(md);
        // Only one line separator max (from ThematicBreak), not footnote separator
        // Count occurrences of #line — with no footnotes there should be 0
        let count = out.matches("#line(length: 100%)").count();
        assert_eq!(count, 0, "unexpected footnote separator in:\n{out}");
    }

    // ── Definition (description) lists ───────────────────────────────────────

    #[test]
    fn definition_list_term_is_bold() {
        let md = "Apple\n:   A fruit.\n";
        let out = render(md);
        assert!(out.contains("#strong["), "expected bold term, got:\n{out}");
        assert!(out.contains("Apple"), "got:\n{out}");
    }

    #[test]
    fn definition_list_details_are_indented() {
        let md = "Term\n:   Definition text here.\n";
        let out = render(md);
        assert!(
            out.contains("#pad(left:"),
            "expected indented details, got:\n{out}"
        );
        assert!(out.contains("Definition text here."), "got:\n{out}");
    }

    #[test]
    fn definition_list_multiple_terms() {
        let md = "Cat\n:   A domesticated feline.\n\nDog\n:   A domesticated canine.\n";
        let out = render(md);
        assert!(out.contains("Cat"), "got:\n{out}");
        assert!(out.contains("Dog"), "got:\n{out}");
        assert!(out.contains("feline"), "got:\n{out}");
        assert!(out.contains("canine"), "got:\n{out}");
    }

    // ── Combined GFM fixture ──────────────────────────────────────────────────

    #[test]
    fn full_gfm_fixture_roundtrip() {
        let md = r#"# GFM Fixture

## Task List

- [x] Done
- [ ] Pending
  - [ ] Sub-task

## Ordered List

1. First
2. Second
   1. Nested A
   2. Nested B
3. Third

## Table with Alignment

| Item | Qty | Price |
|:-----|:---:|------:|
| Widget | 10 | 9.99 |
| Gadget |  1 | 49.99 |

## Autolinks

Visit <https://example.com> or https://rust-lang.org for info.

## Footnote

This sentence has a note.[^fn1]

[^fn1]: Footnote body text.

## Definition List

Markdown
:   A lightweight markup language.

Typst
:   A modern typesetting system.
"#;
        let out = render(md);

        // Headings
        assert!(out.contains("= GFM Fixture"), "heading, got:\n{out}");

        // Task list
        assert!(out.contains("☑"), "checked box, got:\n{out}");
        assert!(out.contains("☐"), "unchecked box, got:\n{out}");

        // Ordered list
        assert!(out.contains("+ First"), "ordered, got:\n{out}");
        assert!(out.contains("+ Third"), "ordered, got:\n{out}");

        // Table with alignment
        assert!(out.contains("#table("), "table, got:\n{out}");
        assert!(
            out.contains("align: right"),
            "right-align price col, got:\n{out}"
        );
        assert!(
            out.contains("align: center"),
            "center-align qty col, got:\n{out}"
        );

        // Autolinks
        assert!(out.contains("https://example.com"), "autolink, got:\n{out}");

        // Footnote
        assert!(out.contains("#super["), "footnote ref, got:\n{out}");
        assert!(
            out.contains("Footnote body text."),
            "footnote body, got:\n{out}"
        );

        // Definition list
        assert!(
            out.contains("#strong["),
            "definition term bold, got:\n{out}"
        );
        assert!(out.contains("Markdown"), "definition term, got:\n{out}");
        assert!(
            out.contains("lightweight markup language"),
            "definition detail, got:\n{out}"
        );
    }
}
