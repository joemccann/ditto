# Known Limitations

This document records all known incompatibilities between ditto and the
CommonMark / GitHub Flavored Markdown (GFM) specifications, including
unsupported HTML tags, unsupported LaTeX/math commands, and remaining GFM gaps.

Each entry states the current behaviour so you know exactly what to expect.

---

## 1. HTML in Markdown

### 1.1 Supported HTML tags

ditto translates a fixed set of HTML tags embedded in Markdown.
Tags **not** in either list below are silently stripped; their text content is
preserved.

**Inline tags** (inside paragraphs)

| Tag | Typst output |
|---|---|
| `<b>`, `<strong>` | `#strong[…]` |
| `<i>`, `<em>` | `#emph[…]` |
| `<u>` | `#underline[…]` |
| `<s>`, `<del>` | `#strike[…]` |
| `<ins>` | `#underline[…]` (no distinct insert style) |
| `<mark>` | `#highlight[…]` |
| `<small>` | `#text(size: 0.8em)[…]` |
| `<sub>` | `#sub[…]` |
| `<sup>` | `#super[…]` |
| `<code>`, `<kbd>`, `<samp>`, `<var>` | `#raw("…")` monospace |
| `<cite>`, `<dfn>`, `<abbr>`, `<acronym>`, `<q>`, `<time>`, `<data>` | `#emph[…]` (italic fallback) |
| `<span style="color:…; font-size:…">` | `#text(fill:…, size:…)[…]` |
| `<a href="…">` | `#link("…", […])` |
| `<br>`, `<wbr>` | `\` (line break) |
| `<hr>` | `#line(length: 100%)` |

**Block tags** (standalone HTML blocks)

| Tag | Typst output |
|---|---|
| `<p>` | paragraph |
| `<div>`, `<section>`, `<article>`, `<main>`, `<header>`, `<footer>`, `<nav>`, `<aside>` | pass-through container |
| `<blockquote>` | left-bordered block |
| `<pre>` (+ inner `<code>`) | monospace verbatim block |
| `<ul>` / `<li>` | bullet list |
| `<ol>` / `<li>` | numbered list (always starts at 1; see §1.3) |
| `<dl>` / `<dt>` / `<dd>` | definition list |
| `<table>` / `<thead>` / `<tbody>` / `<tfoot>` / `<tr>` / `<th>` / `<td>` | table (single fixed column; see §1.3) |
| `<img src="…" alt="…">` | `#figure(image(…))` |
| `<figure>` / `<figcaption>` | figure with italic caption |
| `<details>` / `<summary>` | bordered block with bold summary |
| `<h1>`–`<h6>` | Typst heading markers |
| `<hr>` | `#line(length: 100%)` |
| `<br>` | line break |

### 1.2 Unsupported / stripped HTML tags

The following tags are explicitly **not** translated. Their text content is
passed through but all visual / semantic meaning is lost.

- `<button>`, `<input>`, `<select>`, `<textarea>`, `<form>` — interactive form
  controls have no PDF equivalent.
- `<video>`, `<audio>`, `<canvas>`, `<embed>`, `<object>`, `<iframe>` — media
  and embedded content is not renderable in a static PDF.
- `<script>`, `<style>`, `<link>`, `<meta>`, `<head>` — document metadata and
  scripting are discarded.
- `<ruby>`, `<rt>`, `<rp>` — ruby annotations are stripped to plain text.
- `<bdi>`, `<bdo>` — bidirectional text overrides are ignored.
- `<map>`, `<area>` — image maps are not supported.
- `<progress>`, `<meter>`, `<output>` — form output elements.
- `<template>`, `<slot>`, `<portal>` — shadow DOM / web-component elements.
- `<noscript>` — content is discarded along with the tag.
- `<address>` — rendered as plain text (no italic or block styling).
- `<caption>` (inside `<table>`) — caption text is discarded; use `<figcaption>` instead.
- `<colgroup>`, `<col>` — column attributes are not applied.
- `<optgroup>`, `<option>` — form select options are ignored.
- SVG inline markup (`<svg>`, `<path>`, `<circle>`, etc.) — inline SVG is
  stripped to text. Use `<img src="file.svg">` to embed an SVG file.

### 1.3 Partially supported HTML features

