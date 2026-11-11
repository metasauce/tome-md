// DIN 5008 Form B business letter.
//
// This example imports the letter-pro package from Typst Universe and uses
// its DIN 5008 layout, address/information helpers and folding marks.
//
// Layout & package: https://github.com/Sematre/typst-letter-pro
// Package:         @preview/letter-pro:3.0.0 (MIT)

#import "@preview/letter-pro:3.0.0": (
  letter-generic,
  header-simple,
  sender-box,
  annotations-box,
  recipient-box,
  address-tribox,
)

#let template(
  sender,
  recipient,
  date,
  subject,
  annotations: none,
  reference: none,
  enclosures: none,
  signature: none,
  body,
) = {
  set text(lang: "de", font: "Source Sans 3", hyphenate: false)

  let sender_phone = sender.at("phone", default: none)
  let sender_email = sender.at("email", default: none)

  letter-generic(
    header: pad(
      left: 25mm,
      right: 20mm,
      top: 20mm,
      bottom: 5mm,
      align(bottom + right, header-simple(
        sender.name,
        [#sender.street #linebreak() #sender.city],
        extra: if sender_phone != none or sender_email != none {
          [
            #if sender_phone != none [ Telefon: #sender_phone ]
            #if sender_email != none [ #linebreak() E-Mail: #sender_email ]
          ]
        },
      )),
    ),
    address-box: address-tribox(
      sender-box(name: sender.name, [#sender.street, #sender.city]),
      annotations-box(annotations),
      recipient-box([
        #recipient.name \
        #recipient.department \
        #recipient.street #linebreak()
        #recipient.city
      ]),
    ),
    reference-signs: if reference != none {
      ((reference.label, reference.value),)
    },
    {
      if date != none {
        align(right, date)
        v(0.65em)
      }
      if subject != none {
        pad(right: 10%, strong(subject))
        v(0.65em)
      }
      set par(justify: true)
      for element in body {
        element.content
      }
      if signature != none {
        // Roughly one centimetre of visible space after the closing line.
        v(0.8cm)
        signature
      }
      if enclosures != none {
        v(1fr)
        [*Anlagen:* #linebreak() - #enclosures]
      }
    },
  )
}
