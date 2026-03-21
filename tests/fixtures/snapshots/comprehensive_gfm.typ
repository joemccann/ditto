#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#show outline.entry.where(level: 1): it => {
strong(it)
}
#outline(
title: [Contents],
depth: 3,
indent: 1.5em,
)
#pagebreak()

= Comprehensive GFM Reference <comprehensive-gfm-reference>

A complete reference document exercising all GitHub Flavored Markdown features.

#line(length: 100%)

== 1. Headings (ATX Style) <h-1-headings-atx-style>

= H1 Heading <h1-heading>

== H2 Heading <h2-heading>

=== H3 Heading <h3-heading>

==== H4 Heading <h4-heading>

===== H5 Heading <h5-heading>

====== H6 Heading <h6-heading>

#line(length: 100%)

== 2. Text Formatting <h-2-text-formatting>

Regular paragraph text.

#strong[Bold text] and #emph[italic text] and #emph[#strong[bold italic]].

#strike[Strikethrough text] is supported.

`Inline code` with backticks.

Text with a hard break (two trailing spaces): Second line after hard break.

#line(length: 100%)

== 3. Blockquotes <h-3-blockquotes>

#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[
Simple blockquote with a single paragraph.
]

#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[
Multi-line blockquote.

Second paragraph inside the quote.

#strong[Bold] and #emph[italic] work inside blockquotes too.
]

#line(length: 100%)

== 4. Lists <h-4-lists>

=== Unordered <unordered>

- Alpha
- Beta
  - Beta sub-one
  - Beta sub-two
    - Deep level
- Gamma

=== Ordered <ordered>

+ First
+ Second
  + Sub-item one
  + Sub-item two
+ Third

=== Starting at non-1 <starting-at-non-1>

#block[
#set enum(start: 5)
+ Fifth item
+ Sixth item
+ Seventh item
]

=== Task List <task-list>

- #box(width: 1em)[☑] Completed item
- #box(width: 1em)[☐] Incomplete item
- #box(width: 1em)[☑] Another completed
  - #box(width: 1em)[☐] Nested incomplete
  - #box(width: 1em)[☑] Nested complete

#line(length: 100%)

== 5. Code <h-5-code>

Inline: `println!("Hello, world!");`

Fenced Rust:

#block(fill: rgb("#f6f8fa"), inset: (x: 10pt, y: 8pt), radius: 4pt, width: 100%, clip: true)[#set text(font: ("DejaVu Sans Mono",), size: 9pt)
#text(fill: rgb("#a71d5d"), weight: "bold")[use]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[std]#text(fill: rgb("#323232"))[::]#text(fill: rgb("#323232"))[fmt]#text(fill: rgb("#323232"))[;]\
#h(0pt)\
#text(fill: rgb("#a71d5d"), weight: "bold")[struct]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[Point]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[{]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[x]#text(fill: rgb("#323232"))[:]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[f64]#text(fill: rgb("#323232"))[, ]#text(fill: rgb("#323232"))[y]#text(fill: rgb("#323232"))[:]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[f64]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[}]\
#h(0pt)\
#text(fill: rgb("#a71d5d"), weight: "bold")[impl]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[fmt]#text(fill: rgb("#323232"))[::]#text(fill: rgb("#323232"))[Display ]#text(fill: rgb("#a71d5d"), weight: "bold")[for]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[Point]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[{]\
#text(fill: rgb("#323232"))[    ]#text(fill: rgb("#a71d5d"), weight: "bold")[fn]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#795da3"), weight: "bold")[fmt]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#a71d5d"), weight: "bold")[&]#text(fill: rgb("#323232"))[self]#text(fill: rgb("#323232"))[, ]#text(fill: rgb("#323232"))[f]#text(fill: rgb("#323232"))[:]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[&]#text(fill: rgb("#a71d5d"), weight: "bold")[mut]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[fmt]#text(fill: rgb("#323232"))[::]#text(fill: rgb("#323232"))[Formatter]#text(fill: rgb("#323232"))[)]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[->]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[fmt]#text(fill: rgb("#323232"))[::]#text(fill: rgb("#323232"))[Result]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[{]\
#text(fill: rgb("#323232"))[        ]#text(fill: rgb("#323232"))[write!]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#323232"))[f,]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#183691"))["]#text(fill: rgb("#183691"))[(]#text(fill: rgb("#0086b3"))[{}]#text(fill: rgb("#183691"))[, ]#text(fill: rgb("#0086b3"))[{}]#text(fill: rgb("#183691"))[)]#text(fill: rgb("#183691"))["]#text(fill: rgb("#323232"))[,]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[self]#text(fill: rgb("#323232"))[.x]#text(fill: rgb("#323232"))[,]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[self]#text(fill: rgb("#323232"))[.y]#text(fill: rgb("#323232"))[)]\
#text(fill: rgb("#323232"))[    ]#text(fill: rgb("#323232"))[}]\
#text(fill: rgb("#323232"))[}]]

Fenced Python:

