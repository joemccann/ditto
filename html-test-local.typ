#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#pagebreak()
= Table of Contents
- HTML-in-Markdown Test Document
  - Inline HTML Tags
    - Subscript and Superscript
    - Keyboard / Monospace
    - Semantic
    - Links
    - Span with CSS styles
    - Line break
    - Unknown / pass-through tags
  - Block HTML Elements
  - Mixed Markdown + HTML
  - Entity Decoding
#pagebreak()

= HTML-in-Markdown Test Document

This document exercises raw HTML tags inside Markdown.

#line(length: 100%)

== Inline HTML Tags

Normal paragraph with #strong[bold via b-tag], #strong[bold via strong], #emph[italic via i-tag], #emph[italic via em-tag], and #underline[underline].

Strikethrough: #strike[old price] and #strike[deleted text].

Inserted text: #underline[newly added].

Highlighted text: #highlight[this is highlighted].

Small text: #text(size: 0.8em)[fine print here].

=== Subscript and Superscript

H#sub[2]O is water. E = mc#super[2] is Einstein’s famous equation.

=== Keyboard / Monospace

Press #raw("Ctrl")+#raw("Alt")+#raw("Del") to reset.

Shell output: #raw("Permission denied").

Inline code via HTML: #raw("printf(“Hello\\n”);")

=== Semantic

#emph[The Elements of Style] by Strunk and White.

#emph[HTML] is the backbone of the web.

=== Links

Visit #link("https://www.rust-lang.org", [The Rust Programming Language]).

=== Span with CSS styles

Text with #text(fill: red)[red colour] and #text(fill: rgb("#0077cc"))[hex blue colour].

Text with #text(size: 18pt)[large font].

=== Line break

First line.\
Second line right after.

=== Unknown / pass-through tags

This has a blinking tag that is stripped but content preserved.

#line(length: 100%)

== Block HTML Elements

This is a raw HTML paragraph.

#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[
 This is a raw HTML blockquote with a left-border inset. 
]

#line(length: 100%)

=== An H3 Inside HTML

#block(fill: luma(245), inset: 10pt, radius: 4pt)[#text(font: ("DejaVu Sans Mono",), size: 10pt)[
fn greet(name: &str) -> String \{\
    format!(\"Hello, \{\}!\", name)\
\};\

]]

   - First item
   - Second item
   - Third item with #strong[bold]

   + Step one
   + Step two
   + Step three

   #strong[Markdown]
   #block(inset: (left: 12pt))[A lightweight markup language for creating formatted text.]
   #strong[Typst]
   #block(inset: (left: 12pt))[A new markup-based typesetting language.]

#table(columns: 1, stroke: luma(180), inset: 6pt, align: left,
               [#strong[Language]],       [#strong[Paradigm]],                       [Rust],       [Systems / multi-paradigm],                 [Python],       [General purpose],         
)

#block(stroke: 1pt + luma(180), inset: 8pt, radius: 4pt)[
   #strong[Click to expand]
   Hidden content revealed. 
]

This paragraph is after an HTML comment. The comment should be invisible.

#line(length: 100%)

== Mixed Markdown + HTML

#strong[Bold Markdown], #emph[italic Markdown], and #highlight[highlighted HTML] all together.

- Markdown item
- Item with #super[superscript] HTML
- Item with #raw("code") HTML

#line(length: 100%)

== Entity Decoding

- A & B in a span: A & B
- Copyright: © 2026
- Em dash: before—after
- Numeric: A = A

#line(length: 100%)

#emph[End of HTML test document.]


