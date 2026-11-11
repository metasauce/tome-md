// General meeting notes.
//
// Front matter: title, date, attendees. The body is ordinary Markdown.

#let template(title, date, attendees, body) = {
  set page(paper: "a4", margin: 22mm)
  set text(font: "Libertinus Serif", size: 10.5pt)
  set par(leading: 0.65em, spacing: 1em)

  text(size: 20pt, weight: "bold")[#title]
  v(0.1em)
  text(size: 9.5pt, fill: luma(90))[#date · #attendees]
  v(0.5cm)

  // A few light block styles keep notes easy to scan.
  show quote.where(block: true): it => block(
    fill: luma(246),
    stroke: (left: 2pt + luma(150)),
    inset: (x: 10pt, y: 6pt),
  )[#it.body]
  show raw: set text(font: "DejaVu Sans Mono", size: 9pt)
  show raw.where(block: true): it => block(
    fill: luma(246),
    inset: 8pt,
  )[#it]
  show table: set table(stroke: (x: none, y: 0.3pt), inset: 4pt)
  show table.header: set text(weight: "bold")

  for element in body {
    element.content
  }
}
