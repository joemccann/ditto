mod cli;
mod renderer;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use cli::Cli;
use renderer::{FontSet, RenderConfig, read_input, render_markdown_to_pdf};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let markdown_content = read_input(&cli.input)?;

    let input_path = if cli.input == "-" {
        None
    } else {
        Some(PathBuf::from(&cli.input))
    };

    let summary = render_markdown_to_pdf(
        &markdown_content,
        &cli.output,
        RenderConfig {
            page_width_mm: cli.page_width,
            page_height_mm: cli.page_height,
            margin_mm: cli.margin,
            base_font_size_pt: cli.font_size,
            fonts: FontSet::default(),
            input_path,
            syntax_theme: cli.syntax_theme,
        },
    )?;

    eprintln!(
        "✓ Successfully converted to PDF: {} ({} page(s), {} TOC entries)",
        cli.output.display(),
        summary.pages,
        summary.toc_entries.len()
    );
    Ok(())
}
