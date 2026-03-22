# HTML-in-Markdown Test Document

This document exercises raw HTML tags inside Markdown.

---

## Inline HTML Tags

Normal paragraph with <b>bold via b-tag</b>, <strong>bold via strong</strong>,
<i>italic via i-tag</i>, <em>italic via em-tag</em>, and <u>underline</u>.

Strikethrough: <s>old price</s> and <del>deleted text</del>.

Inserted text: <ins>newly added</ins>.

Highlighted text: <mark>this is highlighted</mark>.

Small text: <small>fine print here</small>.

### Subscript and Superscript

H<sub>2</sub>O is water. E = mc<sup>2</sup> is Einstein's famous equation.

### Keyboard / Monospace

Press <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>Del</kbd> to reset.

Shell output: <samp>Permission denied</samp>.

Inline code via HTML: <code>printf("Hello\n");</code>

### Semantic

<cite>The Elements of Style</cite> by Strunk and White.

<abbr title="HTML">HTML</abbr> is the backbone of the web.

### Links

Visit <a href="https://www.rust-lang.org">The Rust Programming Language</a>.

### Span with CSS styles

Text with <span style="color: red">red colour</span> and
<span style="color: #0077cc">hex blue colour</span>.

Text with <span style="font-size: 18px">large font</span>.

### Line break

First line.<br>Second line right after.

### Unknown / pass-through tags

This has a <blink>blinking tag</blink> that is stripped but content preserved.

---

## Block HTML Elements

<p>This is a raw HTML paragraph.</p>

<blockquote>
This is a raw HTML blockquote with a left-border inset.
</blockquote>

<hr>

<h3>An H3 Inside HTML</h3>

<pre><code>fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}</code></pre>

<ul>
  <li>First item</li>
  <li>Second item</li>
  <li>Third item with <strong>bold</strong></li>
</ul>

<ol>
  <li>Step one</li>
  <li>Step two</li>
  <li>Step three</li>
</ol>

<dl>
  <dt>Markdown</dt>
  <dd>A lightweight markup language for creating formatted text.</dd>
  <dt>Typst</dt>
  <dd>A new markup-based typesetting language.</dd>
</dl>

<table>
  <thead>
    <tr>
      <th>Language</th>
      <th>Paradigm</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Rust</td>
      <td>Systems / multi-paradigm</td>
    </tr>
    <tr>
      <td>Python</td>
      <td>General purpose</td>
    </tr>
  </tbody>
</table>

<details>
  <summary>Click to expand</summary>
  Hidden content revealed.
</details>

<!-- This comment should not appear in the PDF output -->

This paragraph is after an HTML comment. The comment should be invisible.

---

## Mixed Markdown + HTML

**Bold Markdown**, *italic Markdown*, and <mark>highlighted HTML</mark> all together.

- Markdown item
- Item with <sup>superscript</sup> HTML
- Item with <code>code</code> HTML

---

## Entity Decoding

- A &amp; B in a span: <span>A &amp; B</span>
- Copyright: <span>&copy; 2026</span>
- Em dash: <span>before&mdash;after</span>
- Numeric: <span>&#65;</span> = A

---

*End of HTML test document.*
