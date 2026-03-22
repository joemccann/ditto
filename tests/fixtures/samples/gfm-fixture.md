# GFM Fidelity Test Fixture

This document exercises every improved GFM feature in a single renderable file.

---

## Nested Lists

### Bullet nesting (3 levels)

- Top-level item A
  - Second-level item B
    - Third-level item C
  - Second-level item D
- Top-level item E

### Ordered nesting

1. First step
2. Second step
   1. Sub-step one
   2. Sub-step two
      1. Deep sub-step
3. Third step

### Mixed bullet + ordered

- Category Alpha
  1. Alpha item one
  2. Alpha item two
- Category Beta
  1. Beta item one

### Ordered list starting at 3

3. Already third
4. Fourth
5. Fifth

---

## Task Lists

- [x] Complete task
- [ ] Incomplete task
- [x] Another done item
  - [ ] Nested incomplete
  - [x] Nested complete
- [ ] Final pending

---

## Tables with Alignment Markers

| Name        |  Score  | Grade |
|:-----------|:-------:|------:|
| Alice       |   95    |  A+   |
| Bob         |   82    |  B    |
| Carol       |   77    |  C+   |
| Dave        |  100    |  A+   |

Table without explicit alignment (defaults to left):

| Foo | Bar | Baz |
|-----|-----|-----|
| 1   | 2   | 3   |

---

## Autolinks

Plain URL autolink: https://example.com

Angle-bracket URL autolink: <https://www.rust-lang.org>

Email autolink: <user@example.com>

Explicit link (label differs from URL): [GitHub](https://github.com)

---

## Footnotes

Markdown was invented by John Gruber.[^gruber]

Typst is a new typesetting system.[^typst]

Both footnote references appear in text; definitions are collected below.

[^gruber]: John Gruber created Markdown in 2004 with Aaron Swartz.

[^typst]: Typst was designed to be an alternative to LaTeX with a simpler syntax.

---

## Definition Lists

Markdown
:   A lightweight markup language created in 2004. Uses plain text formatting syntax.

CommonMark
:   A strongly defined, highly compatible specification of Markdown.

GFM (GitHub Flavored Markdown)
:   GitHub's Markdown dialect that extends CommonMark with tables, task lists,
    strikethrough, and autolinks.
:   Also includes footnotes and definition lists via extensions.

---

## Combined Inline Elements

This text has ~~strikethrough~~ and **bold** and *italic* and `inline code`.

An expression: $E = mc^2$ and a display equation:

$$\int_0^1 x^2\,dx = \frac{1}{3}$$

---

## Blockquote

> This is a blockquote with **formatting** and _emphasis_.
>
> It spans multiple paragraphs.

---

## Fenced Code Block

```rust
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    println!("{}", greet("world"));
}
```

---

## That's all!

End of GFM fidelity fixture. All sections above should render correctly.
