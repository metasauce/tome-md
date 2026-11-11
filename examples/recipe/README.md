# Recipe

A cocktail recipe card with a top-right image and a metadata box for portions
and timings.

```sh
tome examples/recipe/example.md \
  -t examples/recipe/template.typ \
  -o out/recipe.pdf
```

## Front matter

- `title`
- `serves`
- `prep`
- `cook` (optional; omitted for cocktails)

## Markdown

The example declares its image in Markdown (`![Pisco Sour](assets/pisco-sour.svg)`);
the template places it in the top-right. Image paths are resolved inside the
template folder. The body also demonstrates unordered and ordered lists, block
quotes and bold text.
