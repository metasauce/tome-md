# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.0 - 2026-11-27

### Added

- A fenced block tagged `{=typst}` passes its body to Typst unchanged, for
  vertical space, page breaks and layout blocks that Markdown has no syntax
  for. Any other language hint still renders as a code block.
- `--fm-section <key>` and `RenderOptions::with_fm_section` read template
  values from one named top-level front-matter mapping. A site generator's
  `title` and `slug` can then sit next to the keys the template uses, and a
  registry template can route to a layout from a front-matter value.
- A guide for selecting layouts from front matter, with Docusaurus and a GitHub
  Actions workflow, in `docs/user/integrations.md`.

### Fixed

- Quotes, alerts and code blocks sit as close to a heading as a paragraph does.
  Their 1.2em default spacing used to win over the heading's.
- Newlines in YAML front-matter scalar values are preserved as Typst line
  breaks instead of collapsing into spaces.

### Changed

- The build compiles OpenSSL from source (`vendor-openssl`) instead of linking
  the system one, so neither CI nor a local build needs `pkg-config` or
  `libssl-dev`. The first build takes about a minute longer; caches absorb it.
- `render` and `render_pdf` take a `&RenderOptions`, and `markdown::parse` takes
  an optional front-matter section. Library callers that do not need a section
  pass `&RenderOptions::default()`.

## 0.2.1 - 2026-11-16

### Fixed

- Lists no longer open with a wider gap. A bullet followed by its sub-bullets
  used to sit further from them than plain items sit from each other.

## 0.2.0 - 2026-11-16

### Added

- GitHub alerts (`> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`,
  `[!CAUTION]`) render as callout blocks.
- Superscript and subscript through inline `<sup>` and `<sub>` tags.
- Front matter reaches the template as arguments; a required value that is
  missing stops the render with an error.
- Images keep the alt text from the Markdown.
- Examples for a letter, a CV, meeting notes, a recipe, a flyer, and a feature
  tour that covers alerts, Mermaid diagrams and images.
- `--keep-typ` keeps the intermediate Typst file next to the PDF.
- A `small` build profile for a smaller binary, at the cost of a slower link.

### Changed

- Templates get the document as a list of elements instead of one chunk of
  text, so a template can style headings, tables and their rows separately.
- `templates/` became `examples/`.
- Templates see Typst's built-in fonts in addition to their own.

### Fixed

- Nested lists keep their indentation.

## 0.1.0 - 2026-11-11

First release. `tome` turns Markdown into a PDF using a Typst template, and
`tome-md-core` offers the same pipeline as a library.
