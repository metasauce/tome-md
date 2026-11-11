# tome-md

`tome` turns Markdown into polished PDFs. Write your content in plain Markdown,
style it with a [Typst](https://typst.app) template, and get a clean document
no browser, office suite or LaTeX toolchain required. Everything runs in one
small Rust binary.

## Installation

Either grab the binary for your platform from the
[releases page](https://github.com/semohr/tome-md/releases) or build from source.

### Build from source

Requires a recent stable Rust toolchain.

```sh
git clone https://github.com/semohr/tome-md.git
cd tome-md
cargo install --path cli
```

This installs `tome` into `~/.cargo/bin`. To build without installing, run
`cargo build -p tome-md --release` and use `target/release/tome`.

## Usage

```sh
tome notes.md                              # bundled plain template
tome notes.md -o out/notes.pdf             # explicit output path
tome notes.md -t my-template/template.typ  # custom template
tome notes.md --keep-typ                   # keep the generated Typst source
```

## CLI

```
tome <input.md> [--template <template.typ>] [-o <out.pdf>] [--keep-typ] [--force]
```

`--template` points at a `.typ` entry point; without it the bundled plain
template is used. Existing outputs are rejected unless `--force` is given, and
paths that would overwrite the template entry point, the input Markdown (or the
PDF when `--keep-typ` is set) are always rejected. Exit codes: `0` success, `1`
render error, `2` usage error.

## Templates

A template decides what the PDF looks like. `tome` uses
[Typst](https://typst.app) for layout, so any `.typ` file can be a template as
long as it defines a `template` function. The minimal one takes the document
body as a list of elements:

```typst
#let template(body) = {
  // page setup, styling, cover page…
  for element in body {
    element.content
  }
}
```

Each element has `content` (the Typst markup) and `data` (`none`, or the rows
of a table). To pull values from the front matter at the top of your Markdown
file, add parameters to the signature:

```typst
#let template(without_default, with_default: none, body) = {
  // …
  without_default
  if with_default != none {
    with_default
  }
  for element in body {
    element.content
  }
}
```

```markdown
---
without_default: Im required
with_default: Im not required
---

# Content
```

A parameter with a default can be left out while one without a default is
required.

Other files in the folder, like fonts and images, are available to the template
as if they sat next to it. That keeps a template self-contained, so you can copy
it around or share it as a whole.

For a number of examples see the [examples](examples/) folder.

See [docs/templates.md](docs/templates.md) for the full rules and font handling.

## License

This project is licensed under the
[PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0).

In short, this license allows broad use for personal, educational, research, and
non-profit purposes, but forbids commercial use.

If you want to use this commercially, send me an email and we can discuss.

See the [LICENSE](LICENSE) file for the full terms.
