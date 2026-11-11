# DIN 5008 letter with `letter-pro`

A German business-letter example that imports the
[`letter-pro`](https://github.com/Sematre/typst-letter-pro) package from Typst
Universe:

```typst
#import "@preview/letter-pro:3.0.0": (
  letter-generic,
  header-simple,
  sender-box,
  annotations-box,
  recipient-box,
  address-tribox,
)
```

The package provides the DIN 5008 Form B layout (45 mm header, address and
reference boxes, folding/hole marks) and the justified body text.

The example content is the Finanzamt objection sample from `letter-pro`, with
fictional names and addresses.

```sh
tome examples/letter-din/example.md \
  -t examples/letter-din/template.typ \
  -o out/letter-din.pdf
```

`letter-pro` is MIT licensed by Sematre. See the upstream project for the
package documentation and license.

## Front matter

Nested mappings keep the sender, recipient and reference groups together:

```yaml
---
sender:
  name: Anja Ahlsen
  street: Beispielallee 28
  city: 12345 Musterstadt
  phone: +49 152 28817386
  email: aahlsen@example.com
recipient:
  name: Finanzamt Musterstadt
  department: Einkommensteuerstelle
  street: Beispielweg 5
  city: 12345 Musterstadt
reference:
  label: Steuernummer
  value: 333/24692/5775
date: 12. November 2026
subject: Einspruch gegen den ESt-Bescheid
annotations: Einschreiben – Rückschein
enclosures: Rechnung
signature: Anja Ahlsen
---
```

Required: `sender`, `recipient`, `date`, `subject` (`sender` and `recipient`
are mappings).

Optional: `sender.phone`, `sender.email`, `annotations`, `reference`,
`enclosures`, `signature`.

## Fonts

Source Sans 3, licensed under the SIL Open Font License
(`assets/fonts/OFL.txt`).
