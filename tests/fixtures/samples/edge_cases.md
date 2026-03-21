# Edge Cases and Regression Fixtures

Documents designed to catch tricky rendering bugs.

---

## Dollar Signs in Plain Text

Price: $9.99 per unit, or buy two for $18.

Special offer: save $0.50 on every $10 purchase.

Math still works inline: the cost $C = p \times q$ where $p$ is price.

---

## Hash Characters in Text

The HTML `<h1>` tag is written as `# Heading` in Markdown.

Function call: `foo#bar` is not valid Typst without escaping.

---

## Brackets in Text

Array access: `arr[0]` gets the first element.

Typst blocks use `[content]` notation.

---

## Angle Characters

`a < b` and `b > a` are comparisons.

Generic types: `Vec<T>` in Rust.

---

## Backslash Sequences

Windows path: `C:\Users\name\Documents`.

A literal backslash: use `\\` in code.

---

## Mixed Emphasis

***Bold and italic*** text.

**Bold with `inline code` inside**.

*Italic with **nested bold** inside*.

---

## Empty and Single-Character Items

- a
- b
- c

1. x
2. y
3. z

---

## Very Long Words and URLs

Supercalifragilisticexpialidocious is a long word.

A very long URL: https://example.com/very/long/path/that/might/overflow/the/line/width/by/quite/a/bit/indeed

---

## Blockquote Nesting

> First level blockquote.
>
> > Second level — deeper.
> >
> > Still inside nested.
>
> Back to first level.

---

## Code With Special Characters

```python
# Python dict with special keys
data = {
    "price": "$9.99",
    "label": "#featured",
    "formula": "$E=mc^2$",
    "path": "C:\\Users\\name",
}
```

---

## Table With Inline Formatting

| Feature | Code | Status |
|---------|------|--------|
| **Bold** | `**text**` | ✅ |
| *Italic* | `*text*` | ✅ |
| ~~Strike~~ | `~~text~~` | ✅ |
| `Code` | backtick | ✅ |

---

## Zero-Width and Whitespace

Trailing spaces at end of line  
should produce a line break.

Multiple blank lines below should collapse:



Back to normal.

---

## Setext-Style Heading (if supported)

This is a heading
=================

This is a subheading
--------------------

---

## Fenced Code With No Language

```
plain
text
here
no
syntax
```

---

## Single-Item Lists

- Only one item

1. Only one numbered item

---

## Deeply Nested List

- Level 1
  - Level 2
    - Level 3
      - Level 4
        - Level 5
          - Level 6

---

## Task List With All States

- [x] Done
- [ ] Not done
- [x] Also done
- [ ] Also not done
