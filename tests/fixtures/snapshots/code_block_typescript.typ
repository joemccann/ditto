#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#block(fill: rgb("#f6f8fa"), inset: (x: 10pt, y: 8pt), radius: 4pt, width: 100%, clip: true)[#set text(font: ("DejaVu Sans Mono",), size: 9pt)
#raw("interface User { id: number; name: string; }
async function get(id: number): Promise<User> {
    return fetch(`/users/${id}`);
}", block: false)]


