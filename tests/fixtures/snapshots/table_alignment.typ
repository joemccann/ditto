#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#table(
  columns: (1fr, 1fr, 1fr),
  stroke: luma(200),
  inset: 6pt,
  fill: (col, row) => if row == 0 { luma(230) } else { white },
  table.cell(align: left)[#strong[L]],
  table.cell(align: center)[#strong[C]],
  table.cell(align: right)[#strong[R]],
  table.cell(align: left)[a],
  table.cell(align: center)[b],
  table.cell(align: right)[c]
)


