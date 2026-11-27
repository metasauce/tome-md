# Development

## Crate layout

```
core/                     tome-md-core library
├─ Cargo.toml
├─ src/
│  ├─ lib.rs              render(markdown, template, options) -> { typst, pdf }
│  ├─ error.rs            thiserror error enum
│  ├─ fonts.rs            selected system fallback metadata
│  ├─ markdown.rs         Markdown → Typst emitter
│  ├─ templates/
│  │  ├─ mod.rs           public template API
│  │  ├─ entry.rs         path, source and signature parsing
│  │  └─ template.rs      options, asset directory, validation and show rule
│  └─ world.rs            minimal Typst World (source, fonts, assets, packages)
└─ tests/fixtures/        template fixture for unit tests

cli/                      tome-md binary
├─ Cargo.toml
├─ src/
│  ├─ main.rs             clap CLI
│  └─ plain.typ           bundled default template
└─ tests/cli.rs           end-to-end CLI tests

examples/                 example templates (letter-din, cv, notes, recipe, flyer, feature_tour)
```

`core` owns the Typst, Markdown and font dependencies. `cli` owns clap, anyhow,
logging and the bundled plain source, and depends on the `tome-md-core`
library. The rendering pipeline is pure Rust, so there is no external `typst`
binary to invoke.

## Logging

The CLI uses `tracing` + `tracing-subscriber`. Logs go to stderr; the default
filter is `tome_md=info`, so a successful run reports the loaded template,
render timing and output files. Use `RUST_LOG` for more:

```sh
RUST_LOG=debug cargo run -p tome-md -- notes.md
RUST_LOG=tome_md=debug cargo run -p tome-md -- notes.md
RUST_LOG=warn cargo run -p tome-md -- notes.md
```

## Errors

The library returns typed errors (`core/src/error.rs`); the CLI adds context
with `anyhow` and prints the full chain to stderr:

```
Error: could not render `notes.md`

Caused by:
    template template.typ is missing front matter key(s): title
```

`main` returns the `anyhow` result, so runtime errors are always printed and
exit 1, independently of `RUST_LOG`. Explicit `--template` loading failures are
reported by clap as usage errors (exit 2).

## Build speed

- `cargo build` / `cargo run` use the debug profile. The first build compiles
  Typst (~2 min), then incremental rebuilds take about a second.
- Debug runtime is fine for iteration: ~0.3 s for a 2-page document and ~4 s
  for a 49-page one.
- `cargo build -p tome-md --release` uses `opt-level = 3` without LTO, so
  linking is much faster than a size-oriented LTO build while still producing a
  fast binary. It is the default for `cargo install` and for iteration.
- `cargo build -p tome-md --profile small` is the size-oriented build
  (`opt-level = "z"`, fat LTO, one codegen unit). On x86_64 it took ~3.5 min
  and produced a ~35.6 MB binary, compared with ~1.7 min and ~47.8 MB for
  `release`. Use `target/small/tome`. The `z` opt level trades some render
  speed for size; use `release` when iterating on layout or timing.

## Markdown formatting

Markdown is formatted with [Prettier](https://prettier.io/) through
[prek](https://prek.j178.dev/):

```sh
cargo install --locked prek
prek install
```

Run the hook manually with:

```sh
prek run --all-files
```

The hook config is in `.pre-commit-config.yaml`; Prettier options are in
`.prettierrc.json`. CI runs the same Prettier version on Markdown changes.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI (`.github/workflows/rust-checks.yml`) runs formatting, clippy and tests for
changes touching Rust sources, manifests or the lockfile. The release build only
happens when a tag is published, in `.github/workflows/release.yml`.
The workspace lints turn panics, unwraps and silent error drops into warnings,
so clippy is the main gate.
