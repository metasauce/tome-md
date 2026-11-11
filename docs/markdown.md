# Markdown support

Conversion uses [`pulldown-cmark`](https://github.com/raphlinus/pulldown-cmark)
with all GFM options enabled. The emitter lives in `core/src/markdown.rs`.

Supported:

- headings, paragraphs, emphasis, strong, strikethrough
- superscript and subscript via inline `<sup>` / `<sub>` tags
- inline code and fenced code blocks (the language hint is kept on the fence)
- ordered lists (including start numbers), unordered lists, nesting, task markers
- block quotes, tables with header rows and column alignment
- GitHub alert blockquotes (`> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`)
- images with local paths; the Markdown alt text is kept as the image alt
- links, horizontal rules, hard breaks

Partially supported or not rendered:

- **remote images** — not fetched; use a local image path
- **raw HTML** — dropped, except inline `<sup>` / `<sub>`
- **math** — dropped
- **footnotes** — not emitted as footnotes

Image paths are resolved in the template folder by Typst, so keep referenced
images next to the template or in one of its subfolders.

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

The fence must be `---` on its own line. The content between the fences is
YAML, so nested mappings, sequences, multiline strings and quoted values work:

```markdown
---
sender:
  name: Jane Doe
  address: 1 Example Street
recipient:
  name: Example GmbH
  address: 2 Sample Road
tags: [invoice, reminder]
note: >
  Folded multiline text
---

# Content
```

Top-level keys are matched to template parameters; nested mappings become Typst
dictionaries (`sender.name`, `sender.address`), sequences become arrays, and
scalars are inserted as Typst content (`subtitle: Hello` becomes `[Hello]`).
YAML `null` becomes Typst `none`. Keys are case-sensitive.

Invalid YAML aborts rendering with an `invalid front matter` error.

Unknown keys are ignored and logged at debug level.

## Escaping

Plain text is escaped so that Typst markup characters do not change the
meaning of the prose, and links become `#link("url")[text]`. The body is
finished with blank lines between blocks so the generated Typst source stays
readable with `--keep-typ`.
