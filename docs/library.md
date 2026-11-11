# Library

`tome-md-core` exposes the same pipeline the CLI uses.

```rust
use tome_md_core::{Template, TemplateOptions, render};

let markdown = std::fs::read_to_string("notes.md")?;
let template = Template::from_path("my-template/template.typ", TemplateOptions::default())?;
let rendered = render(&markdown, &template)?;
std::fs::write("notes.pdf", rendered.pdf)?;
```

`render` returns `Rendered { typst, pdf }`: the generated Typst source and the
PDF bytes.

`TemplateOptions::default()` loads only the template's own fonts. Build a
template from an embedded string with `Template::from_source` when there is no
folder on disk; the CLI does this for its bundled `plain.typ`.
