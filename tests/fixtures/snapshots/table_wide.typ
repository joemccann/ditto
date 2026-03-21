#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#table(
  columns: (1fr, 1fr, 1fr, 1fr, 1fr),
  stroke: luma(200),
  inset: 6pt,
  fill: (col, row) => if row == 0 { luma(230) } else { white },
  table.cell(align: left)[#strong[ID]],
  table.cell(align: left)[#strong[Name]],
  table.cell(align: left)[#strong[Email]],
  table.cell(align: left)[#strong[Role]],
  table.cell(align: left)[#strong[Active]],
  table.cell(align: left)[1],
  table.cell(align: left)[Alice],
  table.cell(align: left)[#link("mailto:alice@example.com", [alice\@example.com])],
  table.cell(align: left)[Admin],
  table.cell(align: left)[Yes],
  table.cell(align: left)[2],
  table.cell(align: left)[Bob],
  table.cell(align: left)[#link("mailto:bob@example.com", [bob\@example.com])],
  table.cell(align: left)[User],
  table.cell(align: left)[No]
)


