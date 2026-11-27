# Markdown support

TomeMD reads CommonMark and the common GitHub extensions.

Supported:

- headings, paragraphs, emphasis, strong, strikethrough
- superscript and subscript via inline `<sup>` / `<sub>` tags
- inline code and fenced code blocks (the language hint is kept on the fence)
- raw Typst blocks through a `{=typst}` fence, see
  [Raw Typst blocks](#raw-typst-blocks)
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

Unsupported constructs do not fail the render; TomeMD skips what it cannot
convert.

## Raw Typst blocks

A fence tagged `{=typst}` goes to Typst unchanged, for layout Markdown cannot
express:

```{=typst}
#v(1em)
#pagebreak()
```

Every other language hint stays a code block, `typst` included.

> [!WARNING]
> The block runs with the template's reach: it can read files in the template
> folder and import packages.

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
title: |-
  My Awesome Project
  with a very long Name
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
Literal block scalar line breaks are preserved as Typst line breaks; use `|-` to strip the final newline.

Invalid YAML aborts rendering with an `invalid front matter` error.

By default, TomeMD uses the whole top-level mapping. To keep site-generator
metadata such as Docusaurus fields separate from TomeMD template arguments,
select a mapping with `--fm-section`:

```yaml
---
title: Docusaurus page title
slug: /getting-started
tome:
  title: TomeMD document title
  author: Jane Doe
---
```

Run `tome page.mdx --fm-section tome`; only keys inside `tome` are matched to
template parameters. If the section is absent, TomeMD sees no front-matter
values. If it is present but is not a YAML mapping, rendering fails with an
`invalid front matter` error. Without `--fm-section`, the existing flat
front-matter behavior is unchanged.

Unknown keys in the selected mapping are ignored and logged at debug level.

## Escaping

Plain text is escaped so that Typst markup characters do not change the
meaning of the prose, and links become `#link("url")[text]`. The body is
finished with blank lines between blocks so the generated Typst source stays
readable with `--keep-typ`.