#block(fill: rgb("#f6f8fa"), inset: (x: 10pt, y: 8pt), radius: 4pt, width: 100%, clip: true)[#set text(font: ("DejaVu Sans Mono",), size: 9pt)
#text(fill: rgb("#a71d5d"), weight: "bold")[def]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"), weight: "bold")[factorial]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#323232"))[n]#text(fill: rgb("#323232"))[:]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#0086b3"))[int]#text(fill: rgb("#323232"))[)]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[->]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#0086b3"))[int]#text(fill: rgb("#323232"))[:]\
#text(fill: rgb("#323232"))[    ]#text(fill: rgb("#a71d5d"), weight: "bold")[return]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#0086b3"))[1]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[if]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[n]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[<=]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#0086b3"))[1]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[else]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[n]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[\*]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[factorial]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#323232"))[n]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[-]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#0086b3"))[1]#text(fill: rgb("#323232"))[)]]

Plain (no language):

#block(fill: rgb("#f6f8fa"), inset: (x: 10pt, y: 8pt), radius: 4pt, width: 100%, clip: true)[#set text(font: ("DejaVu Sans Mono",), size: 9pt)
#raw("plain text code block
no syntax highlighting", block: false)]

#line(length: 100%)

== 6. Tables <h-6-tables>

Basic table:

#table(
  columns: (1fr, 1fr, 1fr),
  stroke: luma(200),
  inset: 6pt,
  fill: (col, row) => if row == 0 { luma(230) } else { white },
  table.cell(align: left)[#strong[Col A]],
  table.cell(align: left)[#strong[Col B]],
  table.cell(align: left)[#strong[Col C]],
  table.cell(align: left)[1],
  table.cell(align: left)[2],
  table.cell(align: left)[3],
  table.cell(align: left)[4],
  table.cell(align: left)[5],
  table.cell(align: left)[6]
)

With alignment:

#table(
  columns: (1fr, 1fr, 1fr),
  stroke: luma(200),
  inset: 6pt,
  fill: (col, row) => if row == 0 { luma(230) } else { white },
  table.cell(align: left)[#strong[Left]],
  table.cell(align: center)[#strong[Center]],
  table.cell(align: right)[#strong[Right]],
  table.cell(align: left)[a],
  table.cell(align: center)[b],
  table.cell(align: right)[c],
  table.cell(align: left)[d],
  table.cell(align: center)[e],
  table.cell(align: right)[f]
)

With formatted cells:

#table(
  columns: (1fr, 1fr, 1fr),
  stroke: luma(200),
  inset: 6pt,
  fill: (col, row) => if row == 0 { luma(230) } else { white },
  table.cell(align: left)[#strong[Feature]],
  table.cell(align: center)[#strong[Supported]],
  table.cell(align: left)[#strong[Notes]],
  table.cell(align: left)[#strong[Bold]],
  table.cell(align: center)[✅],
  table.cell(align: left)[Works in cells],
  table.cell(align: left)[#emph[Italic]],
  table.cell(align: center)[✅],
  table.cell(align: left)[Works in cells],
  table.cell(align: left)[`Code`],
  table.cell(align: center)[✅],
  table.cell(align: left)[Works in cells],
  table.cell(align: left)[#strike[Strike]],
  table.cell(align: center)[✅],
  table.cell(align: left)[Works in cells]
)

#line(length: 100%)

== 7. Links and Autolinks <h-7-links-and-autolinks>

Explicit link: #link("https://www.rust-lang.org", [Rust Language])

Autolink URL: #link("https://typst.app")

Angle-bracket URL: #link("https://github.com")

Email autolink: #link("mailto:user@example.com", [user\@example.com])

#line(length: 100%)

== 8. Horizontal Rules <h-8-horizontal-rules>

Above the rule.

#line(length: 100%)

Below the rule.

#line(length: 100%)

Another style.

#line(length: 100%)

== 9. Math <h-9-math>

Inline: $x = frac(-b plus.minus sqrt(b^2 - 4ac), 2a)$

Display block:

$ sum _{n=1}^{oo} frac(1, n^2) = frac(pi^2, 6) $

Greek letters: $alpha, beta, gamma, Delta, Omega$

Operators: $sin theta$, $cos theta$, $lim _{x ->0}$

Matrix:

$ mat(delim: "[", a, b; c, d) $

#line(length: 100%)

== 10. Footnotes <h-10-footnotes>

Markdown was created by John Gruber#super[1] and Aaron Swartz.

CommonMark provides a formal specification#super[2].

GFM extends CommonMark#super[3] with additional features.

#line(length: 100%)

== 11. Definition Lists <h-11-definition-lists>

#strong[Markdown]\
#pad(left: 1.5em)[A lightweight markup language for creating formatted text.]

#strong[Typst]\
#pad(left: 1.5em)[A new typesetting system, written in Rust, designed as a LaTeX alternative.]

#pad(left: 1.5em)[Available at #link("https://typst.app")]

#strong[CommonMark]\
#pad(left: 1.5em)[A strongly defined, highly compatible specification of Markdown.]


#line(length: 100%)

== 12. HTML Inline Elements <h-12-html-inline-elements>

Text with #strong[HTML bold] and #emph[HTML italic].

A line break: before\
after.

#line(length: 100%)

== 13. Special Characters <h-13-special-characters>

Dollar signs: \$9.99 and \$100.

Hash marks: \#1 trending, the \#rust channel.

Backslash path: `C:\Users\name`.

#line(length: 100%)

== End of Reference <end-of-reference>

All GFM features above should render correctly to PDF.


#line(length: 100%)

#super[1] John Gruber published the original Markdown specification in 2004.

#super[2] CommonMark is at #link("https://commonmark.org")

#super[3] GitHub Flavored Markdown spec: #link("https://github.github.com/gfm/")


