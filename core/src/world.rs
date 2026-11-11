use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};
use typst_kit::downloader::SystemDownloader;
use typst_kit::fonts::{self as kit_fonts, FontStore};
use typst_kit::packages::SystemPackages;

/// The standard library is immutable; build it once per process.
static LIBRARY: LazyLock<LazyHash<Library>> =
    LazyLock::new(|| LazyHash::new(Library::builder().build()));

/// Minimal Typst world for one in-memory source, template fonts/assets and
/// optional system fallback fonts.
pub struct PdfWorld {
    fonts: FontStore,
    packages: SystemPackages,
    main: Source,
    main_id: FileId,
    asset_dir: Option<PathBuf>,
    assets: Mutex<HashMap<FileId, Bytes>>,
    date: Datetime,
}

impl PdfWorld {
    pub fn new(source: String, asset_dir: Option<PathBuf>, use_system_fonts: bool) -> Self {
        let mut fonts = FontStore::new();
        // Typst's bundled base fonts (Libertinus Serif, New Computer Modern,
        // DejaVu Sans Mono) keep font-less templates usable.
        fonts.extend(kit_fonts::embedded());

        if let Some(dir) = &asset_dir {
            fonts.extend(crate::fonts::template_fonts(dir));
        }
        if use_system_fonts {
            tracing::debug!("loading selected system fallback fonts; this can be expensive");
            fonts.extend(crate::fonts::system_fallbacks());
        }

        let main_id = main_id();
        Self {
            fonts,
            packages: SystemPackages::new(SystemDownloader::new("tome-md")),
            main: Source::new(main_id, source),
            main_id,
            asset_dir: asset_dir.map(normalize_asset_dir),
            assets: Mutex::new(HashMap::new()),
            // Deterministic: no clock access.
            date: Datetime::from_ymd(2026, 7, 28).expect("valid date"),
        }
    }

    /// The generated Typst source used by this world.
    pub fn source_text(&self) -> &str {
        self.main.text()
    }

    fn cached_asset(&self, id: FileId) -> Option<Bytes> {
        self.assets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&id)
            .cloned()
    }

    fn cache_asset(&self, id: FileId, bytes: Bytes) {
        self.assets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(id, bytes);
    }
}

fn normalize_asset_dir(dir: PathBuf) -> PathBuf {
    let dir = if dir.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        dir
    };
    fs::canonicalize(&dir).unwrap_or(dir)
}

fn relative_file_error(error: std::io::Error, path: &str) -> FileError {
    match error.kind() {
        std::io::ErrorKind::NotFound => FileError::NotFound(PathBuf::from(path)),
        std::io::ErrorKind::PermissionDenied => FileError::AccessDenied,
        _ => FileError::Other(Some(
            format!("cannot read template asset {path}: {error}").into(),
        )),
    }
}

fn main_id() -> FileId {
    RootedPath::new(
        VirtualRoot::Project,
        VirtualPath::new("main.typ").expect("valid virtual path"),
    )
    .intern()
}

impl typst::World for PdfWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main_id {
            return Ok(self.main.clone());
        }

        let bytes = self.file(id)?;
        let text = String::from_utf8(bytes.as_slice().to_vec())
            .map_err(|_error| FileError::InvalidUtf8)?;
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main_id {
            return Ok(Bytes::new(self.main.text().as_bytes().to_vec()));
        }
        if let Some(bytes) = self.cached_asset(id) {
            return Ok(bytes);
        }

        if let VirtualRoot::Package(spec) = id.root() {
            let root = self.packages.obtain(spec).map_err(|error| {
                FileError::Other(Some(format!("cannot load package {spec}: {error}").into()))
            })?;
            let bytes = root.load(id.vpath())?;
            self.cache_asset(id, bytes.clone());
            return Ok(bytes);
        }

        let path = id.vpath().get_without_slash();
        let Some(dir) = &self.asset_dir else {
            return Err(FileError::NotFound(PathBuf::from(path)));
        };

        let relative = Path::new(path);
        if relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(FileError::NotFound(PathBuf::from(path)));
        }

        // Resolve on demand and keep the file inside the template directory,
        // even when the path goes through a symlink.
        let file = dir.join(relative);
        let resolved = fs::canonicalize(&file).map_err(|error| relative_file_error(error, path))?;
        if !resolved.starts_with(dir) {
            return Err(FileError::NotFound(PathBuf::from(path)));
        }

        let data = fs::read(&resolved).map_err(|error| relative_file_error(error, path))?;
        let bytes = Bytes::new(data);
        self.cache_asset(id, bytes.clone());
        Ok(bytes)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        Some(self.date)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use typst::World;
    use typst::diag::FileError;

    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("tome-md-{label}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn file_id(path: &str) -> FileId {
        RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new(path).expect("valid virtual path"),
        )
        .intern()
    }

    fn cleanup(dir: &Path) {
        fs::remove_dir_all(dir).expect("remove temp dir");
    }

    #[test]
    fn source_can_read_a_template_file() {
        let dir = temp_dir("source");
        fs::write(dir.join("partial.typ"), "Partial\n").expect("write partial");
        let world = PdfWorld::new("Main\n".to_owned(), Some(dir.clone()), false);

        let source = world.source(file_id("partial.typ")).expect("read source");
        assert_eq!(source.text(), "Partial\n");

        cleanup(&dir);
    }

    #[test]
    fn file_does_not_read_an_asset_named_main() {
        let dir = temp_dir("main");
        fs::write(dir.join("main.typ"), "asset").expect("write asset");
        let world = PdfWorld::new("generated".to_owned(), Some(dir.clone()), false);

        let data = world.file(world.main()).expect("read main file");
        assert_eq!(
            std::str::from_utf8(data.as_slice()).expect("utf8"),
            "generated"
        );

        cleanup(&dir);
    }

    #[test]
    fn missing_assets_are_not_found() {
        let dir = temp_dir("missing");
        let world = PdfWorld::new("Main".to_owned(), Some(dir.clone()), false);

        let error = world.file(file_id("missing.txt"));
        assert!(
            matches!(error, Err(FileError::NotFound(_))),
            "expected NotFound, got {error:?}"
        );

        cleanup(&dir);
    }

    #[test]
    fn embedded_fonts_are_available() {
        let world = PdfWorld::new("Hello".to_owned(), None, false);
        assert!(world.book().contains_family("libertinus serif"));
    }

    #[test]
    fn assets_are_cached() {
        let dir = temp_dir("cache");
        fs::write(dir.join("cached.txt"), "cached").expect("write asset");
        let world = PdfWorld::new("Main".to_owned(), Some(dir.clone()), false);

        let first = world.file(file_id("cached.txt")).expect("read asset");
        fs::remove_file(dir.join("cached.txt")).expect("remove asset");
        let second = world
            .file(file_id("cached.txt"))
            .expect("read cached asset");
        assert_eq!(first.as_slice(), second.as_slice());

        cleanup(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_cannot_escape_the_asset_directory() {
        let dir = temp_dir("symlink");
        let outside = temp_dir("outside");
        fs::write(outside.join("secret.txt"), "secret").expect("write secret");
        std::os::unix::fs::symlink(outside.join("secret.txt"), dir.join("link.txt"))
            .expect("create symlink");
        let world = PdfWorld::new("Main".to_owned(), Some(dir.clone()), false);

        let error = world.file(file_id("link.txt"));
        assert!(
            matches!(
                error,
                Err(FileError::NotFound(_)) | Err(FileError::AccessDenied)
            ),
            "expected a contained-file error, got {error:?}"
        );

        cleanup(&dir);
        cleanup(&outside);
    }
}
