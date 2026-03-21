# Regression Test Fixtures

Each section targets a previously-discovered or plausible rendering bug.

---

## REG-001: Dollar Sign Escaping

The price is $9.99 — should not open math mode.

Two dollars: $50 and $100 on the same line.

A paragraph with $-signs at line end: pay $

---

## REG-002: Hash Escaping

The #rust IRC channel and #1 trending should not trigger Typst function calls.

---

## REG-003: Duplicate Heading Labels

Two identical headings must produce unique Typst labels.

### Overview

First overview section.

### Overview

Second overview section — the label must be deduplicated.

---

## REG-004: Ordered List Non-1 Start

A list starting at 3:

3. Third item
4. Fourth item
5. Fifth item

---

## REG-005: Nested Task List Spacing

Parent and child must not have a blank line between them:

- [x] Parent task
  - [ ] Child task
  - [x] Another child

---

## REG-006: Table With Formatted Header Cells

| **Bold** header | *Italic* header | `Code` header |
|-----------------|-----------------|---------------|
| data            | data            | data          |

---

## REG-007: Blockquote With Code Block Inside

> Here is a code block inside a blockquote:
>
> Some quoted text after.

---

## REG-008: Footnote Definition Before Reference

[^early]: This definition appears before its reference in the source.

Now we reference it[^early] in the text.

---

## REG-009: Email Autolink @ Escaping

Contact us at <support@example.com> — the @ must be escaped for Typst.

---

## REG-010: Math in Table Cells

| Expression | Value |
|------------|-------|
| $x^2$      | quadratic |
| $\sqrt{x}$ | root |

---

## REG-011: Long Code Block

```python
# Long code block to test vertical pagination
import sys
from pathlib import Path

def process_file(path: Path) -> dict:
    """Process a single file and return statistics."""
    result = {
        "path": str(path),
        "size": path.stat().st_size,
        "lines": 0,
        "words": 0,
        "chars": 0,
    }
    with path.open() as f:
        for line in f:
            result["lines"] += 1
            result["words"] += len(line.split())
            result["chars"] += len(line)
    return result

def main(directory: str) -> None:
    root = Path(directory)
    files = list(root.rglob("*.py"))
    print(f"Processing {len(files)} Python files...")
    totals = {"lines": 0, "words": 0, "chars": 0}
    for f in sorted(files):
        stats = process_file(f)
        totals["lines"] += stats["lines"]
        totals["words"] += stats["words"]
        totals["chars"] += stats["chars"]
        print(f"  {f.relative_to(root)}: {stats['lines']} lines")
    print(f"Total: {totals['lines']} lines, {totals['words']} words")

if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else ".")
```

---

## REG-012: Empty Document Sections

### Empty Section

### Another Empty Section

Content continues here.

---

## REG-013: Unicode in Headings

### 日本語の見出し

### Über Alles

### Café Résumé

---

## REG-014: Inline Code With Special Characters

`$variable` should not start math mode.

`#include <stdio.h>` should not trigger Typst markup.

`path\to\file` with backslashes.

`array[0]` with square brackets.
