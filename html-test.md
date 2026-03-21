# HTML-in-Markdown Test Document

This document exercises all supported raw HTML tags inside Markdown.

---

## Inline HTML Tags

### Text formatting

Normal paragraph with <b>bold via b-tag</b>, <strong>bold via strong</strong>,
<i>italic via i-tag</i>, <em>italic via em-tag</em>, and <u>underline</u>.

Strikethrough: <s>old price</s> and <del>deleted text</del>.

Inserted text: <ins>newly added</ins>.

Highlighted text: <mark>this is highlighted</mark>.

Small text: <small>fine print here</small>.

### Subscript and Superscript

H<sub>2</sub>O is water. E = mc<sup>2</sup> is Einstein's famous equation.
x<sub>n+1</sub> = x<sub>n</sub><sup>2</sup> + c

### Monospace / Keyboard / Sample

Press <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>Del</kbd> to reset.

Shell output: <samp>Permission denied</samp>.

Variable: <var>x</var> and <var>y</var>.

Inline code via HTML: <code>printf("Hello\n");</code>

### Semantic inline tags

<cite>The Elements of Style</cite> by Strunk and White.

<abbr title="HyperText Markup Language">HTML</abbr> is the backbone of the web.

<dfn>Markdown</dfn> is a lightweight markup language.

<q>To be or not to be</q> — that is the question.

<time>2026-03-21</time> — today's date.

### Links

Visit <a href="https://www.rust-lang.org">The Rust Programming Language</a> website.

### Span with CSS styles

Text with <span style="color: red">red colour</span> and
<span style="color: #0077cc">hex blue colour</span>.

Text with <span style="font-size: 18px">large font</span> and
<span style="font-size: 0.8em">smaller font</span>.

Combined: <span style="color: green; font-size: 14px">green medium text</span>.

### Line break

First line.<br>Second line right after.

First line.<br />Second line (XHTML self-closing).

### Unknown / pass-through tags

This has a <blink>blinking tag</blink> that is stripped but content preserved.

---

## Block HTML Elements

### Paragraph

<p>This is a raw HTML paragraph. It should render as a normal paragraph with proper spacing.</p>

### Div / Section / Article

<div>
This content is inside a div. It should render as a block of text.
</div>

<section>
A semantic section element — treated as a plain block.
</section>

### Blockquote

<blockquote>
This is a raw HTML blockquote. It should have the same visual treatment as a Markdown blockquote — a left border and inset.
</blockquote>

### Horizontal Rule

<hr>

### Headings inside HTML

<h2>An H2 Inside HTML</h2>

<h3>An H3 Inside HTML</h3>

### Preformatted / Code Block

<pre><code>fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    println!("{}", greet("world"));
}</code></pre>

### Unordered List

<ul>
  <li>First item</li>
  <li>Second item</li>
  <li>Third item with <strong>bold</strong></li>
</ul>

### Ordered List

<ol>
  <li>Step one</li>
  <li>Step two</li>
  <li>Step three</li>
</ol>

### Description List

<dl>
  <dt>Markdown</dt>
  <dd>A lightweight markup language for creating formatted text.</dd>
  <dt>Typst</dt>
  <dd>A new markup-based typesetting language designed to be as powerful as LaTeX while being easier to learn.</dd>
</dl>

### Image

<img src="https://via.placeholder.com/400x200.png" alt="A placeholder image" width="400">

### Figure with Caption

<figure>
  <img src="https://via.placeholder.com/300x150.png" alt="Another image">
  <figcaption>Figure 1: A sample placeholder image with a caption.</figcaption>
</figure>

### Table

<table>
  <thead>
    <tr>
      <th>Language</th>
      <th>Paradigm</th>
      <th>First appeared</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Rust</td>
      <td>Systems / multi-paradigm</td>
      <td>2010</td>
    </tr>
    <tr>
      <td>Python</td>
      <td>General purpose</td>
      <td>1991</td>
    </tr>
    <tr>
      <td>Go</td>
      <td>Systems / concurrent</td>
      <td>2009</td>
    </tr>
  </tbody>
</table>

### Details / Summary (collapsible)

<details>
  <summary>Click to expand hidden content</summary>
  This content is normally hidden until the user clicks the summary.
  In a PDF context we render all content but visually group it.
</details>

### HTML Comments (should be invisible)

<!-- This comment should not appear in the PDF output -->

This paragraph is after a comment. The comment above should be invisible.

---

## Mixed Markdown + HTML

This section mixes native Markdown with inline HTML.

Here's **bold Markdown**, *italic Markdown*, and <mark>highlighted HTML</mark> all together.

- Markdown list item
- Item with <sup>superscript</sup> HTML inline
- Item with <code>code</code> HTML inline

> Markdown blockquote with an <a href="https://example.com">HTML link</a> inside.

---

## Entity Decoding

HTML entities in raw HTML are decoded before rendering:

- `&amp;` → <span>A &amp; B</span>
- `&lt;` and `&gt;` → <span>&lt;tag&gt;</span>
- `&nbsp;` → <span>non&nbsp;breaking&nbsp;space</span>
- `&mdash;` → <span>em&mdash;dash</span>
- `&copy;` → <span>&copy; 2026</span>
- `&#65;` → <span>&#65;</span> (numeric decimal = 'A')
- `&#x41;` → <span>&#x41;</span> (numeric hex = 'A')

---

*End of HTML test document.*
