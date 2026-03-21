#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#show outline.entry.where(level: 1): it => {
strong(it)
}
#outline(
title: [Table of Contents],
depth: 3,
indent: 1.5em,
)
#pagebreak()

= H1 <h1>

== H2 <h2>


