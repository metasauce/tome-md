// Feature-tour template.
//
// Front matter: title and description. The body is Markdown. Mermaid fenced
// blocks are rendered through the Merman package.

#import "@preview/merman:0.5.0": mermaid

#let template(title: none, description: none, body) = {
  set page(paper: "a4", margin: 20mm, numbering: "1")
  set text(font: "Libertinus Serif", size: 10.5pt)
  set par(leading: 0.65em, spacing: 1em)
  // Boxes should sit as close to headings as paragraphs do. Spacing collapses
  // to the larger of the two, so keep this below `par.spacing`.
  set block(above: 0.7em, below: 0.7em)

  show heading: set text(weight: "bold")
  show heading.where(level: 1): set text(size: 20pt)
  show heading.where(level: 2): set text(size: 15pt)
  show heading.where(level: 3): set text(size: 12.5pt)
  show heading.where(level: 4): set text(size: 11pt)

  set list(indent: 1em, spacing: 0.35em)
  set enum(indent: 1em, spacing: 0.35em)
  set table(inset: (x: 0.5em, y: 0.35em), stroke: 0.5pt + luma(200))
  show table.header: set text(weight: "bold")

  show link: set text(fill: rgb("#1565c0"))
  show link: underline
  show quote.where(block: true): it => block(
    fill: luma(246),
    stroke: (left: 2pt + luma(150)),
    inset: (x: 10pt, y: 6pt),
    above: 0.7em,
    below: 0.7em,
  )[#it.body]

  show raw: set text(font: "DejaVu Sans Mono", size: 9pt)
  show image: set image(width: 100%)

  // Keep this after the generic raw styling so Mermaid blocks reach Merman.
  show raw.where(lang: "mermaid"): it => mermaid(it.text, width: 100%)

  if title != none {
    text(size: 22pt, weight: "bold")[#title]
    v(0.25em)
  }
  if description != none {
    text(size: 9.5pt, fill: luma(95))[#description]
    v(1em)
  }

  for element in body {
    element.content
  }
}
