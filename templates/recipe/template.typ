// Recipe template.
//
// Front matter: title, serves, prep and optional cook. The body starts with an
// image declared in Markdown; the template places it in the top-right and lays
// the title and timing box out beside it.

#let template(
  title,
  serves,
  prep,
  cook: none,
  body,
) = {
  set page(paper: "a4", margin: 22mm)
  set text(font: "Libertinus Serif", size: 11pt)
  set par(leading: 0.6em, spacing: 0.9em)
  set list(spacing: 0.45em)
  set enum(spacing: 0.45em)

  // Images come from the Markdown body and fill a square top-right slot.
  set image(width: 45mm, height: 45mm, fit: "cover")
  show image: it => place(top + right, dx: 0mm, dy: 0mm, it)

  // Reserve the right column for the image.
  block(
    width: 100%,
    height: 50mm,
    pad(right: 55mm)[
      #text(size: 24pt, weight: "bold")[#title]
      #v(0.3cm)
      #block(
        fill: luma(246),
        inset: 8pt,
        table(
          columns: (auto, 1fr),
          stroke: none,
          column-gutter: 0.8em,
          inset: (x: 0pt, y: 2pt),
          [*Servings:*], [#serves],
          [*Prep:*], [#prep],
          ..if cook != none { ([*Cook:*], [#cook]) } else { () },
        ),
      )
    ],
  )

  // Section headings and callout quotes.
  v(0.3cm)
  show heading.where(level: 2): set block(above: 1.4em, below: 0.45em)
  show heading.where(level: 2): set text(size: 14pt, weight: "bold")
  show quote.where(block: true): it => block(
    fill: luma(246),
    stroke: (left: 2pt + luma(150)),
    inset: (x: 10pt, y: 6pt),
  )[#it.body]

  // Ingredients and steps from example.md.
  for element in body {
    element.content
  }
}
