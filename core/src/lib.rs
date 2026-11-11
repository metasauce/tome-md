//! Markdown -> Typst -> PDF as a library.
//!
//! A template is a self-contained folder with a `template.typ` entry point, or
//! an embedded template built from source (see [`templates`]).

pub mod error;
mod fonts;
pub mod markdown;
pub mod templates;
pub mod world;

pub use error::Error;
pub use templates::{Template, TemplateEntry, TemplateOptions};

use std::time::Instant;

use tracing::debug;
use typst::{World, WorldExt};

use crate::world::PdfWorld;

/// A rendered document: the intermediate Typst source and the PDF bytes.
#[derive(Debug)]
pub struct Rendered {
    pub typst: String,
    pub pdf: Vec<u8>,
}

/// Render Markdown with a loaded template.
///
/// Use [`render_pdf`] when the generated Typst source is not needed; that path
/// skips copying it out of the Typst world.
pub fn render(markdown: &str, template: &Template) -> Result<Rendered, Error> {
    let (world, pdf) = render_world(markdown, template)?;
    Ok(Rendered {
        typst: world.source_text().to_owned(),
        pdf,
    })
}

/// Render Markdown to PDF bytes with a loaded template, without keeping the
/// generated Typst source.
pub fn render_pdf(markdown: &str, template: &Template) -> Result<Vec<u8>, Error> {
    let (_world, pdf) = render_world(markdown, template)?;
    Ok(pdf)
}

/// The shared pipeline. Returns the Typst world so callers can decide whether
/// to copy the generated source out before dropping it.
fn render_world(markdown: &str, template: &Template) -> Result<(PdfWorld, Vec<u8>), Error> {
    let doc = markdown::parse(markdown)?;
    debug!(
        markdown_bytes = markdown.len(),
        title = doc
            .meta
            .get("title")
            .and_then(markdown::MetaValue::as_scalar)
            .unwrap_or("-"),
        "parsed Markdown"
    );

    let typst = template.build(&doc)?;
    debug!(typst_bytes = typst.len(), "generated Typst");

    let started = Instant::now();
    let world = PdfWorld::new(
        typst,
        template.asset_dir().map(|path| path.to_path_buf()),
        template.options().use_system_fonts,
    );
    debug!(
        elapsed_ms = started.elapsed().as_millis(),
        "created Typst world"
    );

    let started = Instant::now();
    let compiled = typst::compile::<typst_layout::PagedDocument>(&world);
    let document = compiled.output.map_err(|errors| Error::Compile {
        messages: format_errors(&world, &errors),
    })?;
    debug!(elapsed_ms = started.elapsed().as_millis(), "compiled Typst");

    let started = Instant::now();
    let pdf = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default()).map_err(|errors| {
        Error::Export {
            messages: format_errors(&world, &errors),
        }
    })?;
    debug!(elapsed_ms = started.elapsed().as_millis(), "exported PDF");

    Ok((world, pdf))
}

fn format_errors(world: &PdfWorld, errors: &[typst::diag::SourceDiagnostic]) -> String {
    let mut out = String::new();
    for error in errors {
        if let Some(id) = error.span.id() {
            out.push_str(id.vpath().get_without_slash());
            if let Some(range) = world.range(error.span)
                && let Ok(source) = world.source(id)
                && let Some((line, column)) = source.lines().byte_to_line_column(range.start)
            {
                out.push_str(&format!(":{}:{}", line + 1, column + 1));
            }
            out.push_str(": ");
        }
        out.push_str(&error.message);
        for hint in &error.hints {
            out.push_str("\n  hint: ");
            out.push_str(hint.v.as_str());
        }
        out.push('\n');
    }
    out.pop();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_errors_include_a_source_location() {
        let template = Template::from_source(
            r#"#let template(title: none, body) = {
  for element in body { element.content }
}
#panic("boom")"#,
            "test.typ",
            TemplateOptions::default(),
        )
        .expect("build template");

        let error = render("Hello", &template).expect_err("expected a compile error");
        let message = error.to_string();
        assert!(message.contains("main.typ:"), "{message}");
    }

    #[test]
    fn render_pdf_matches_render() {
        let template = Template::from_source(
            "#let template(title: none, body) = { for element in body { element.content } }",
            "test.typ",
            TemplateOptions::default(),
        )
        .expect("build template");
        let markdown = "# Title\n\nHello, world.";

        let rendered = render(markdown, &template).expect("render");
        let pdf = render_pdf(markdown, &template).expect("render_pdf");

        assert!(!rendered.typst.is_empty());
        assert_eq!(rendered.pdf, pdf);
    }
}
