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
assets are not embedded in the binary. System fallback fonts are opt-in via
`TemplateOptions::use_system_fonts`; the bundled plain template enables them,
while on-disk templates use only their own fonts by default.

## Arguments and validation

The `#let template(...)` signature is the contract. Typst's parser reads it and
the renderer matches front matter keys by name:

- **positional** (no default) — required, passed by position; missing keys
  abort before Typst (`… missing front matter key(s): title`);
- **named** (with default) — optional, passed by name when present, otherwise
  the Typst default applies;
- **`body`** — must be positional; receives the document as a list of
  elements, each with `content` (Typst) and `data` (`none` or table rows);
- values are Typst content (`subtitle: Hello` → `[Hello]`); front matter is
  flat and case-sensitive.

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

## Errors

An explicit `--template` is loaded while clap parses it, so a missing file or an
invalid signature is a usage error (exit 2). The bundled default is loaded at
render time, so its failures are runtime errors (exit 1). See
[development.md](development.md) for the error chain printed by the CLI.

## Tables as data

Every element has a `content` field. Table elements also carry plain-text rows
in `data`, so a template can render the document and compute from the tables
without a separate argument:

```typst
#let template(invoice, body) = {
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

The invoice template in `templates/invoice/` uses this to sum the hours column
and compute the invoice amount from the same Markdown file that produces the
Leistungsübersicht.

## Bundled template

The default is `cli/src/plain.typ`, a single file with no assets. Its signature
is `plain(title: none, subtitle: none, body)` and it uses system Arial (with
Liberation Sans / DejaVu Sans fallback).

## Fonts and licences

Templates supply their own fonts: put static font files in `<template>/fonts/`
together with their licences and provenance notes. The bundled plain template
uses system fonts; the invoice example in `templates/invoice/` ships Latin
Modern Mono with its licence.
