# Feature tour

A compact tour of the Markdown features supported by `tome`, with a template
that enables Mermaid diagrams.

```sh
tome examples/feature_tour/example.md \
  -t examples/feature_tour/template.typ \
  -o out/feature-tour.pdf
```

## Front matter

- `title`
- `description`

## Supported

- headings at levels 1–5
- paragraphs, bold, italic, strikethrough
- superscript/subscript via inline `<sup>` / `<sub>` tags
- inline code and fenced code blocks
- links, horizontal rules and hard line breaks
- ordered, unordered, nested and task lists
- normal blockquotes and GitHub alerts
- tables with column alignment
- local images
- Mermaid diagrams through the `merman` package

The first Mermaid render downloads `@preview/merman:0.5.0` into Typst's package
cache.

## Not yet

- remote images
- Docusaurus `:::` admonitions
- MDX/JSX
- platform-specific custom blocks
- footnotes and math
