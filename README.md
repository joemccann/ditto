# md-to-pdf

A Rust CLI application that converts Markdown (including GitHub Flavored Markdown) to PDF.

## Features

- ✅ Full CommonMark markdown support
- ✅ GitHub Flavored Markdown (GFM) support:
  - Tables
  - Strikethrough (~strikethrough~)
  - Task lists
  - Fenced code blocks with syntax highlighting hints
- ✅ Basic text formatting (bold, italic, code)
- ✅ Lists (ordered and unordered)
- ✅ Blockquotes
- ✅ Headings (H1-H6)
- ✅ Code blocks with language specification
- ✅ Horizontal rules
- ✅ Customizable page dimensions
- ✅ Customizable font size and margins
- ✅ Support for reading from stdin

## Installation

### From Source

```bash
cargo install --path .
```

### From Binary

Download the latest release from the releases page and add it to your PATH.

## Usage

Basic usage:

```bash
md-to-pdf input.md output.pdf
```

Read from stdin:

```bash
cat input.md | md-to-pdf - output.pdf
echo "# Hello World" | md-to-pdf - output.pdf
```

### Options

```
USAGE:
    md-to-pdf [OPTIONS] <INPUT> <OUTPUT>

ARGUMENTS:
    <INPUT>    Input markdown file (use - for stdin)
    <OUTPUT>   Output PDF file

OPTIONS:
    -w, --page-width <PAGE_WIDTH>      Page width in mm [default: 210.0]
        --page-height <PAGE_HEIGHT>    Page height in mm [default: 297.0]
    -m, --margin <MARGIN>              Margin in mm [default: 20.0]
    -f, --font-size <FONT_SIZE>        Font size in points [default: 12.0]
    -h, --help                         Print help
```

### Examples

A4 size with larger font:

```bash
md-to-pdf -f 14 README.md README.pdf
```

Custom page size (Letter):

```bash
md-to-pdf -w 216 --page-height 279 input.md output.pdf
```

Wide margins:

```bash
md-to-pdf -m 30 input.md output.pdf
```

Pipe from another command:

```bash
pandoc -t markdown input.adoc | md-to-pdf - output.pdf
```

## Limitations

Currently in active development. Some GFM features are partially implemented:

- ⚠️ Links: Text is rendered but URLs are not clickable
- ⚠️ Images: Basic support for image references (full rendering in progress)
- ⚠️ Tables: Basic rendering (improved formatting in progress)
- ⚠️ Text wrapping: Lines may overflow on very long text
- ⚠️ Page breaks: All content on a single page (multi-page support in progress)
- ⚠️ Math rendering: LaTeX math not yet implemented
- ✅ HTML content: Common inline and block HTML tags rendered via Typst mapping

## Roadmap

- [ ] Multi-page support with automatic page breaks
- [ ] Text wrapping and proper line breaking
- [ ] Full image rendering (PNG, JPEG, GIF, WebP, BMP, SVG)
- [ ] Improved table formatting
- [ ] Clickable links
- [ ] Heading numbering
- [ ] Table of contents generation
- [ ] Custom font loading
- [ ] Syntax highlighting for code blocks
- [ ] Math/LaTeX rendering

## Development

### Build

```bash
cargo build --release
```

### Run tests

```bash
cargo test
```

### Run directly

```bash
cargo run -- test.md output.pdf
```

## Dependencies

- `pulldown-cmark` - Markdown parsing with GFM support
- `printpdf` - PDF generation
- `clap` - CLI argument parsing
- `anyhow` - Error handling
- `image` - Image handling (for future image rendering)

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.