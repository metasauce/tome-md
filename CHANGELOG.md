# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
