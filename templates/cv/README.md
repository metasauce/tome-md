# CV / Résumé

A modern single-column CV with a sand-toned header, spice-colored accents,
section bars and a striped education table.

```sh
tome templates/cv/example.md \
  -t templates/cv/template.typ \
  -o out/cv.pdf
```

## Front matter

- `name`, `role`
- `email`, `phone`, `location`, `website`

## Markdown

Headings, bullet lists, tables, bold/italic text and links.

The education table is rendered by the template from the element data and
expects three columns: `Period | Training | Location`. With the `1fr` middle
column it always spans the full text width.
