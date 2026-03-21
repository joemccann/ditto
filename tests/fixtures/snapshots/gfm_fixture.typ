#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
= GFM Fidelity Test Fixture <gfm-fidelity-test-fixture>

This document exercises every improved GFM feature in a single renderable file.

#line(length: 100%)

== Nested Lists <nested-lists>

=== Bullet nesting (3 levels) <bullet-nesting-3-levels>

- Top-level item A
  - Second-level item B
    - Third-level item C
  - Second-level item D
- Top-level item E

=== Ordered nesting <ordered-nesting>

+ First step
+ Second step
  + Sub-step one
  + Sub-step two
    + Deep sub-step
+ Third step

=== Mixed bullet + ordered <mixed-bullet-ordered>

- Category Alpha
  + Alpha item one
  + Alpha item two
- Category Beta
  + Beta item one

=== Ordered list starting at 3 <ordered-list-starting-at-3>

#block[
#set enum(start: 3)
+ Already third
+ Fourth
+ Fifth
]

#line(length: 100%)

== Task Lists <task-lists>

- #box(width: 1em)[☑] Complete task
- #box(width: 1em)[☐] Incomplete task
- #box(width: 1em)[☑] Another done item
  - #box(width: 1em)[☐] Nested incomplete
  - #box(width: 1em)[☑] Nested complete
- #box(width: 1em)[☐] Final pending

#line(length: 100%)

== Tables with Alignment Markers <tables-with-alignment-markers>

#table(
  columns: (1fr, 1fr, 1fr),
  stroke: luma(200),
  inset: 6pt,
  fill: (col, row) => if row == 0 { luma(230) } else { white },
  table.cell(align: left)[#strong[Name]],
  table.cell(align: center)[#strong[Score]],
  table.cell(align: right)[#strong[Grade]],
  table.cell(align: left)[Alice],
  table.cell(align: center)[95],
  table.cell(align: right)[A+],
  table.cell(align: left)[Bob],
  table.cell(align: center)[82],
  table.cell(align: right)[B],
  table.cell(align: left)[Carol],
  table.cell(align: center)[77],
  table.cell(align: right)[C+],
  table.cell(align: left)[Dave],
  table.cell(align: center)[100],
  table.cell(align: right)[A+]
)

Table without explicit alignment (defaults to left):

#table(
  columns: (1fr, 1fr, 1fr),
  stroke: luma(200),
  inset: 6pt,
  fill: (col, row) => if row == 0 { luma(230) } else { white },
  table.cell(align: left)[#strong[Foo]],
  table.cell(align: left)[#strong[Bar]],
  table.cell(align: left)[#strong[Baz]],
  table.cell(align: left)[1],
  table.cell(align: left)[2],
  table.cell(align: left)[3]
)

#line(length: 100%)

== Autolinks <autolinks>

Plain URL autolink: #link("https://example.com")

Angle-bracket URL autolink: #link("https://www.rust-lang.org")

Email autolink: #link("mailto:user@example.com", [user\@example.com])

Explicit link (label differs from URL): #link("https://github.com", [GitHub])

#line(length: 100%)

== Footnotes <footnotes>

Markdown was invented by John Gruber.#super[1]

Typst is a new typesetting system.#super[2]

Both footnote references appear in text; definitions are collected below.

#line(length: 100%)

== Definition Lists <definition-lists>

#strong[Markdown]\
#pad(left: 1.5em)[A lightweight markup language created in 2004. Uses plain text formatting syntax.]

#strong[CommonMark]\
#pad(left: 1.5em)[A strongly defined, highly compatible specification of Markdown.]

#strong[GFM (GitHub Flavored Markdown)]\
#pad(left: 1.5em)[GitHub’s Markdown dialect that extends CommonMark with tables, task lists, strikethrough, and autolinks.]

#pad(left: 1.5em)[Also includes footnotes and definition lists via extensions.]


#line(length: 100%)

== Combined Inline Elements <combined-inline-elements>

This text has #strike[strikethrough] and #strong[bold] and #emph[italic] and `inline code`.

An expression: $E = mc^2$ and a display equation:

$ integral _0^1 x^2 dx = frac(1, 3) $

#line(length: 100%)

== Blockquote <blockquote>

#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[
This is a blockquote with #strong[formatting] and #emph[emphasis].

It spans multiple paragraphs.
]

#line(length: 100%)

== Fenced Code Block <fenced-code-block>

#block(fill: rgb("#f6f8fa"), inset: (x: 10pt, y: 8pt), radius: 4pt, width: 100%, clip: true)[#set text(font: ("DejaVu Sans Mono",), size: 9pt)
#text(fill: rgb("#a71d5d"), weight: "bold")[fn]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#795da3"), weight: "bold")[greet]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#323232"))[name]#text(fill: rgb("#323232"))[:]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[&]#text(fill: rgb("#a71d5d"), weight: "bold")[str]#text(fill: rgb("#323232"))[)]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[->]#text(fill: rgb("#323232"))[ String]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[{]\
#text(fill: rgb("#323232"))[    ]#text(fill: rgb("#323232"))[format!]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#183691"))["]#text(fill: rgb("#183691"))[Hello, ]#text(fill: rgb("#0086b3"))[{}]#text(fill: rgb("#183691"))[!]#text(fill: rgb("#183691"))["]#text(fill: rgb("#323232"))[,]#text(fill: rgb("#323232"))[ name]#text(fill: rgb("#323232"))[)]\
#text(fill: rgb("#323232"))[}]\
#h(0pt)\
#text(fill: rgb("#a71d5d"), weight: "bold")[fn]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#795da3"), weight: "bold")[main]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#323232"))[)]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#323232"))[{]\
#text(fill: rgb("#323232"))[    ]#text(fill: rgb("#323232"))[println!]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#183691"))["]#text(fill: rgb("#0086b3"))[{}]#text(fill: rgb("#183691"))["]#text(fill: rgb("#323232"))[,]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#62a35c"))[greet]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#183691"))["]#text(fill: rgb("#183691"))[world]#text(fill: rgb("#183691"))["]#text(fill: rgb("#323232"))[)]#text(fill: rgb("#323232"))[)]#text(fill: rgb("#323232"))[;]\
#text(fill: rgb("#323232"))[}]]

#line(length: 100%)

== That’s all! <that-s-all>

End of GFM fidelity fixture. All sections above should render correctly.


#line(length: 100%)

#super[1] John Gruber created Markdown in 2004 with Aaron Swartz.

#super[2] Typst was designed to be an alternative to LaTeX with a simpler syntax.