**`<table>` in HTML blocks — fixed single-column layout**
HTML-block tables are rendered with `columns: 1` regardless of the actual
column count. This is a known simplification; Markdown-native GFM tables
(pipe syntax) are fully supported with per-column alignment.

**`<ol>` start attribute in HTML blocks**
The `start` attribute on `<ol>` is not honoured inside raw HTML blocks;
numbering always begins at 1. Markdown-native ordered lists do support
non-1 starting numbers via the `start` attribute.

**`<span>` CSS — color and font-size only**
Only the `color` and `font-size` CSS properties are translated on `<span>`.
All other properties (`background-color`, `font-weight`, `text-decoration`,
`text-align`, `border`, `padding`, `margin`, `display`, `position`, etc.) are
silently ignored.

**Named CSS colours — limited set**
Only these named colours are mapped to Typst built-in names: `red`, `green`,
`blue`, `white`, `black`, `gray`/`grey`, `orange`, `purple`, `yellow`, `pink`.
All other CSS named colours (e.g. `coral`, `teal`, `navy`, `salmon`) fall back
to `rgb("…")` with the raw CSS name, which may not compile correctly in Typst.
Use `#rrggbb` hex values for reliable colour rendering.

**`<a href>` — no `title` attribute**
The `title` tooltip attribute on anchor tags is discarded; only `href` is used.

**`<img>` in HTML blocks — no remote image download**
Unlike Markdown-native images, `<img>` tags inside raw HTML blocks are not
downloaded at build time. Remote `src` URLs are passed directly to Typst,
which will produce a build error for non-local paths. Use Markdown image syntax
`![alt](url)` for remote images.

**HTML entities — partial coverage**
Named entities decoded: `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`,
`&mdash;`, `&ndash;`, `&laquo;`, `&raquo;`, `&copy;`, `&reg;`, `&trade;`,
`&hellip;`. All other named entities (e.g. `&eacute;`, `&auml;`, `&yen;`) are
passed through as-is, which will appear literally in the PDF. Numeric entities
(`&#123;` and `&#x7B;`) are fully supported.

**HTML comments** are silently discarded (correct behaviour).

**DOCTYPE and XML declarations** are silently discarded.

---

## 2. Math / LaTeX

ditto translates LaTeX-style math expressions to Typst native math syntax.
Most common constructs work, but the following are known to be incomplete.

### 2.1 Supported math constructs

- Greek letters (lower and upper case): `\alpha` … `\Omega`
- Fractions: `\frac`, `\dfrac`, `\tfrac`, `\cfrac`, `\binom`
- Roots: `\sqrt`, `\sqrt[n]`
- Integrals: `\int`, `\iint`, `\iiint`, `\oint`
- Sums / products: `\sum`, `\prod`
- Big operators: `\bigcup`, `\bigcap`
- Trig and log functions: `\sin`, `\cos`, `\tan`, `\arcsin`, `\arccos`,
  `\arctan`, `\sinh`, `\cosh`, `\tanh`, `\ln`, `\log`, `\exp`, `\lim`,
  `\limsup`, `\liminf`, `\sup`, `\inf`, `\max`, `\min`, `\arg`, `\det`,
  `\dim`, `\gcd`, `\hom`, `\ker`
- Accents: `\hat`, `\tilde`, `\bar`/`\overline`, `\underline`, `\vec`,
  `\dot`, `\ddot`, `\underbrace`, `\overbrace`, `\widehat`, `\widetilde`
- Arrows: `\to`/`\rightarrow`, `\leftarrow`, `\Rightarrow`, `\Leftarrow`,
  `\leftrightarrow`, `\Leftrightarrow`, `\mapsto`, `\uparrow`, `\downarrow`,
  `\updownarrow`, `\longrightarrow`, `\longleftarrow`
- Relations: `\leq`/`\le`, `\geq`/`\ge`, `\neq`/`\ne`, `\approx`, `\sim`,
  `\simeq`, `\cong`, `\equiv`, `\propto`, `\ll`, `\gg`
- Set operators: `\in`, `\notin`, `\subset`, `\subseteq`, `\supset`,
  `\supseteq`, `\cup`, `\cap`, `\setminus`, `\emptyset`/`\varnothing`,
  `\forall`, `\exists`, `\nexists`
