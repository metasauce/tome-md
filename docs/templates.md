# Templates

A template is one folder: a `.typ` entry point plus everything it needs.

```
my-template/
├─ template.typ               # template function + signature
├─ logo.png                   # optional asset, read on demand
└─ fonts/                     # optional, indexed up front
   ├─ Inter-Regular.ttf
   └─ …OFL.txt / …PROVENANCE.md
```

Pass the entry point explicitly with `-t/--template`; there is no discovery.

## How a template is loaded

`Template::from_path` (`core/src/templates/template.rs`) does this:

1. read `template.typ` and inspect the `#let template(...)` signature,
2. index font metadata from `<template>/fonts/` (and the template root) with
   `typst-kit`'s `FontStore`; font data is loaded only when used,
3. serve the other files next to `template.typ` through the Typst virtual file
   system, so `logo.png` and friends work and are only read when Typst asks,
4. match front matter to the signature, pass the element list as `body` and
   call the template.

The folder is self-contained: copy it anywhere and it still works, and its
assets are not embedded in the binary. Templates can also import Typst Universe
packages by name; those are downloaded on first use and then served from the
package cache. System fallback fonts are opt-in via
`TemplateOptions::use_system_fonts`; the bundled plain template enables them.
On-disk templates always get Typst's embedded base fonts plus their own fonts.

## Packages

A template can import a Typst Universe package directly:

```typst
#import "@preview/letter-pro:3.0.0": letter-generic

#let template(title, body) = {
  letter-generic({
    strong(title)
    v(0.65em)
    for element in body {
      element.content
    }
  })
}
```

Missing packages are downloaded into Typst's package cache on first render and
resolved from there afterwards. Pin the version in the import: templates that
use `@preview` packages are not fully self-contained until the package is
cached.

## Arguments and validation

The `#let template(...)` signature is the contract. Typst's parser reads it and
the renderer matches front matter keys by name:

- **positional** (no default) — required, passed by position; missing keys
  abort before Typst (`… missing front matter key(s): title`);
- **named** (with default) — optional, passed by name when present, otherwise
  the Typst default applies;
- **`body`** — must be positional; receives the document as a list of
  elements, each with `content` (Typst) and `data` (`none` or table rows);
- scalar values are Typst content (`subtitle: Hello` → `[Hello]`); nested
  mappings become Typst dictionaries and sequences become arrays; front matter
  is case-sensitive.

Arguments are emitted in signature order. Duplicate parameters, destructuring,
`..args` and a named or missing `body` are rejected at load time. Unused keys
are ignored and logged at debug level.

A template can declare anything, for example:

```typst
#let template(title, subtitle: none, body) = {
  // cover page, headings, body…
  for element in body {
    element.content
  }
}
```

with input:

```markdown
---
title: Notes
subtitle: A short example
---

# Content
```

Only `title` is required above; `subtitle` is optional.

Nested YAML keys become dictionaries, so parameters can group related values:

```markdown
---
sender:
  name: Jane Doe
  address: 1 Example Street
recipient:
  name: Example GmbH
  address: 2 Sample Road
---
```

```typst
#let template(sender, recipient, body) = {
  sender.name
  sender.address
  recipient.name
  recipient.address
  for element in body {
    element.content
  }
}
```

The template receives `sender` and `recipient` as Typst dictionaries. See
[markdown.md](markdown.md#front-matter) for the supported YAML forms.

## Errors

An explicit `--template` is loaded while clap parses it, so a missing file or an
invalid signature is a usage error (exit 2). The bundled default is loaded at
render time, so its failures are runtime errors (exit 1). See
[development.md](development.md) for the error chain printed by the CLI.

## GitHub alerts

GitHub alert blockquotes render through a small Typst helper that the renderer
injects only into documents containing an alert. Existing templates continue to
style ordinary blockquotes normally. Alerts use a self-contained GitHub-style
block and are not restyled by `show quote` rules; a template can deliberately
override `_tome_md_github_alert` if it needs a different appearance.

A table inside a blockquote is emitted inline with the quote content so the
quote structure stays balanced. Such inline tables are not exposed as separate
`Element::Table` data.

## Tables as data

Every element has a `content` field. Table elements also carry plain-text rows
in `data`, so a template can render the document and compute from the tables
without a separate argument:

```typst
#let template(title, body) = {
  let tables = body
    .filter(element => element.data != none)
    .map(element => element.data)
  let rows = tables.at(0).slice(1)
  // rows.at(0).at(0) is the first cell of the first body row
  for element in body {
    element.content
  }
}
```

`data` is `none` for text elements and an array of rows for table elements.
The first row is the header. Cells are plain text: emphasis, code and links
lose their formatting, and a link becomes its label.
The field is populated by the parser and is never read from front matter.

This lets a template render the document and still compute totals or summaries
from the tables in it.

## Bundled template

The default is `cli/src/plain.typ`, a single file with no assets. Its signature
is `plain(title: none, subtitle: none, body)` and it uses system Arial (with
Liberation Sans / DejaVu Sans fallback).

## Fonts and licences

Typst's embedded base fonts (Libertinus Serif, New Computer Modern, DejaVu
Sans Mono) are always available, so a template without a `fonts/` folder still
renders. Most example templates rely on those fonts; `letter-din` bundles
Source Sans 3. Put additional static fonts in `<template>/fonts/` together with
their licences and provenance notes; the bundled plain template uses system
fonts.
