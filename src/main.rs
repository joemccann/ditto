// The binary re-uses the library modules. Declaring them here (instead of in
// lib.rs) would cause duplicate compilation, so we use the lib crate.
use md_to_pdf::cli::Cli;
use md_to_pdf::renderer::{FontSet, RenderConfig, read_input, render_markdown_to_pdf};

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // --doctor: self-check mode (exits after printing report)
    if cli.doctor {
        md_to_pdf::doctor::run()?;
        return Ok(());
    }

    // Positional args are required_unless_present = "doctor", so unwrap is safe.
    let input = cli.input.as_deref().unwrap_or("-");
    let output = cli.output.as_ref().expect("OUTPUT is required");

    let markdown_content = read_input(input)?;

    let input_path = if input == "-" {
        None
    } else {
        Some(std::path::PathBuf::from(input))
    };

    let layout = cli.resolved_layout();

    let summary = render_markdown_to_pdf(
        &markdown_content,
        output,
        RenderConfig {
            page_width_mm: layout.page_width_mm,
            page_height_mm: layout.page_height_mm,
            margin_mm: layout.margin_mm,
            base_font_size_pt: cli.font_size,
            fonts: FontSet {
                regular: cli.font_family.clone(),
                monospace: cli.mono_font_family.clone(),
            },
            input_path,
            syntax_theme: cli.syntax_theme.clone(),
            toc: cli.emit_toc(),
            toc_explicit: cli.toc_was_explicit(),
            toc_depth: cli.toc_depth,
            no_remote_images: cli.no_remote_images,
            cache_dir_override: cli.cache_dir.clone(),
        },
    )?;

    eprintln!(
        "✓ Successfully converted to PDF: {} ({} page(s), {} TOC entries)",
        output.display(),
        summary.pages,
        summary.toc_entries.len()
    );
    Ok(())
}
