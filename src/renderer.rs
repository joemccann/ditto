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
    syntax_theme: String,
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
                if math.display_math {
                    Ok(format!("$ {} $\n\n", math.literal))
                } else {
                    Ok(format!("$ {} $", math.literal))
                }
            }
            NodeValue::HtmlInline(html) => Ok(self.handle_html_inline(html)),
            NodeValue::HtmlBlock(html) => {
                let rendered = block_html_to_typst(&html.literal);
                if rendered.is_empty() { Ok(String::new()) } else { Ok(rendered) }
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
    /// We maintain `html_inline_stack` across calls so opening tags push a Typst
    /// prefix and closing tags pop+emit the matching suffix.
    fn handle_html_inline(&mut self, html: &str) -> String {
        match parse_inline_tag(html) {
            // Void elements (self-closing by definition)
            InlineTag::SelfClose { ref name } | InlineTag::Open { ref name, .. }
                if is_void_inline(name) =>
            {
                match name.as_str() {
                    "br" | "wbr" => "\\
".to_string(),
                    "hr" => "#line(length: 100%)

".to_string(),
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