- Binary operators: `\cdot`, `\times`, `\div`, `\pm`, `\mp`, `\oplus`,
  `\otimes`, `\circ`, `\bullet`, `\land`/`\wedge`, `\lor`/`\vee`,
  `\neg`/`\lnot`
- Delimiters: `\left`/`\right` (auto-sized), `\langle`/`\rangle`,
  `\lfloor`/`\rfloor`, `\lceil`/`\rceil`, `\lVert`/`\rVert`
- Text and font switching: `\text`, `\mathrm`, `\mathit`, `\mathsf`,
  `\mathtt`, `\mathbf`, `\boldsymbol`, `\bm`, `\operatorname`,
  `\textbf`, `\textrm`, `\textnormal`
- Blackboard bold: `\mathbb{R}` → `RR`, `\mathbb{Z}` → `ZZ`, `\mathbb{N}` →
  `NN`, `\mathbb{Q}` → `QQ`, `\mathbb{C}` → `CC`, `\mathbb{H}` → `HH`;
  all others → `bb(…)`
- Spacing: `\,`, `\:`, `\;`, `\!`, `\ ` (all map to a single space), `\quad`,
  `\qquad`
- Misc: `\partial`, `\nabla`, `\infty`, `\hbar`, `\ell`, `\Re`, `\Im`,
  `\aleph`, `\prime`, `\dagger`, `\ddagger`, `\star`, `\ast`
- Dots: `\cdots`, `\ldots`/`\dots`, `\vdots`, `\ddots`
- Layout modifiers (silently dropped): `\displaystyle`, `\textstyle`,
  `\scriptstyle`, `\scriptscriptstyle`, `\limits`, `\nolimits`,
  `\nonumber`, `\notag`, `\label`, `\tag`
- Environments: `matrix`, `pmatrix`, `bmatrix`, `Bmatrix`, `vmatrix`,
  `Vmatrix`, `cases`, `align`, `align*`, `aligned`, `equation`, `equation*`

### 2.2 Unsupported or partially supported LaTeX commands

**Commands with no translation (passed through as `\cmd`)**

Any command not in the supported list above is emitted verbatim as `\cmd`,
which will produce a Typst compile warning or error. Examples:

- `\color{red}`, `\textcolor{red}{…}` — colour inside math
- `\boxed{…}` — framed expression
- `\cancel{…}`, `\bcancel{…}`, `\xcancel{…}` — strikethrough in math
- `\mathcal{…}`, `\mathfrak{…}`, `\mathscr{…}` — calligraphic / fraktur / script fonts
- `\not\equiv`, `\not\subset`, etc. — negated operators via `\not`
- `\xleftarrow{…}`, `\xrightarrow{…}` — labelled arrows
- `\overset{…}{…}`, `\underset{…}{…}` — stacked expressions
- `\stackrel{…}{…}` — stacked relations
- `\smash{…}` — zero-height expression
- `\phantom{…}`, `\hphantom{…}`, `\vphantom{…}` — invisible spacing
- `\mbox{…}` — text in math (use `\text{…}` instead)
- `\substack{…}` — multi-line limits
- `\sideset{…}{…}{…}` — side subscripts/superscripts
- `\prescript{…}{…}{…}` — pre-subscripts
- `\bigoplus`, `\bigotimes`, `\bigsqcup`, `\biguplus`, `\bigvee`, `\bigwedge` — large operators beyond `\bigcup`/`\bigcap`
- `\oint` (supported) vs. `\oiint`, `\oiiint` (not supported)
- `\varlimsup`, `\varliminf`, `\varinjlim`, `\varprojlim` — variant limit operators
- `\DeclareMathOperator` and other preamble-style commands
- AMS math packages: `\intertext`, `\shortintertext`, `\allowdisplaybreaks`
- `\boldsymbol` outside math mode
- `\newcommand`, `\renewcommand`, `\def` — custom macro definitions are not
  processed; the definition is silently dropped and uses of the macro emit
  `\macroname` literally

**Environments not translated**

- `multline`, `multline*`, `gather`, `gather*`, `flalign`, `flalign*`,
  `alignat`, `alignat*`, `split` — treated as plain `align`; alignment
  characters (`&`) may produce unexpected output
- `array` — column format specifier is ignored; rows/cells rendered as-is
- `tabular` (math context) — not supported
- `tikzpicture`, `pgfplot` — TikZ drawing commands are not supported
- `minipage`, `figure` (inside math) — not supported

