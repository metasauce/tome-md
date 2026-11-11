// CV / résumé template.
//
// Front matter: name, role, email, phone, location, website. The Markdown body
// holds the sections (profile, experience, education, skills) and is rendered
// below the header.

// Desert palette: sun-baked sand, spice/terracotta and warm ink.
#let accent = rgb("#9a4b2f")
#let sand = rgb("#f3e7d3")
#let stripe = rgb("#f8f0e2")
#let muted = rgb("#6b5a48")
#let ink = rgb("#2b2520")

#let template(
  name,
  role,
  email,
  phone,
  location,
  website,
  body,
) = {
  set page(
    paper: "a4",
    margin: (x: 20mm, y: 18mm),
    fill: rgb("#fdfaf4"),
  )
  set text(font: "Libertinus Serif", size: 10.5pt, fill: ink)
  set par(leading: 0.65em, spacing: 0.95em)
  set list(indent: 1em, spacing: 0.4em)

  // Header: name and role on the left, contact details on the right, on a
  // soft sand block and separated from the body by a spice-colored rule.
  block(
    width: 100%,
    fill: sand,
    radius: 4pt,
    inset: (x: 12pt, y: 12pt),
    grid(
      columns: (1fr, auto),
      column-gutter: 16pt,
      align: (left, right),
      [
        #text(size: 28pt, weight: "bold")[#name]
        #v(0.1em)
        #text(size: 13pt, fill: accent)[#role]
      ],
      text(size: 9pt, fill: muted)[
        #email \
        #phone \
        #location \
        #website
      ],
    ),
  )
  v(0.4em)
  line(length: 100%, stroke: 2pt + accent)

  // Typst's built-in heading spacing cannot be combined reliably with custom
  // section rules, so reset it and control all spacing with explicit v().
  show heading: set block(above: 0em, below: 0em)

  // Section titles: bold text with a subtle sand-colored hairline.
  show heading.where(level: 2): it => {
    v(1.8em)
    text(size: 13pt, weight: "bold")[#it.body]
    v(0.35em)
    line(length: 100%, stroke: 0.6pt + rgb("#d9cbb5"))
    v(0.45em)
  }
  // Company/role entries inside a section. The explicit top space keeps a
  // consistent gap after the preceding list or section rule.
  show heading.where(level: 3): it => {
    v(0.9em)
    text(size: 11pt, weight: "bold")[#it.body]
    v(0.2em)
  }

  // Links use the accent color.
  show link: set text(fill: accent)

  // Profile, experience, education and skills from example.md. Table elements
  // are rendered here so the education table spans the full text width and
  // gets the striped, borderless treatment.
  for element in body {
    if element.data == none {
      element.content
    } else {
      let rows = element.data
      if rows.len() > 0 {
        let header = rows.at(0)
        let entries = rows.slice(1)
        table(
          columns: (auto, 1fr, auto),
          stroke: none,
          inset: (x: 7pt, y: 6pt),
          align: (left, left, right),
          fill: (x, y) => if y > 0 and calc.odd(y) { stripe } else { none },
          table.header(
            ..header.map(cell => text(size: 8.5pt, weight: "bold", fill: muted, tracking: 0.05em)[#cell]),
          ),
          ..entries
            .map(row => (
              text(fill: accent)[#row.at(0)],
              text(weight: "bold")[#row.at(1)],
              text(fill: muted)[#row.at(2)],
            ))
            .flatten(),
        )
      }
    }
  }
}
