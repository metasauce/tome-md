use std::path::{Path, PathBuf};

use tracing::debug;

use crate::error::Error;
use crate::markdown::{Document, Element, MarkdownTable};

use super::entry::{ParamKind, TemplateEntry, TemplateParam};

/// Self-contained Typst helper injected only for documents with GitHub alerts.
/// Templates receive the default GitHub-style block and can override the
/// helper if they need a different look.
const GITHUB_ALERT_HELPER: &str = r##"#let _tome_md_github_alert(kind, body) = {
  let styles = (
    note: (label: "Note", color: rgb("#0969da"), fill: rgb("#ddf4ff")),
    tip: (label: "Tip", color: rgb("#1a7f37"), fill: rgb("#dafbe1")),
    important: (
      label: "Important",
      color: rgb("#8250df"),
      fill: rgb("#fbefff"),
    ),
    warning: (label: "Warning", color: rgb("#9a6700"), fill: rgb("#fff8c5")),
    caution: (label: "Caution", color: rgb("#cf222e"), fill: rgb("#ffebe9")),
  )
  let style = styles.at(kind)
  block(
    width: 100%,
    breakable: false,
    fill: style.fill,
    stroke: (left: 4pt + style.color),
    inset: (x: 12pt, y: 8pt),
    radius: 2pt,
    above: 0.7em,
    below: 0.7em,
  )[
    #text(weight: "bold", fill: style.color)[#style.label]
    #v(0.35em)
    #body
  ]
}

"##;

/// Options used when rendering a template.
///
/// Self-contained by default: template fonts plus Typst's embedded base fonts.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct TemplateOptions {
    /// Also register common system fallback fonts.
    pub use_system_fonts: bool,
}

impl TemplateOptions {
    /// Enable or disable system fallback fonts.
    pub fn with_system_fonts(mut self, use_system_fonts: bool) -> Self {
        self.use_system_fonts = use_system_fonts;
        self
    }
}

/// A loaded template: entry point, options and asset directory.
#[derive(Clone)]
pub struct Template {
    entry: TemplateEntry,
    asset_dir: Option<PathBuf>,
    options: TemplateOptions,
}

impl Template {
    /// Loads a template from its `template.typ` entry point.
    pub fn from_path(path: PathBuf, options: TemplateOptions) -> Result<Self, Error> {
        let entry = TemplateEntry::from_path(path)?;
        let asset_dir = Some(entry.dir().to_path_buf());
        Ok(Self {
            entry,
            options,
            asset_dir,
        })
    }

    /// Builds a template from embedded source.
    pub fn from_source(
        source: impl Into<String>,
        label: impl Into<PathBuf>,
        options: TemplateOptions,
    ) -> Result<Self, Error> {
        Ok(Self {
            entry: TemplateEntry::from_source(source, label)?,
            options,
            asset_dir: None,
        })
    }

    /// The parsed entry point.
    pub fn entry(&self) -> &TemplateEntry {
        &self.entry
    }

    /// Rendering options.
    pub fn options(&self) -> &TemplateOptions {
        &self.options
    }

    /// Directory served to Typst for assets, if this is a folder template.
    pub fn asset_dir(&self) -> Option<&Path> {
        self.asset_dir.as_deref()
    }

    /// Checks that required front-matter keys exist.
    pub fn validate(&self, doc: &Document) -> Result<(), Error> {
        // The parser guarantees that `body` is present and positional.
        let missing = self
            .entry
            .params()
            .iter()
            .filter(|param| {
                param.kind == ParamKind::Positional
                    && param.name != "body"
                    && !doc.meta.contains_key(&param.name)
            })
            .map(|param| param.name.as_str())
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(Error::MissingFrontMatter {
                path: self.entry.path().to_path_buf(),
                keys: missing.join(", "),
            });
        }

        // unused parameters
        for key in doc.meta.keys() {
            if key != "body" && !self.entry.params().iter().any(|param| &param.name == key) {
                debug!(key = %key, "front matter key is not used by the template");
            }
        }

        Ok(())
    }

    /// Wraps the Markdown body in the template using a Typst show rule.
    pub fn build(&self, doc: &Document) -> Result<String, Error> {
        self.validate(doc)?;

        let mut args = self
            .entry
            .params()
            .iter()
            .filter_map(|param| template_argument(param, doc))
            .collect::<Vec<_>>()
            .join(",\n  ");

        if !args.is_empty() {
            args = format!("  {args},\n");
        }

        let helper = if doc.has_alerts() {
            GITHUB_ALERT_HELPER
        } else {
            ""
        };

        Ok(format!(
            "{helper}{}\n#template(\n{})\n",
            self.entry.source(),
            args
        ))
    }
}

fn template_argument(param: &TemplateParam, doc: &Document) -> Option<String> {
    match param.kind {
        ParamKind::Positional if param.name == "body" => Some(elements_literal(&doc.elements)),
        ParamKind::Positional => {
            let value = doc.meta.get(&param.name)?;
            Some(value.to_typst())
        }
        ParamKind::Named => doc
            .meta
            .get(&param.name)
            .map(|value| format!("{}: {}", param.name, value.to_typst())),
    }
}

