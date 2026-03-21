#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#block(fill: rgb("#f6f8fa"), inset: (x: 10pt, y: 8pt), radius: 4pt, width: 100%, clip: true)[#set text(font: ("DejaVu Sans Mono",), size: 9pt)
#text(fill: rgb("#a71d5d"), weight: "bold")[SELECT]#text(fill: rgb("#323232"))[ name, ]#text(fill: rgb("#62a35c"))[COUNT]#text(fill: rgb("#323232"))[(]#text(fill: rgb("#a71d5d"), weight: "bold")[\*]#text(fill: rgb("#323232"))[) ]#text(fill: rgb("#a71d5d"), weight: "bold")[FROM]#text(fill: rgb("#323232"))[ users ]#text(fill: rgb("#a71d5d"), weight: "bold")[GROUP BY]#text(fill: rgb("#323232"))[ name ]#text(fill: rgb("#a71d5d"), weight: "bold")[ORDER BY]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#0086b3"))[2]#text(fill: rgb("#323232"))[ ]#text(fill: rgb("#a71d5d"), weight: "bold")[DESC]#text(fill: rgb("#323232"))[;]]


