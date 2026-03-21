use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "md-to-pdf")]
#[command(about = "Convert Markdown to PDF", long_about = None)]
pub struct Cli {
    /// Input markdown file (use - for stdin)
    #[arg(value_name = "INPUT")]
    pub input: String,

    /// Output PDF file
    #[arg(value_name = "OUTPUT")]
    pub output: PathBuf,

    /// Page width in mm (default: 210mm - A4)
    #[arg(short = 'w', long, default_value = "210.0")]
    pub page_width: f32,

    /// Page height in mm (default: 297mm - A4)
    #[arg(long, default_value = "297.0")]
    pub page_height: f32,

    /// Margin in mm (default: 20mm)
    #[arg(short, long, default_value = "20.0")]
    pub margin: f32,

    /// Base font size in points
    #[arg(short, long, default_value = "12.0")]
    pub font_size: f32,

    /// Syntect theme name used for code blocks
    #[arg(long, default_value = "InspiredGitHub")]
    pub syntax_theme: String,
}
