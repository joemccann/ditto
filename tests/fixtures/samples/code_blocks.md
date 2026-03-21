# Code Blocks

## Rust

```rust
use std::collections::HashMap;

fn word_count(text: &str) -> HashMap<&str, usize> {
    let mut map = HashMap::new();
    for word in text.split_whitespace() {
        *map.entry(word).or_insert(0) += 1;
    }
    map
}

fn main() {
    let counts = word_count("hello world hello");
    println!("{:?}", counts);
}
```

## Python

```python
from typing import Optional

def fibonacci(n: int, memo: Optional[dict] = None) -> int:
    if memo is None:
        memo = {}
    if n in memo:
        return memo[n]
    if n <= 1:
        return n
    result = fibonacci(n - 1, memo) + fibonacci(n - 2, memo)
    memo[n] = result
    return result
```

## JavaScript / TypeScript

```typescript
interface User {
  id: number;
  name: string;
  email?: string;
}

async function fetchUser(id: number): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}
```

## SQL

```sql
SELECT u.name, COUNT(o.id) AS order_count
FROM users u
LEFT JOIN orders o ON o.user_id = u.id
WHERE u.created_at > '2024-01-01'
GROUP BY u.id, u.name
ORDER BY order_count DESC
LIMIT 10;
```

## Shell

```bash
#!/usr/bin/env bash
set -euo pipefail

OUTPUT="${1:-output.pdf}"
INPUT="${2:-README.md}"

md-to-pdf --preset a4 --toc "$INPUT" "$OUTPUT"
echo "Generated: $OUTPUT"
```

## Plain (no language tag)

```
this is plain text
no syntax highlighting
just monospace
```

## Inline Code

Use `git commit -m "message"` to commit and `git push origin main` to push.
