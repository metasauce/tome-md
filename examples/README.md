# Examples

Ready-to-use `tome` examples. Each folder is self-contained and contains:

- `example.md` — the Markdown input
- `template.typ` — the Typst template
- `example.pdf` — the rendered result
- `README.md` — front-matter fields and feature notes

| Example                   | What it shows            |
| ------------------------- | ------------------------ |
| [letter-din](letter-din/) | DIN 5008 business letter |
| [cv](cv/)                 | Dune-themed CV           |
| [notes](notes/)           | General meeting notes    |
| [recipe](recipe/)         | Recipe card              |
| [flyer](flyer/)           | Capitol gala invitation  |

## Run one example

```sh
tome examples/recipe/example.md \
  -t examples/recipe/template.typ \
  -o out/recipe.pdf
```

## Rebuild all examples

```sh
./examples/build-examples.sh
```

Assumes `tome` is available and installed.
