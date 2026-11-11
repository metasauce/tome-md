//! A template is a Typst entry point plus optional fonts and assets.
//!
//! [`TemplateEntry`] owns the path, source and parsed signature.
//! [`Template`] adds options and the directory used for assets.
//! Assets are resolved on demand; unused files are never read.

mod entry;
mod template;

pub use entry::{ParamKind, TemplateEntry, TemplateParam};
pub use template::{Template, TemplateOptions};
