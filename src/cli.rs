use clap::{ArgAction, Parser, ValueEnum};
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// Page / layout presets
// ─────────────────────────────────────────────────────────────────────────────

/// Named page/layout presets that bundle common page sizes and margin defaults.
///
/// Individual `--page-width`, `--page-height`, and `--margin` flags always
/// override the values that a preset provides.
#[derive(Debug, Clone, ValueEnum)]
pub enum Preset {
    /// A4 portrait — 210 × 297 mm, 20 mm margins (default)
    A4,
    /// US Letter portrait — 216 × 279 mm, 20 mm margins
    Letter,
    /// A5 pocket — 148 × 210 mm, 15 mm margins
    A5,
    /// US Legal — 216 × 356 mm, 20 mm margins
    Legal,
    /// 16 : 9 presentation slide deck — 338 × 190 mm, 12 mm margins
    Slides,
}

pub struct PresetValues {
    pub width_mm: f32,
    pub height_mm: f32,
    pub margin_mm: f32,
}

impl Preset {
    pub fn values(&self) -> PresetValues {
        match self {
            Preset::A4 => PresetValues {
                width_mm: 210.0,
                height_mm: 297.0,
                margin_mm: 20.0,
            },
            Preset::Letter => PresetValues {
                width_mm: 215.9,
                height_mm: 279.4,
                margin_mm: 20.0,
            },
            Preset::A5 => PresetValues {
                width_mm: 148.0,
                height_mm: 210.0,
                margin_mm: 15.0,
            },
            Preset::Legal => PresetValues {
                width_mm: 215.9,
                height_mm: 355.6,
                margin_mm: 20.0,
            },
            Preset::Slides => PresetValues {
                width_mm: 338.0,
                height_mm: 190.0,
                margin_mm: 12.0,
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CLI definition
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
#[command(
    name = "md-to-pdf",
    version,
    about = "Convert Markdown to PDF with full GFM, math, and syntax-highlighting support.",
    long_about = "\
md-to-pdf converts Markdown (CommonMark + GitHub Flavored Markdown) to PDF using a \
pure-Rust Typst engine.  The tool supports tables, task lists, fenced code blocks \
with syntax highlighting, LaTeX math ($…$ and $$…$$), footnotes, remote image \
caching, and an auto-generated table of contents.

EXAMPLES

  # Basic conversion
  md-to-pdf README.md README.pdf

  # US-Letter, 14 pt font, custom body + mono fonts
  md-to-pdf --preset letter --font-size 14 \\
            --font-family \"EB Garamond\" --mono-font-family \"Fira Code\" \\
            report.md report.pdf

  # Presentation slide deck
  md-to-pdf --preset slides --no-toc slides.md slides.pdf

  # Dark code theme, skip remote images, custom cache
  md-to-pdf --syntax-theme \"base16-ocean.dark\" --no-remote-images \\
            --cache-dir /tmp/mdcache input.md out.pdf

  # Table of contents — headings 1 and 2 only
  md-to-pdf --toc --toc-depth 2 big-doc.md big-doc.pdf

  # Check your environment (fonts, Typst engine, …)
  md-to-pdf --doctor

  # Read from stdin
  cat README.md | md-to-pdf - output.pdf",
    after_help = "SYNTAX-HIGHLIGHTING THEMES

  InspiredGitHub (default), base16-ocean.dark, base16-ocean.light,
  base16-eighties.dark, base16-mocha.dark, Solarized (dark), Solarized (light)

TIP: Run `md-to-pdf --doctor` to verify that required fonts are available \
and that the Typst engine is working correctly."
)]
pub struct Cli {
    // ── Positional ──────────────────────────────────────────────────────────
    /// Input Markdown file.  Pass `-` to read from stdin.
    #[arg(value_name = "INPUT", required_unless_present = "doctor")]
    pub input: Option<String>,

    /// Destination PDF file path.
    #[arg(value_name = "OUTPUT", required_unless_present = "doctor")]
    pub output: Option<PathBuf>,

    // ── Page layout ─────────────────────────────────────────────────────────
    /// Named page preset.  Bundles width, height, and a sensible margin.
    /// Individual dimension flags always override the preset.
    ///
    /// Presets: a4 (default), letter, a5, legal, slides
    #[arg(long, value_name = "PRESET", value_enum, default_value = "a4")]
    pub preset: Preset,

    /// Page width in millimetres.  Overrides the preset.
    #[arg(short = 'w', long, value_name = "MM")]
    pub page_width: Option<f32>,

    /// Page height in millimetres.  Overrides the preset.
    #[arg(long, value_name = "MM")]
    pub page_height: Option<f32>,

    /// Page margin in millimetres (applied to all four sides).  Overrides the preset.
    #[arg(short, long, value_name = "MM")]
    pub margin: Option<f32>,

    // ── Typography ──────────────────────────────────────────────────────────
    /// Base font size in points.
    #[arg(short = 'f', long, value_name = "PT", default_value = "12.0")]
    pub font_size: f32,

    /// Body font family name (must be installed on the system or embedded by
    /// typst-kit).  Enclose multi-word names in quotes.
    ///
    /// Examples: "Libertinus Serif" (default), "EB Garamond", "Source Serif 4",
    ///           "New Computer Modern", "Linux Libertine"
    #[arg(long, value_name = "FAMILY", default_value = "Libertinus Serif")]
    pub font_family: String,

    /// Monospace font family used for code blocks and inline code.
    ///
    /// Examples: "DejaVu Sans Mono" (default), "Fira Code", "JetBrains Mono",
    ///           "Inconsolata", "Source Code Pro"
    #[arg(long, value_name = "FAMILY", default_value = "DejaVu Sans Mono")]
    pub mono_font_family: String,

    /// Syntect syntax-highlighting theme for fenced code blocks.
    /// `--theme` is a convenient short alias for `--syntax-theme`.
    ///
    /// Built-in themes: InspiredGitHub (default), base16-ocean.dark,
    /// base16-ocean.light, base16-eighties.dark, base16-mocha.dark,
    /// Solarized (dark), Solarized (light)
    #[arg(
        long,
        alias = "theme",
        value_name = "THEME",
        default_value = "InspiredGitHub"
    )]
    pub syntax_theme: String,

