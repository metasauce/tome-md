# Integrating with other tools

When TomeMD is part of a larger toolchain, it is often useful to select a
layout from the document's front matter. Site generators such as Docusaurus
already use front matter for page metadata, and a GitHub Actions job can render
different documents.

A common problem to solve when integrating tomeMD with site generators is that
you do not want to specify the template to use for a markdown file as part of the
command that invokes `tome`. Rather, the command should be the same for all documents
of the workspace, and the document should be able to specify the template to use,
e.g. via its frontmatter.

The following example for docusaurus shows how to do this with tomeMD and Docusaurus.

## Docusaurus

A Docusaurus repository might organize its content, layouts, shared resources,
and workflow like this:

```
.github/
└─ workflows/
   └─ render-pdf.yml
docs/
└─ reports/
   ├─ quarter_1.mdx
   └─ quarter_2.mdx
templates/
├─ registry.typ
├─ report.typ
├─ memo.typ
└─ shared/
   ├─ brand.typ
   └─ logo.svg
```

### Registry template

The `registry.typ` acts as the central entry point passed to TomeMD.
The other `.typ` files and assets are resolved relative to the project,
so layouts can share common resources rather than duplicating them.

It is an ordinary Typst template that imports each layout and dispatches on
the `layout` value. Front-matter scalar strings arrive as Typst content, so
we can use them to route into other `.typ` files.

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

### Frontmatter

Docusaurus keeps its own `title` and `slug` fields at the top level.
To keep things organize we will later pass `--fm-section` so that TomeMD
matches only the keys nested under `tome`.

The `layout` value is what is used in our registry from above.
(This way, template selection stays in the Typst project, and we do not need
any special wiring inside TomeMD.)

```yaml
---
title: Docusaurus page title
slug: /reports/quarter_1
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
tome docs/reports/quarter_1.mdx \
  --template templates/registry.typ \
  --fm-section tome \
  --output build/quarter_1.pdf \
  --force
```


### Render in GitHub Actions

The workflow below first runs a setup to download the Linux x86-64 binary
from the latest GitHub release, uses the registry entry point, and publishes
the resulting PDFs as a workflow artifact.
It renders every `.mdx` in `docs/reports/` on relevant pushes, and gives
each PDF a name that matches the input file.

```yaml
name: Render report PDF

on:
  push:
    paths:
      - "docs/reports/*.mdx"
      - "templates/**"
      - ".github/workflows/render-pdf.yml"
  workflow_dispatch:

permissions:
  contents: read

jobs:
  render:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7

      - name: Install TomeMD
        run: |
          mkdir -p "$HOME/.local/bin"
          curl --fail --location \
            --output "$HOME/.local/bin/tome" \
            https://github.com/metasauce/tome-md/releases/latest/download/tome-x86_64-unknown-linux-gnu
          chmod +x "$HOME/.local/bin/tome"
          echo "$HOME/.local/bin" >> "$GITHUB_PATH"

      - name: Render PDFs
        run: |
          mkdir -p build
          for report in docs/reports/*.mdx; do
            [[ -f "$report" ]] || continue
            filename="${report##*/}"
            output="build/${filename%.mdx}.pdf"
            tome "$report" \
              --template templates/registry.typ \
              --fm-section tome \
              --output "$output" \
              --force
              done

      - uses: actions/upload-artifact@v4
        with:
          name: rendered-reports
          path: build/*.pdf
```


### Notes

The release URL above selects the Linux x86-64 binary; use the matching asset
for another runner architecture or operating system. For reproducible builds,
replace `latest` with a specific release tag. The first render may also need
network access if the templates import uncached Typst Universe packages.

TomeMD renders Markdown rather than executing Docusaurus. If a page uses MDX
components, JSX, or Docusaurus-only syntax, preprocess or simplify that content
before passing it to `tome`; ordinary YAML front matter and Markdown content can
be passed directly as shown above. We are working on improving TomeMDs handling
of JSX and html components.

The registry imports all its layouts up front. This is a simple, maintainable
choice for a modest set of templates, but imports are not lazy, so a project
with many large layouts or packages may use more memory than rendering a single
standalone template.
