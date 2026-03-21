---
toc: true
toc_depth: 3
toc_title: Contents
---

# Comprehensive GFM Reference

A complete reference document exercising all GitHub Flavored Markdown features.

---

## 1. Headings (ATX Style)

# H1 Heading
## H2 Heading
### H3 Heading
#### H4 Heading
##### H5 Heading
###### H6 Heading

---

## 2. Text Formatting

Regular paragraph text.

**Bold text** and *italic text* and ***bold italic***.

~~Strikethrough text~~ is supported.

`Inline code` with backticks.

Text with a hard break (two trailing spaces):  
Second line after hard break.

---

## 3. Blockquotes

> Simple blockquote with a single paragraph.

> Multi-line blockquote.
>
> Second paragraph inside the quote.
>
> **Bold** and *italic* work inside blockquotes too.

---

## 4. Lists

### Unordered

- Alpha
- Beta
  - Beta sub-one
  - Beta sub-two
    - Deep level
- Gamma

### Ordered

1. First
2. Second
   1. Sub-item one
   2. Sub-item two
3. Third

### Starting at non-1

5. Fifth item
6. Sixth item
7. Seventh item

### Task List

- [x] Completed item
- [ ] Incomplete item
- [x] Another completed
  - [ ] Nested incomplete
  - [x] Nested complete

---

## 5. Code

Inline: `println!("Hello, world!");`

Fenced Rust:

```rust
use std::fmt;

struct Point { x: f64, y: f64 }

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

Fenced Python:

```python
def factorial(n: int) -> int:
    return 1 if n <= 1 else n * factorial(n - 1)
```

Plain (no language):

```
plain text code block
no syntax highlighting
```

---

## 6. Tables

Basic table:

| Col A | Col B | Col C |
|-------|-------|-------|
| 1     | 2     | 3     |
| 4     | 5     | 6     |

With alignment:

| Left | Center | Right |
|:-----|:------:|------:|
| a    |   b    |     c |
| d    |   e    |     f |

With formatted cells:

| Feature    | Supported | Notes              |
|------------|:---------:|-------------------|
| **Bold**   | ✅        | Works in cells    |
| *Italic*   | ✅        | Works in cells    |
| `Code`     | ✅        | Works in cells    |
| ~~Strike~~ | ✅        | Works in cells    |

---

## 7. Links and Autolinks

Explicit link: [Rust Language](https://www.rust-lang.org)

Autolink URL: https://typst.app

Angle-bracket URL: <https://github.com>

Email autolink: <user@example.com>

---

## 8. Horizontal Rules

Above the rule.

---

Below the rule.

***

Another style.

---

## 9. Math

Inline: $x = \frac{-b \pm \sqrt{b^2 - 4 a c}}{2 a}$

Display block:

$$\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}$$

Greek letters: $\alpha, \beta, \gamma, \Delta, \Omega$

Operators: $\sin\theta$, $\cos\theta$, $\lim_{x \to 0}$

Matrix:

$$\begin{bmatrix} a & b \\ c & d \end{bmatrix}$$

---

## 10. Footnotes

Markdown was created by John Gruber[^gruber] and Aaron Swartz.

CommonMark provides a formal specification[^commonmark].

GFM extends CommonMark[^gfm] with additional features.

[^gruber]: John Gruber published the original Markdown specification in 2004.
[^commonmark]: CommonMark is at https://commonmark.org
[^gfm]: GitHub Flavored Markdown spec: https://github.github.com/gfm/

---

## 11. Definition Lists

Markdown
: A lightweight markup language for creating formatted text.

Typst
: A new typesetting system, written in Rust, designed as a LaTeX alternative.
: Available at https://typst.app

CommonMark
: A strongly defined, highly compatible specification of Markdown.

---

## 12. HTML Inline Elements

Text with <strong>HTML bold</strong> and <em>HTML italic</em>.

A line break: before<br>after.

---

## 13. Special Characters

Dollar signs: $9.99 and $100.

Hash marks: #1 trending, the #rust channel.

Backslash path: `C:\Users\name`.

---

## End of Reference

All GFM features above should render correctly to PDF.
