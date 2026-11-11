use std::path::{Path, PathBuf};

use tracing::debug;

use crate::error::Error;
use crate::markdown::{Document, escape_typst};

use super::entry::{ParamKind, TemplateEntry, TemplateParam};

/// Options used when rendering a template.
///
/// Self-contained by default: only the template's own fonts are used.
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

        Ok(format!(
            "{}\n#show: body => template(\n{})\n{}",
            self.entry.source(),
            args,
            doc.body,
        ))
    }
}

fn template_argument(param: &TemplateParam, doc: &Document) -> Option<String> {
    match param.kind {
        ParamKind::Positional if param.name == "body" => Some("body".to_owned()),
        ParamKind::Positional => {
            let value = doc.meta.get(&param.name)?;
            Some(format!("[{}]", escape_typst(value)))
        }
        ParamKind::Named => doc
            .meta
            .get(&param.name)
            .map(|value| format!("{}: [{}]", param.name, escape_typst(value))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown;

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
            "#let template(title: none, body) = { body }",
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
            .validate(&markdown::parse("# Hi"))
            .expect_err("missing title");
        assert!(error.to_string().contains("title"));
    }

    #[test]
    fn optional_parameters_are_omitted() {
        let source = "#let template(title, subtitle: none, body) = { body }";
        let template = Template::from_source(source, "test.typ", TemplateOptions::default())
            .expect("build template");
        let doc = markdown::parse("---\ntitle: T\n---\nHi");
        let typst = template.build(&doc).expect("build document");
        assert!(typst.contains("[T]"));
        assert!(!typst.contains("subtitle: ["));
        assert!(typst.contains("body,"));
    }
}
