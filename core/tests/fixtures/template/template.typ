// Minimal self-contained template used by the tests. The folder is the asset
// directory, so files next to this entry point are served to Typst on demand.
#let template(title, subtitle: none, body) = {
  heading(level: 1)[#title]
  body
}