    // ── Table of contents ───────────────────────────────────────────────────
    /// Emit a Table of Contents page before the document body.
    /// Headings up to `--toc-depth` levels deep are included.
    #[arg(long, overrides_with = "no_toc", default_value = "false")]
    pub toc: bool,

    /// Suppress the Table of Contents even if the document has headings.
    #[arg(long, overrides_with = "toc", action = ArgAction::SetTrue)]
    pub no_toc: bool,

    /// Maximum heading depth included in the Table of Contents (1–6).
    ///
    /// `--toc-depth 2` includes only H1 and H2 headings.
    #[arg(long, value_name = "DEPTH", default_value = "6",
          value_parser = clap::value_parser!(u8).range(1..=6))]
    pub toc_depth: u8,

    // ── Image handling ──────────────────────────────────────────────────────
    /// Skip downloading remote images (http / https).
    /// Placeholders are left in place of remote image references.
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_remote_images: bool,

    /// Directory used for cached remote images.
    /// Defaults to `.md-to-pdf-cache/` next to the input file (or the
    /// current working directory when reading from stdin).
    #[arg(long, value_name = "DIR")]
    pub cache_dir: Option<PathBuf>,

    // ── Doctor ──────────────────────────────────────────────────────────────
    /// Run a self-check that verifies the Typst engine, font availability,
    /// network access, and cache writability, then exit.
    #[arg(long, action = ArgAction::SetTrue)]
    pub doctor: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Resolved config helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Final, resolved page dimensions after merging preset + overrides.
pub struct ResolvedLayout {
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub margin_mm: f32,
}

impl Cli {
    /// Resolve page dimensions: preset first, then any explicit flag overrides.
    pub fn resolved_layout(&self) -> ResolvedLayout {
        let base = self.preset.values();
        ResolvedLayout {
            page_width_mm: self.page_width.unwrap_or(base.width_mm),
            page_height_mm: self.page_height.unwrap_or(base.height_mm),
            margin_mm: self.margin.unwrap_or(base.margin_mm),
        }
    }

    /// Whether the TOC should be rendered.  `--toc` wins over `--no-toc`; if
    /// neither is supplied the default is `false` (no TOC).
    pub fn emit_toc(&self) -> bool {
        // `overrides_with` already handles mutual exclusion; just use `toc`.
        self.toc && !self.no_toc
    }

    /// Returns true when the user explicitly passed `--toc` or `--no-toc`.
    /// When neither flag was given, frontmatter in the document can override
    /// the default.
    pub fn toc_was_explicit(&self) -> bool {
        self.toc || self.no_toc
    }
}
