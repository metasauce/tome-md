// Event flyer template (A5).
//
// Front matter: event, subtitle, date, time, venue, dress and closing. The
// body holds the programme and notes. The assets/ folder is served to Typst,
// so `image("assets/...")` reads the crest and divider next to this template.

#let gold = rgb("#c9a227")
#let ivory = rgb("#f4ead5")
#let ink = rgb("#12100e")

#let template(
  event,
  subtitle,
  date,
  time,
  venue,
  dress,
  closing,
  body,
) = {
  set page(paper: "a5", margin: 0mm, fill: ink)
  set text(font: "Libertinus Serif", size: 9.5pt, fill: ivory)
  set par(leading: 0.75em, spacing: 1em)
  set list(marker: text(fill: gold)[◆], indent: 0.6em, spacing: 0.5em)

  // Double art-deco frame, inset from the page edge.
  place(center + horizon, rect(width: 100% - 10mm, height: 100% - 10mm, stroke: 0.7pt + gold))
  place(center + horizon, rect(width: 100% - 12mm, height: 100% - 12mm, stroke: 0.3pt + gold))

  // Body headings and quotes in the Capitol palette.
  show heading.where(level: 2): it => {
    v(1em)
    text(size: 9.5pt, weight: "bold", tracking: 0.12em, fill: gold)[#it.body]
    v(0.25em)
  }
  block(width: 100%, height: 100%, {
    pad(x: 15mm, y: 13mm)[
      #align(center)[
        #text(size: 7pt, tracking: 0.3em, fill: gold)[PANEM · CAPITOL]
        #v(0.9em)
        #image("assets/crest.svg", width: 18mm)
        #v(0.5em)
        #text(size: 26pt, weight: "bold")[#event]
        #v(0.1em)
        #text(size: 11pt, style: "italic", fill: gold)[#subtitle]
        #v(0.55em)
        #image("assets/ornament.svg", width: 46mm)
      ]

      #v(1em)
      #grid(
        columns: (auto, 1fr),
        column-gutter: 6mm,
        row-gutter: 0.5em,
        align: (left + horizon, left + horizon),
        text(size: 7.5pt, tracking: 0.12em, fill: gold)[DATE], text[#date · #time],
        text(size: 7.5pt, tracking: 0.12em, fill: gold)[VENUE], text[#venue],
        text(size: 7.5pt, tracking: 0.12em, fill: gold)[DRESS], text[#dress],
        text(size: 7.5pt, tracking: 0.12em, fill: gold)[ACCESS], text[By invitation only],
      )

      #v(1.25em)
      #block(width: 100%, height: 0.4pt, fill: gold)

      #for element in body {
        element.content
      }
    ]

    // Closing line, anchored to the bottom of the page.
    place(bottom + center, dy: -20mm, align(center)[
      #block(width: 35%, height: 0.3pt, fill: gold)
      #v(12pt)
      #text(size: 10pt, style: "italic")[#closing]
      #v(12pt)
      #block(width: 35%, height: 0.3pt, fill: gold)
    ])
  })
}