**Multi-letter implicit products**

Typst treats multi-letter sequences in math as named variables (e.g. `dx` is
rendered as `dx` in italic). When the intent is the product `d·x`, write
`d x` (with a space) in the LaTeX source so Typst sees two separate tokens.

**Display math — equation numbering**

`\tag{…}` and `\label{…}` inside display math are silently discarded. Numbered
equations are not supported.

**Aligned environments — ampersand columns**

The `&` column separator in `align`/`aligned` is preserved as-is and passed to
Typst. Complex multi-column alignment (`&\quad&`) may not render correctly.

---

## 3. GitHub Flavored Markdown (GFM) gaps

### 3.1 Supported GFM extensions

| Feature | Status |
|---|---|
| Tables (pipe syntax) | ✅ Full support with `left`/`center`/`right` alignment |
| Strikethrough `~~text~~` | ✅ |
| Task lists `- [x]` | ✅ |
| Autolinks `<url>` and bare URLs | ✅ |
| Footnotes `[^ref]` | ✅ |
| Definition / description lists | ✅ |
| GitHub Alerts `> [!NOTE]` etc. | ✅ (Note, Tip, Important, Warning, Caution) |
| Superscript `^text^` | ✅ |
| Subscript `~text~` (single tilde) | ✅ |
| Underline `__text__` | ✅ |
| Fenced code blocks with language | ✅ 100+ languages via Syntect |
| Math `$…$` and `$$…$$` | ✅ via LaTeX-to-Typst translation |
| Math `` ```math ``` `` blocks | ✅ |
| YAML front matter `---` | ✅ (toc keys only; see §3.3) |

### 3.2 Unsupported GFM extensions

**Wikilinks `[[Page Name]]`**
Not enabled. comrak supports wikilinks as an opt-in extension; ditto does
not enable it and the `[[…]]` syntax is treated as plain text.

**Spoiler / hidden text**
Not supported.

**Custom heading IDs `{#my-id}`**
The `header_ids` comrak extension is not enabled. Custom `{#id}` syntax is
passed through as literal text.

**Extended autolink protocol prefixes**
Only `http://` and `https://` URLs are treated as links in the autolink path.
URLs with other schemes (e.g. `ftp://`, `mailto:`, `tel:`) in bare-text
position are rendered as plain text unless surrounded by `<` … `>` angle-
bracket autolink syntax.

**Link reference definitions as footnotes**
CommonMark link reference definitions (`[label]: url "title"`) work as
expected for links. They are not repurposed as footnotes.

### 3.3 Front-matter parsing — limited key support

The YAML front-matter block is parsed with a minimal hand-rolled parser that
recognises only these keys:

| Key | Effect |
|---|---|
| `toc` | Boolean — enable/disable TOC |
| `no_toc` | Boolean — alias for `toc: false` |
| `toc_depth` | Integer 1–6 — max heading depth in TOC |
| `toc_title` | String — custom TOC page heading |

All other YAML keys (`title`, `author`, `date`, `tags`, custom keys, etc.) are
silently ignored. Full YAML parsing (sequences, nested maps, multi-line values)
is not implemented.

---

## 4. CommonMark edge cases

### 4.1 Soft breaks

Comrak's `smart` punctuation mode is enabled. This means certain ASCII
sequences are converted to typographic characters:

| Input | Rendered as |
|---|---|
| `--` | – (en-dash) |
| `---` | — (em-dash) |
| `...` | … (ellipsis) |
| `"text"` | "text" (smart quotes) |
| `'text'` | 'text' (smart quotes) |

This affects code that appears outside fenced/inline code spans. To disable
smart punctuation you would need to rebuild without `options.parse.smart = true`.

### 4.2 Indented code blocks

Four-space indented code blocks are parsed correctly, but syntax highlighting
is applied only to *fenced* code blocks that include an explicit language
identifier. Indented code blocks and fenced blocks without a language tag
render in plain monospace without token colouring.

### 4.3 Tight vs. loose lists

Whether a list is tight (no blank lines between items) or loose (blank lines
between items) affects Markdown rendering. ditto renders all list items
with a newline after each, which may add extra vertical space in lists that a
browser would render tightly.

### 4.4 Setext headings

