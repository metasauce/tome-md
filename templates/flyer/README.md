# Flyer

An A5 invitation flyer for a Capitol high-society Hunger Games gala. The dark
ink background, gold frame and accents carry the art-deco mood.

```sh
tome templates/flyer/example.md \
  -t templates/flyer/template.typ \
  -o out/flyer.pdf
```

## Front matter

- `event`, `subtitle`
- `date`, `time`
- `venue`
- `dress`
- `closing`

## Markdown

Section headings, bullet lists and bold text.

## Assets

`template.typ` reads two images from the template folder:

- `assets/crest.svg` — the Capitol crest
- `assets/ornament.svg` — the divider under the title

They are served to Typst from the folder next to `template.typ`, so no extra
configuration is needed.
