# Templates

A template is a Typst project: it has a single `.typ` entry point plus the
resources the entry point uses. This can e.g. be other `.typ` files, fonts, or
images.

A simple template is usually a single self-contained folder. However, a more involved
project with several layouts can share resources and expose a registry entry point that
routes to the right layout. Typst is flexible enough to organize this however fits
your needs.

```
my-template/
├─ template.typ               # template function + signature
├─ logo.png                   # optional asset, read on demand
└─ fonts/                     # optional, indexed up front
   ├─ Inter-Regular.ttf
   └─ …OFL.txt / …PROVENANCE.md
```

Pass the entry point explicitly with `-t/--template`; there is no discovery.

## Sharing resources and routing layouts

For example, a project can keep shared page styling in one module and import it
from multiple layouts:

```
templates/
├─ shared/
│  ├─ brand.typ
│  └─ logo.svg
├─ report.typ
├─ memo.typ
└─ registry.typ
```

```typst
// report.typ and memo.typ can both import this helper.
#import "shared/brand.typ": pageHeader

#let template(title, body) = {
  pageHeader(title)
  for element in body {
    element.content
  }
}
```

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
[markdown.md](../markdown.md#front-matter) for the supported YAML forms.

## Errors

An explicit `--template` is loaded while clap parses it, so a missing file or an
invalid signature is a usage error (exit 2). The bundled default is loaded at
render time, so its failures are runtime errors (exit 1). See
[development.md](../development.md) for the error chain printed by the CLI.

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

## Fonts

Typst's embedded base fonts (Libertinus Serif, New Computer Modern, DejaVu
Sans Mono) are always available. To add fonts add them next to the template entry point in a `fonts/` folder, together with their licences and provenance notes.

## Bundled template

The default is `cli/src/plain.typ`, a single file with no assets. Its signature
is `plain(title: none, subtitle: none, body)` and it uses system Arial (with
Liberation Sans / DejaVu Sans fallback).
