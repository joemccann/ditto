#set page(
  width: 210mm,
  height: 297mm,
  margin: 20mm,
)
#set text(font: ("Libertinus Serif",), size: 12pt)
#set par(justify: false)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11pt)
#show heading.where(level: 1): it => [#block(below: 0.7em)[#set text(size: 24pt, weight: "bold")#it.body]
#place(left + bottom, dx: 0pt, dy: -1.2em, line(length: 100%))]
#show heading.where(level: 2): it => [#block(below: 0.5em)[#set text(size: 19pt, weight: "bold")#it.body]]
#show heading.where(level: 3): it => [#block(below: 0.4em)[#set text(size: 15pt, weight: "bold")#it.body]]
#show link: set text(fill: rgb("1756d1"))
#show par: set block(spacing: 0.8em)
#show table.cell: set text(size: 11pt)
= Table of Contents
- one
#import "@preview/html:0.4.0": html
#html.block[
<h1 id="x">Hello</h1>
<p>World</p>
]
