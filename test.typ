#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#pagebreak()
= Table of Contents
- Markdown to PDF Test Document
  - Features Supported
    - Basic Text Formatting
    - Lists
    - Blockquote
    - Code
    - Links and Images
    - Horizontal Rule
    - Tables
    - Additional GFM Features
#pagebreak()

= Markdown to PDF Test Document

This is a test document to demonstrate the markdown to PDF converter capabilities.

== Features Supported

=== Basic Text Formatting

This document tests #strong[bold text], #emph[italic text], and #emph[#strong[bold italic]] text. It also tests #strike[strikethrough] text.

=== Lists

Unordered list:

- Item 1
- Item 2

- Nested item 2.1
- Nested item 2.2
- Item 3

Ordered list:

+ First item
+ Second item
+ Third item

Task lists:


=== Blockquote

#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[
This is a blockquote. It can span multiple lines.
]

=== Code

Inline `code` looks like this.

#block(fill: luma(245), inset: 10pt, radius: 4pt)[
#text(fill: luma(90), size: 10pt)[rust]
#text(font: ("DejaVu Sans Mono",), [fn main() \{     println!(\"Hello, world!\"); \} ])
]

#block(fill: luma(245), inset: 10pt, radius: 4pt)[
#text(fill: luma(90), size: 10pt)[python]
#text(font: ("DejaVu Sans Mono",), [def greet(name):     return f\"Hello, \{name\}!\" ])
]

=== Links and Images

#link("https://github.com", [Link to GitHub])

=== Horizontal Rule

#line(length: 100%)

=== Tables

#table(
  columns: 3,
  stroke: luma(180),
  inset: 6pt,
  align: left,
  [Header 1],
  [Header 2],
  [Header 3],
  [Cell 1],
  [Cell 2],
  [Cell 3],
  [Cell 4],
  [Cell 5],
  [Cell 6]
)

=== Additional GFM Features

- Task lists (shown above)
- Strikethrough (shown above)
- Tables (shown above)

That’s all for this test document!


