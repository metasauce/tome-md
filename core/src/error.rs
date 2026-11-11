//! Typed errors for template loading, rendering and output.

use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Anything that can go wrong while turning Markdown into a PDF.
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("invalid template {path}: {message}")]
    InvalidTemplate { path: PathBuf, message: String },

    #[error("invalid front matter: {message}")]
    FrontMatter { message: String },

    #[error("template {path} is missing front matter key(s): {keys}")]
    MissingFrontMatter { path: PathBuf, keys: String },

    #[error("Typst compilation failed:\n{messages}")]
    Compile { messages: String },

    #[error("PDF export failed:\n{messages}")]
    Export { messages: String },
}
