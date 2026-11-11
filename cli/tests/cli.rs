use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tome"))
}

const SAMPLE: &str = "---\ntitle: Sample\n---\n\n# Heading\n\nHello, **world**.\n";
const TEMPLATE: &str = "#let template(title, body) = {\n  heading(level: 1)[#title]\n  body\n}\n";

fn write_sample(path: &Path) {
    fs::write(path, SAMPLE).expect("write sample");
}

fn write_template(dir: &Path) -> PathBuf {
    let path = dir.join("template.typ");
    fs::write(&path, TEMPLATE).expect("write template");
    path
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "tome-md-cli-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn run(cwd: &Path, args: &[&Path]) -> Output {
    Command::new(bin())
        .current_dir(cwd)
        .env("RUST_LOG", "off")
        .args(args)
        .output()
        .expect("run tome-md")
}

fn cleanup(dir: &Path) {
    fs::remove_dir_all(dir).expect("remove temp dir");
}

#[test]
fn refuses_to_overwrite_the_input_markdown() {
    let dir = temp_dir("overwrite-input");
    let template = write_template(&dir);
    let input = dir.join("input.md");
    write_sample(&input);
    let original = fs::read(&input).expect("read input");

    let output = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            template.as_path(),
            Path::new("-o"),
            input.as_path(),
        ],
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(fs::read(&input).expect("read input"), original);

    cleanup(&dir);
}

#[test]
fn refuses_to_clobber_the_pdf_with_keep_typ() {
    let dir = temp_dir("keep-typ");
    let template = write_template(&dir);
    let input = dir.join("input.md");
    let output_path = dir.join("out.typ");
    write_sample(&input);

    let output = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            template.as_path(),
            Path::new("-o"),
            output_path.as_path(),
            Path::new("--keep-typ"),
        ],
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");

    cleanup(&dir);
}

#[test]
fn force_never_overwrites_the_input_with_keep_typ() {
    let dir = temp_dir("keep-typ-input");
    let template = write_template(&dir);
    let input = dir.join("input.typ");
    write_sample(&input);
    let original = fs::read(&input).expect("read input");

    let output = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            template.as_path(),
            Path::new("--keep-typ"),
            Path::new("--force"),
        ],
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("would overwrite the input Markdown"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(&input).expect("read input"), original);

    cleanup(&dir);
}

#[test]
fn force_never_overwrites_the_template_with_the_pdf() {
    let dir = temp_dir("template-pdf");
    let template = write_template(&dir);
    let input = dir.join("input.md");
    write_sample(&input);
    let original = fs::read(&template).expect("read template");

    let output = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            template.as_path(),
            Path::new("-o"),
            template.as_path(),
            Path::new("--force"),
        ],
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("refusing to overwrite the template"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(&template).expect("read template"), original);

    cleanup(&dir);
}

#[test]
fn force_never_overwrites_the_template_with_keep_typ() {
    let dir = temp_dir("template-keep-typ");
    let template = write_template(&dir);
    let input = dir.join("input.md");
    write_sample(&input);
    let original = fs::read(&template).expect("read template");

    let output = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            template.as_path(),
            Path::new("-o"),
            dir.join("template.pdf").as_path(),
            Path::new("--keep-typ"),
            Path::new("--force"),
        ],
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("refusing to overwrite the template"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(&template).expect("read template"), original);

    cleanup(&dir);
}

#[test]
fn refuses_to_overwrite_an_existing_output() {
    let dir = temp_dir("existing");
    let template = write_template(&dir);
    let input = dir.join("input.md");
    let output_path = dir.join("out.pdf");
    write_sample(&input);
    fs::write(&output_path, "existing").expect("seed output");

    let output = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            template.as_path(),
            Path::new("-o"),
            output_path.as_path(),
        ],
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(
        fs::read_to_string(&output_path).expect("read output"),
        "existing"
    );

    cleanup(&dir);
}

#[test]
fn plain_template_value_is_a_real_path() {
    let dir = temp_dir("plain");
    let input = dir.join("input.md");
    let output_path = dir.join("out.pdf");
    let template_path = dir.join("plain");
    fs::write(
        &template_path,
        "#let template(title: none, body) = {\n  [LOCAL TEMPLATE]\n  body\n}\n",
    )
    .expect("write template");
    fs::write(&input, "---\ntitle: T\n---\n\nHello\n").expect("write input");

    let output = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            Path::new("plain"),
            Path::new("-o"),
            output_path.as_path(),
            Path::new("--keep-typ"),
        ],
    );

    assert!(
        output.status.success(),
        "rendering failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let typst = fs::read_to_string(dir.join("out.typ")).expect("read generated Typst");
    assert!(typst.contains("LOCAL TEMPLATE"), "{typst}");

    cleanup(&dir);
}

#[test]
fn force_allows_overwriting_an_existing_output() {
    let dir = temp_dir("force");
    let template = write_template(&dir);
    let input = dir.join("input.md");
    let output_path = dir.join("out.pdf");
    write_sample(&input);
    fs::write(&output_path, "existing").expect("seed output");

    let output = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            template.as_path(),
            Path::new("-o"),
            output_path.as_path(),
            Path::new("--force"),
        ],
    );

    assert!(
        output.status.success(),
        "rendering failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = fs::read(&output_path).expect("read output");
    assert!(bytes.starts_with(b"%PDF-"));

    cleanup(&dir);
}

#[test]
fn default_output_and_keep_typ_paths_are_written() {
    let dir = temp_dir("paths");
    let template = write_template(&dir);
    let input = dir.join("input.md");
    write_sample(&input);

    let first = run(
        &dir,
        &[input.as_path(), Path::new("--template"), template.as_path()],
    );
    assert!(
        first.status.success(),
        "default render failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let default_pdf = dir.join("input.pdf");
    assert!(default_pdf.is_file());

    let second = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            template.as_path(),
            Path::new("-o"),
            dir.join("combined.pdf").as_path(),
            Path::new("--keep-typ"),
        ],
    );
    assert!(
        second.status.success(),
        "keep-typ render failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert!(dir.join("combined.pdf").is_file());
    assert!(dir.join("combined.typ").is_file());

    cleanup(&dir);
}

#[test]
fn refuses_to_overwrite_a_pdf_named_input_by_default() {
    let dir = temp_dir("input-pdf");
    let template = write_template(&dir);
    let input = dir.join("input.pdf");
    fs::write(&input, "---\ntitle: T\n---\n\nHello\n").expect("write input");
    let original = fs::read(&input).expect("read input");

    let output = run(
        &dir,
        &[input.as_path(), Path::new("--template"), template.as_path()],
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(fs::read(&input).expect("read input"), original);

    cleanup(&dir);
}

#[test]
fn missing_explicit_template_is_a_usage_error() {
    let dir = temp_dir("missing-template");
    let input = dir.join("input.md");
    write_sample(&input);

    let output = run(
        &dir,
        &[
            input.as_path(),
            Path::new("--template"),
            dir.join("missing.typ").as_path(),
        ],
    );

    assert_eq!(output.status.code(), Some(2));

    cleanup(&dir);
}
