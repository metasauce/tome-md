// Bundled plain template. Uses system Arial (with Liberation Sans / DejaVu Sans
// fallback) so no font files are bundled. Pass --template for a self-contained
// template with its own fonts.

#let sans = ("Arial", "Liberation Sans", "DejaVu Sans")
#let mono = ("DejaVu Sans Mono", "Liberation Mono", "Courier New", "Arial", "Liberation Sans", "DejaVu Sans")

#let template(title: none, subtitle: none, body) = {
  set page(paper: "a4", margin: 20mm, numbering: "1")
  set text(font: sans, size: 11pt)
  set par(leading: 0.65em, spacing: 1.1em)
  // Keep boxes as close to headings as paragraphs are. Spacing collapses to the
  // larger of the two, so anything below `par.spacing` follows the surroundings.
  set block(above: 0.7em, below: 0.7em)

  show heading: set text(weight: 700)
  show heading.where(level: 1): set text(size: 22pt)
  show heading.where(level: 2): set text(size: 16pt)
  show heading.where(level: 3): set text(size: 13pt)
  show heading.where(level: 4): set text(size: 11.5pt)

  set list(indent: 1.1em, spacing: 0.35em)
  set enum(indent: 1.1em, spacing: 0.35em)

  set table(inset: (x: 0.6em, y: 0.4em), stroke: 0.5pt + luma(200))
  set table.hline(stroke: 0.5pt + luma(200))
  show table.cell.where(y: 0): set text(weight: 700)

  show raw.where(block: true): it => block(
    width: 100%,
    fill: luma(246),
    stroke: 0.5pt + luma(205),
    radius: 2pt,
    inset: 8pt,
    above: 0.7em,
    below: 0.7em,
  )[#text(font: mono, size: 9pt)[#it]]
  show raw.where(block: false): it => box(
    fill: luma(246),
    inset: (x: 0.25em, y: 0.05em),
    radius: 1pt,
  )[#text(font: mono, size: 9.5pt)[#it]]

  show quote.where(block: true): it => block(
    fill: luma(246),
    stroke: (left: 2pt + luma(150)),
    inset: (x: 12pt, y: 8pt),
    above: 0.7em,
    below: 0.7em,
  )[#it.body]

  show link: set text(fill: rgb("#1565c0"))
  show link: underline

  if title != none [
    #v(1em)
    #align(center)[
      #text(size: 24pt, weight: 700)[#title]
      #if subtitle != none [
        #v(0.4em)
        #text(size: 10pt, fill: luma(110))[#subtitle]
      ]
    ]
    #v(1.4em)
  ]

  for element in body {
    element.content
  }
}
