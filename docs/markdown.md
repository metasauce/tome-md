# Markdown support

Conversion uses [`pulldown-cmark`](https://github.com/raphlinus/pulldown-cmark)
with all GFM options enabled. The emitter lives in `core/src/markdown.rs`.

Supported:

- headings, paragraphs, emphasis, strong, strikethrough, super- and subscript
- inline code and fenced code blocks (the language hint is kept on the fence)
- ordered lists (including start numbers), unordered lists, nesting, task markers
- block quotes, tables with header rows and column alignment
- links, horizontal rules, hard breaks

Partially supported or not rendered:

- **images** — alt text is rendered as emphasis; the image itself is not fetched
- **raw HTML** — dropped
- **math** — dropped
- **footnotes** — not emitted as footnotes

Unsupported constructs do not fail the render; the emitter just skips what it
does not model.

## Front matter

An optional block at the very top of the file supplies template arguments:

```markdown
---
title: Notes
date: 2026-01-02
---

# Content
```

The fence must be `---` on its own line. Entries are literal `key: value` pairs
— this is not full YAML. Surrounding quotes are stripped, keys are
case-sensitive and must fit on one line. Values are inserted into the template
as Typst content, so `subtitle: Hello` becomes `[Hello]`.

Unknown keys are ignored and logged at debug level.

## Escaping

Plain text is escaped so that Typst markup characters do not change the
meaning of the prose, and links become `#link("url")[text]`. The body is
finished with blank lines between blocks so the generated Typst source stays
readable with `--keep-typ`.
