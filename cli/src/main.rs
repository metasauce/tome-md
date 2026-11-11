use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::Parser;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use tome_md_core::{Error, Template, TemplateOptions, render, render_pdf};

/// Compile Markdown to PDF through Typst.
#[derive(Parser)]
#[command(name = "tome", version, about)]
struct Cli {
    /// Input Markdown file
    input: PathBuf,

    /// Typst template entry point (.typ); defaults to the bundled plain template
    #[arg(short, long, value_parser = parse_template)]
    template: Option<Template>,

    /// Output PDF (defaults to the input path with .pdf)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Also write the generated Typst source next to the PDF
    #[arg(long)]
    keep_typ: bool,

    /// Overwrite existing output files
    #[arg(long)]
    force: bool,
}

fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    run(cli)
}

fn bundled_template(options: TemplateOptions) -> Result<Template, Error> {
    Template::from_source(include_str!("plain.typ"), "<bundled plain>", options)
}

fn default_template() -> Result<Template, Error> {
    bundled_template(TemplateOptions::default().with_system_fonts(true))
}

fn parse_template(value: &str) -> Result<Template, Error> {
    Template::from_path(PathBuf::from(value), TemplateOptions::default())
}

fn same_path(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn run(cli: Cli) -> Result<()> {
    let template = match cli.template {
        Some(template) => template,
        None => default_template()?,
    };
    let output = cli
        .output
        .clone()
        .unwrap_or_else(|| cli.input.with_extension("pdf"));
    let typst_path = cli.keep_typ.then(|| output.with_extension("typ"));

    if same_path(&output, &cli.input) {
        bail!(
            "refusing to overwrite the input Markdown `{}`",
            cli.input.display()
        );
    }
    if typst_path
        .as_ref()
        .is_some_and(|path| same_path(path, &output))
    {
        bail!(
            "`--keep-typ` would overwrite the PDF `{}`",
            output.display()
        );
    }
    if !cli.force && (output.exists() || typst_path.as_ref().is_some_and(|path| path.exists())) {
        bail!(
            "output `{}` already exists; use --force to overwrite it",
            output.display()
        );
    }

    debug!(template = %template.entry().path().display(), input = %cli.input.display(), "starting render");
    let started = Instant::now();

    let markdown = fs::read_to_string(&cli.input)
        .with_context(|| format!("could not read Markdown `{}`", cli.input.display()))?;

    info!(
        template = %template.entry().path().display(),
        params = template.entry().params().len(),
        "loaded template"
    );

    let (typst, pdf) = match &typst_path {
        // The Typst source is only kept when it is written out.
        Some(_) => {
            let rendered = render(&markdown, &template)
                .with_context(|| format!("could not render `{}`", cli.input.display()))?;
            (Some(rendered.typst), rendered.pdf)
        }
        None => {
            let pdf = render_pdf(&markdown, &template)
                .with_context(|| format!("could not render `{}`", cli.input.display()))?;
            (None, pdf)
        }
    };

    let elapsed_ms = started.elapsed().as_millis();
    if let Some(typst) = &typst {
        info!(
            elapsed_ms,
            typst_bytes = typst.len(),
            pdf_bytes = pdf.len(),
            "rendered PDF"
        );
    } else {
        info!(elapsed_ms, pdf_bytes = pdf.len(), "rendered PDF");
    }

    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("could not create directory `{}`", parent.display()))?;
    }

    if let (Some(typst_path), Some(typst)) = (&typst_path, &typst) {
        fs::write(typst_path, typst)
            .with_context(|| format!("could not write `{}`", typst_path.display()))?;
        info!(path = %typst_path.display(), "wrote Typst source");
    }

    fs::write(&output, &pdf).with_context(|| format!("could not write `{}`", output.display()))?;

    info!(
        path = %output.display(),
        bytes = pdf.len(),
        "wrote PDF"
    );
    Ok(())
}

/// Logging goes to stderr; set `RUST_LOG=debug` for details.
fn init_tracing() {
    let filter = if std::env::var_os("RUST_LOG").is_some() {
        match EnvFilter::try_from_default_env() {
            Ok(filter) => filter,
            Err(error) => {
                eprintln!("warning: invalid RUST_LOG ({error}); using tome_md=info");
                EnvFilter::new("tome_md=info")
            }
        }
    } else {
        EnvFilter::new("tome_md=info")
    };
    if let Err(error) = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init()
    {
        eprintln!("warning: logging could not be initialized: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_plain_source_is_valid() {
        let template = default_template().expect("valid bundled plain template");
        assert!(
            template
                .entry()
                .params()
                .iter()
                .any(|param| param.name == "body")
        );
    }
}
