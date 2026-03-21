#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#table(
  columns: (1fr, 1fr),
  stroke: luma(200),
  inset: 6pt,
  fill: (col, row) => if row == 0 { luma(230) } else { white },
  table.cell(align: left)[#strong[Expression]],
  table.cell(align: left)[#strong[Description]],
  table.cell(align: left)[$x^2$],
  table.cell(align: left)[quadratic],
  table.cell(align: left)[$sqrt(x)$],
  table.cell(align: left)[square root]
)


