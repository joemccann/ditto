#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
- #box(width: 1em)[☑] Done
- #box(width: 1em)[☐] Pending
- #box(width: 1em)[☑] Another
  - #box(width: 1em)[☐] Nested pending
  - #box(width: 1em)[☑] Nested done


