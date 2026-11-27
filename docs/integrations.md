# Integrating with other tools

When TomeMD is part of a larger toolchain, it is often useful to select a
layout from the document's front matter. Site generators such as Docusaurus
already use front matter for page metadata, and a GitHub Actions job can render
different documents without keeping a separate template-selection map in the
workflow. Put TomeMD-specific values in a front-matter subsection and select it
with `--fm-section`:

```yaml
---
title: Docusaurus page title
slug: /reports/quarterly
tome:
  layout: report
  title: Quarterly report
  subtitle: Results and outlook
  author: Jane Doe
  date: 2026-04-15
---
# Overview

Document content goes here.
```

Then pass a single registry template to TomeMD and select the `tome` mapping:

```sh
tome docs/reports/quarterly.mdx \
  --template templates/registry.typ \
  --fm-section tome \
  --output build/quarterly.pdf \
  --force
```

Docusaurus keeps its own `title` and `slug` fields at the top level; TomeMD
matches only the keys nested under `tome`. The `layout` value is also passed to
the Typst template, where the registry uses it to choose a layout. Template
selection therefore stays in the Typst project rather than adding
Docusaurus-specific behavior to TomeMD.

## Registry template

The registry is an ordinary Typst template. Import each layout and dispatch on
the `layout` value. Front-matter scalar strings arrive as Typst content, so
compare them with content blocks such as `[report]`:

```typst
#import "report.typ": template as report
#import "memo.typ": template as memo

#let template(
  layout,
  title,
  subtitle: none,
  author: none,
  date: none,
  body,
) = {
  if layout == [report] {
    report(title, body, subtitle: subtitle, author: author, date: date)
  } else if layout == [memo] {
    memo(title, body, author: author, date: date)
  } else {
    panic("unknown layout " + repr(layout) + "; use `report` or `memo`")
  }
}
```

The `report.typ` and `memo.typ` files each expose a `template` function with
arguments that match the values passed in the registry. They can import shared
helpers, fonts, or images from the same project. Keep the error for unknown
layout IDs explicit: it makes misspelled or unsupported front matter fail
clearly instead of silently choosing a different design.

## Project structure

A Docusaurus repository might organize its content, layouts, shared resources,
and workflow like this:

```
.github/
└─ workflows/
   └─ render-pdf.yml
docs/
└─ reports/
   └─ quarterly.mdx
templates/
├─ registry.typ
├─ report.typ
├─ memo.typ
└─ shared/
   ├─ brand.typ
   └─ logo.svg
```

The registry is the entry point passed to TomeMD. The other `.typ` files and
assets are resolved relative to the project, so layouts can share common
resources rather than duplicating them. See [Templates](templates.md#sharing-resources-and-routing-layouts)
for more on organizing a multi-layout Typst project.

## Render in GitHub Actions

This workflow renders the example page on pushes that affect its content,
templates, or workflow. It downloads the Linux x86-64 binary from the latest
GitHub release, uses the registry entry point, and publishes the resulting PDF
as a workflow artifact:

```yaml
name: Render report PDF

on:
  push:
    paths:
      - "docs/reports/quarterly.mdx"
      - "templates/**"
      - ".github/workflows/render-pdf.yml"
  workflow_dispatch:

permissions:
  contents: read

jobs:
  render:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install TomeMD
        run: |
          mkdir -p "$HOME/.local/bin"
          curl --fail --location \
            --output "$HOME/.local/bin/tome" \
            https://github.com/metasauce/tome-md/releases/latest/download/tome-x86_64-unknown-linux-gnu
          chmod +x "$HOME/.local/bin/tome"
          echo "$HOME/.local/bin" >> "$GITHUB_PATH"

      - name: Render PDF
        run: |
          mkdir -p build
          tome docs/reports/quarterly.mdx \
            --template templates/registry.typ \
            --fm-section tome \
            --output build/quarterly.pdf \
            --force

      - uses: actions/upload-artifact@v4
        with:
          name: quarterly-report
          path: build/quarterly.pdf
```

The release URL above selects the Linux x86-64 binary; use the matching asset
for another runner architecture or operating system. For reproducible builds,
replace `latest` with a specific release tag. The first render may also need
network access if the templates import uncached Typst Universe packages.

TomeMD renders Markdown rather than executing Docusaurus. If a page uses MDX
components, JSX, or Docusaurus-only syntax, preprocess or simplify that content
before passing it to `tome`; ordinary YAML front matter and Markdown content can
be passed directly as shown above.

A registry imports all its layouts up front. This is a simple, maintainable
choice for a modest set of templates, but imports are not lazy, so a project
with many large layouts or packages may use more memory than rendering a single
standalone template.
