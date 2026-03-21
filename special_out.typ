#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
= Special Characters & Escaping <special-characters-escaping>

== Typst-Special Characters in Text <typst-special-characters-in-text>

Text with a hash: The language uses hash characters for attributes.

Backslash path: Use `C:\Users\name` in Windows.

Brackets: A function call like `foo[bar]` is common in Typst.

Curly braces: `{key: value}` is JSON notation.

Double-quote: She said “hello world”.

Dollar sign: Prices like $9.99 appear in documents.

== HTML Entities in Raw HTML <html-entities-in-raw-html>

Fish & Chips cost less than 10 dollars today.

== Autolinks <autolinks>

Visit #link("https://example.com") for more.

== Nested Emphasis <nested-emphasis>

#emph[#strong[Bold and italic]] text, then #emph[just italic] and #strong[just bold].

== Strikethrough <strikethrough>

The answer was #strike[42] actually 43.

== Long Line (no wrapping test) <long-line-no-wrapping-test>

This is a very long line that tests how the renderer handles text that extends beyond the typical line length of a markdown document without any natural line breaks or wrapping opportunities present.

== Unicode <unicode>

Emoji: 🦀 🔥 ✨

CJK: 日本語テスト — 中文测试 — 한국어 테스트

Accented: café, naïve, façade, über, résumé


