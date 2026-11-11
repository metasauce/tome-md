use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use typst::syntax::ast::AstNode;
use typst::syntax::{SyntaxNode, ast};

use crate::error::Error;

/// How a template parameter is supplied by the generated show rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// No Typst default: required and passed positionally.
    Positional,
    /// Has a Typst default: optional and passed by name when present.
    Named,
}

/// One parameter of `#let template(...)`, in signature order.
#[derive(Debug, Clone)]
pub struct TemplateParam {
    pub name: String,
    pub kind: ParamKind,
}

/// A template entry point: path, source and parsed signature.
#[derive(Clone)]
pub struct TemplateEntry {
    path: PathBuf,
    source: String,
    params: Vec<TemplateParam>,
}

impl TemplateEntry {
    /// Loads an entry point from its `template.typ` path.
    pub fn from_path(path: PathBuf) -> Result<Self, Error> {
        let source = fs::read_to_string(&path).map_err(|source| Error::Read {
            path: path.clone(),
            source,
        })?;
        Self::new(path, source)
    }

    /// Builds an entry point from embedded source and a display label.
    pub fn from_source(
        source: impl Into<String>,
        label: impl Into<PathBuf>,
    ) -> Result<Self, Error> {
        Self::new(label.into(), source.into())
    }

    fn new(path: PathBuf, source: String) -> Result<Self, Error> {
        let params = parse_signature(&source).map_err(|message| Error::InvalidTemplate {
            path: path.clone(),
            message,
        })?;
        Ok(Self {
            path,
            source,
            params,
        })
    }

    /// Path or display label of the entry point.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Typst source of the entry point.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Parsed parameters of `#let template(...)`.
    pub fn params(&self) -> &[TemplateParam] {
        &self.params
    }

    /// Directory containing the entry point.
    pub(super) fn dir(&self) -> &Path {
        self.path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
    }
}

/// Parses `#let template(...)`.
///
/// Positional parameters are required front-matter keys; named parameters are
/// optional and retain their Typst defaults. `body` must be positional.
fn parse_signature(source: &str) -> Result<Vec<TemplateParam>, String> {
    let root = typst::syntax::parse(source);
    let closure = find_template_closure(&root)
        .ok_or_else(|| "missing `#let template(...)` definition".to_owned())?;
    let param_node = closure.params();
    let (errors, _) = param_node.to_untyped().errors_and_warnings();
    if let Some(error) = errors.first() {
        return Err(error.message.to_string());
    }
    let params = param_node
        .children()
        .map(template_param)
        .collect::<Result<Vec<_>, _>>()?;

    let mut seen = HashSet::new();
    for param in &params {
        if !seen.insert(param.name.as_str()) {
            return Err(format!("duplicate parameter `{}`", param.name));
        }
    }

    match params.iter().find(|param| param.name == "body") {
        Some(body) if body.kind == ParamKind::Positional => Ok(params),
        Some(_) => Err("`body` must be positional (no default)".to_owned()),
        None => Err("missing `body` parameter".to_owned()),
    }
}

fn template_param(param: ast::Param<'_>) -> Result<TemplateParam, String> {
    match param {
        ast::Param::Pos(pattern) => {
            let mut bindings = pattern.bindings();
            if bindings.len() != 1 {
                return Err("parameters must be simple identifiers".to_owned());
            }
            Ok(TemplateParam {
                name: bindings.remove(0).as_str().to_owned(),
                kind: ParamKind::Positional,
            })
        }
        ast::Param::Named(named) => Ok(TemplateParam {
            name: named.name().as_str().to_owned(),
            kind: ParamKind::Named,
        }),
        ast::Param::Spread(_) => Err("spread parameters (`..args`) are not supported".to_owned()),
    }
}

fn find_template_closure<'a>(node: &'a SyntaxNode) -> Option<ast::Closure<'a>> {
    if let Some(ast::Expr::Closure(closure)) = node.cast::<ast::Expr>()
        && closure
            .name()
            .is_some_and(|name| name.as_str() == "template")
    {
        return Some(closure);
    }

    node.children().find_map(find_template_closure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_signature() {
        let params = parse_signature("#let template(title, subtitle: none, body) = { body }")
            .expect("parse signature");
        let names = params
            .iter()
            .map(|param| (param.name.as_str(), param.kind))
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                ("title", ParamKind::Positional),
                ("subtitle", ParamKind::Named),
                ("body", ParamKind::Positional),
            ]
        );
    }

    #[test]
    fn rejects_unsupported_signatures() {
        assert!(
            parse_signature("#let template(title, ..args, body) = { body }")
                .expect_err("spread")
                .contains("spread")
        );
        assert!(
            parse_signature("#let template((title, subtitle), body) = { body }")
                .expect_err("destructuring")
                .contains("simple identifiers")
        );
        assert!(
            parse_signature("#let template(title, body: none) = { body }")
                .expect_err("named body")
                .contains("positional")
        );
        assert!(
            parse_signature("#let template(title, title, body) = { body }")
                .expect_err("duplicate parameter")
                .contains("duplicate")
        );
        assert!(
            parse_signature("#let template(title) = { }")
                .expect_err("missing body")
                .contains("body")
        );
    }
}