/// Encode the element list as Typst dictionaries with content and optional data.
fn elements_literal(elements: &[Element]) -> String {
    let mut out = String::from("(");
    for element in elements {
        match element {
            Element::Content(content) => {
                out.push_str("(content: [");
                out.push_str(content);
                out.push_str("], data: none),");
            }
            Element::Table { content, data } => {
                out.push_str("(content: [");
                out.push_str(content);
                out.push_str("], data: ");
                out.push_str(&table_literal(data));
                out.push_str("),");
            }
        }
    }
    out.push(')');
    out
}

/// Encode one table as a Typst array of rows and string cells.
fn table_literal(table: &MarkdownTable) -> String {
    let mut out = String::from("(");
    for row in &table.rows {
        out.push('(');
        for cell in row {
            out.push_str(&typst_string(cell));
            out.push(',');
        }
        out.push_str("),");
    }
    out.push(')');
    out
}

fn typst_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' | '\r' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown;

    fn parse(input: &str) -> markdown::Document {
        markdown::parse(input).expect("parse markdown")
    }

    #[test]
    fn loads_self_contained_template() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/template/template.typ");
        let template =
            Template::from_path(path, TemplateOptions::default()).expect("load fixture template");
        assert!(template.asset_dir().is_some());
        assert!(
            template
                .entry()
                .params()
                .iter()
                .any(|param| param.name == "body" && param.kind == ParamKind::Positional)
        );
    }

    #[test]
    fn options_default_to_self_contained() {
        assert!(!TemplateOptions::default().use_system_fonts);
    }

    #[test]
    fn builds_from_embedded_source() {
        let template = Template::from_source(
            "#let template(title: none, body) = { for element in body { element.content } }",
            "embedded.typ",
            TemplateOptions::default(),
        )
        .expect("build embedded template");
        assert!(template.asset_dir().is_none());
        assert!(
            template
                .entry()
                .params()
                .iter()
                .any(|param| param.name == "body" && param.kind == ParamKind::Positional)
        );
    }

    #[test]
    fn missing_entry_is_an_error() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../templates/missing/template.typ");
        let error = Template::from_path(path, TemplateOptions::default())
            .err()
            .expect("missing entry");
        assert!(error.to_string().contains("cannot read"));
    }

    #[test]
    fn validation_reports_missing_front_matter() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/template/template.typ");
        let template =
            Template::from_path(path, TemplateOptions::default()).expect("load fixture template");
        let error = template
            .validate(&parse("# Hi"))
            .expect_err("missing title");
        assert!(error.to_string().contains("title"));
    }

    #[test]
    fn optional_parameters_are_omitted() {
        let source = "#let template(title, subtitle: none, body) = { body }";
        let template = Template::from_source(source, "test.typ", TemplateOptions::default())
            .expect("build template");
        let doc = parse("---\ntitle: T\n---\nHi");
        let typst = template.build(&doc).expect("build document");
        assert!(typst.contains("[T]"));
        assert!(!typst.contains("subtitle: ["));
        assert!(typst.contains("(content:"), "{typst}");
    }

    #[test]
    fn alert_helper_is_injected_only_for_alerts() {
        let source = "#let template(body) = { for element in body { element.content } }";
        let template = Template::from_source(source, "test.typ", TemplateOptions::default())
            .expect("build template");

        let alert_doc = parse("> [!NOTE]\n> Body");
        let typst = template.build(&alert_doc).expect("build alert document");
        assert!(typst.contains("#let _tome_md_github_alert"), "{typst}");
        assert!(
            typst.contains("#_tome_md_github_alert(\"note\")["),
            "{typst}"
        );

        let plain_doc = parse("Just text");
        let typst = template.build(&plain_doc).expect("build plain document");
        assert!(!typst.contains("_tome_md_github_alert"), "{typst}");
    }

    #[test]
    fn nested_frontmatter_parameter_is_a_dictionary() {
        let source = "#let template(sender, body) = { for element in body { element.content } }";
        let template = Template::from_source(source, "test.typ", TemplateOptions::default())
            .expect("build template");
        let doc = parse("---\nsender:\n  name: Jane Doe\n  address: 1 Example Street\n---\nHi");
        let typst = template.build(&doc).expect("build document");
        assert!(
            typst.contains("(address: [1 Example Street], name: [Jane Doe])"),
            "{typst}"
        );
    }

    #[test]
    fn tables_are_inside_the_element_list() {
        let source = "#let template(body) = { body }";
        let template = Template::from_source(source, "test.typ", TemplateOptions::default())
            .expect("build template");
        let doc = parse("| Std. | Tätigkeit |\n| ---: | --- |\n| 4,0 | Setup |");
        let typst = template.build(&doc).expect("build document");
        assert!(
            typst.contains(r#"data: (("Std.","Tätigkeit",),("4,0","Setup",),)"#),
            "{typst}"
        );
    }

    #[test]
    fn body_is_not_required_from_front_matter() {
        let source = "#let template(body) = { body }";
        let template = Template::from_source(source, "test.typ", TemplateOptions::default())
            .expect("build template");
        let doc = parse("| A |\n| --- |\n| 1 |");
        let typst = template
            .build(&doc)
            .expect("body is supplied by the parser");
        assert!(typst.contains("(content:"), "{typst}");
    }

    #[test]
    fn elements_without_tables_have_no_data() {
        let source = "#let template(body) = { body }";
        let template = Template::from_source(source, "test.typ", TemplateOptions::default())
            .expect("build template");
        let doc = parse("No table here");
        let typst = template.build(&doc).expect("build document");
        assert!(typst.contains("data: none"), "{typst}");
    }
}
