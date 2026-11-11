# Notes

General meeting notes with a title, date/attendees line and styled Markdown
blocks.

```sh
tome examples/notes/example.md \
  -t examples/notes/template.typ \
  -o out/notes.pdf
```

## Front matter

- `title`
- `date`
- `attendees`

## Markdown

The example demonstrates:

- headings at levels 1, 2 and 3
- paragraphs, **bold**, _italic_ and ~~strikethrough~~
- superscript (`15<sup>th</sup>`) and subscript (`CO<sub>2</sub>`)
- inline code and fenced code blocks with a language
- ordered, unordered, nested and task lists
- block quotes and horizontal rules
- links and hard line breaks
- tables with column alignment

Code uses the embedded DejaVu Sans Mono.