Setext-style headings (underlined with `===` or `---`) are parsed correctly by
comrak and rendered identically to ATX-style headings. No known issues.

### 4.5 Link titles

Link titles (`[text](url "title")`) are parsed but the `title` attribute is
discarded in the PDF output. Only the link text and URL are used.

### 4.6 Reference-style links and images

Reference-style links (`[text][ref]` with `[ref]: url` definitions) and
reference-style images (`![alt][ref]`) are supported. The link title in the
definition is discarded (see §4.5).

### 4.7 Raw HTML pass-through and tag filtering

comrak's `tagfilter` extension is enabled, which means certain potentially
dangerous HTML tags (`<script>`, `<iframe>`, `<style>`, etc.) are filtered from
Markdown before parsing. Additionally, `render.unsafe_` is enabled so that
safe raw HTML blocks are passed to the HTML translator instead of being
replaced with a comment. The net effect is that safe common HTML works and
dangerous HTML is stripped.

---

## 5. Image limitations

### 5.1 Unsupported image formats

Typst supports PNG, JPEG, GIF, WebP, SVG, BMP, TIFF, and AVIF. Images in any
other format (e.g. HEIC, HEIF, RAW camera formats, ICO larger than 16×16) will
cause a Typst compile error. The image pipeline will detect the format and write
the file, but Typst will reject it at compile time.

### 5.2 Remote images in HTML blocks

`<img>` tags inside raw HTML blocks (not Markdown image syntax) are **not**
downloaded at build time. Typst tries to resolve the `src` as a local path,
which will fail for `http://` URLs. Use Markdown image syntax `![alt](url)` for
remote images.

### 5.3 Image sizing

All images default to `width: 100%` (full text width) when no explicit size is
given. There is no support for Markdown-extension image sizing syntax
(e.g. `![alt](url){width=50%}`). To specify a size, use an HTML `<img>` tag
with `width` / `height` attributes in a Markdown-native HTML block adjacent to
the Markdown image.

### 5.4 Cache invalidation

Remote images are cached by URL hash. The cache uses HTTP `ETag` /
`Last-Modified` for conditional requests. If a remote image changes but keeps
the same URL and the server does not send revalidation headers, the stale
cached copy will be used until the cache directory is deleted manually.

### 5.5 SVG images — Typst version dependency

SVG support is provided by Typst's built-in SVG renderer. Complex SVG features
(filters, animations, foreign objects, CSS `@import` inside SVG) may not render
identically to a browser. Animated SVGs are rendered as a static first frame.

---

## 6. Typography and layout limitations

### 6.1 No font embedding control

All fonts are embedded in the PDF. There is no option to subset, exclude, or
reference fonts without embedding.

### 6.2 No page number control in body

Body content always starts on the first or second page (second when a TOC is
enabled). There is no option to offset the page-number counter, restart
numbering, or suppress page numbers.

### 6.3 No column layout

Multi-column layouts are not supported. All content is single-column.

### 6.4 Pagination inside lists and tables

Long tables or deeply nested lists that span a page boundary are handled by
Typst's layout engine. In rare cases this can produce an awkward break. There
is no manual `\pagebreak` or `\nopagebreak` equivalent in Markdown.

### 6.5 Right-to-left text

RTL languages are not explicitly configured. The underlying Typst engine has
some RTL support, but ditto does not expose `dir: rtl` settings.

---

## 7. CLI limitations

### 7.1 No watch mode

There is no `--watch` flag. Re-run the CLI manually after editing the source
file.

### 7.2 No batch / glob input

Only a single input file (or stdin) is accepted per invocation. To convert
multiple files, script the CLI in a loop.

### 7.3 No incremental compilation

Every run re-parses and re-renders the entire document from scratch. Large
documents with many remote images may be slow on first run (images are cached
after the first download).

### 7.4 No PDF metadata control

Document metadata fields (PDF `Title`, `Author`, `Subject`, `Keywords`,
`CreationDate`) are not exposed as CLI flags or front-matter keys. Typst sets
them to default/empty values.

---

## 8. Reporting new incompatibilities

If you find a Markdown construct that renders incorrectly or is silently
dropped, please open an issue with:

1. The smallest Markdown snippet that reproduces the problem.
2. What you expected to see in the PDF.
3. What actually appeared (attach the PDF if possible).
