//! Template font metadata and a small, lazily loaded set of system fallbacks.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

use tracing::{debug, warn};
use typst::text::FontInfo;
use typst_kit::fonts::{self as kit_fonts, FontPath};

static SYSTEM_FALLBACKS: LazyLock<Vec<CachedFont>> = LazyLock::new(load_system_fallbacks);
static TEMPLATE_FONTS: LazyLock<Mutex<HashMap<PathBuf, Arc<Vec<CachedFont>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
struct CachedFont {
    path: PathBuf,
    index: u32,
    info: FontInfo,
}

impl CachedFont {
    fn entry(&self) -> (FontPath, FontInfo) {
        (
            FontPath {
                path: self.path.clone(),
                index: self.index,
            },
            self.info.clone(),
        )
    }
}

/// Template font metadata, scanned once per canonical template directory.
/// Font bytes are still loaded by `FontStore` only when Typst requests them.
pub(crate) fn template_fonts(dir: &Path) -> Vec<(FontPath, FontInfo)> {
    let key = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let fonts = {
        let mut cache = TEMPLATE_FONTS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(fonts) = cache.get(&key) {
            Arc::clone(fonts)
        } else {
            let scanned = Arc::new(
                kit_fonts::scan(dir)
                    .map(|(path, info)| CachedFont {
                        path: path.path,
                        index: path.index,
                        info,
                    })
                    .collect::<Vec<_>>(),
            );
            cache.insert(key, Arc::clone(&scanned));
            scanned
        }
    };

    fonts.iter().map(CachedFont::entry).collect()
}

/// Metadata for selected system fallbacks; font data is loaded by `FontStore`
/// only when Typst actually requests it.
pub(crate) fn system_fallbacks() -> impl Iterator<Item = (FontPath, FontInfo)> {
    SYSTEM_FALLBACKS.iter().map(CachedFont::entry)
}

fn load_system_fallbacks() -> Vec<CachedFont> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let mut fonts = Vec::new();
    let mut seen = HashSet::new();
    let mut covered = [false; 4];
    for family in ["Arial", "Liberation Sans", "DejaVu Sans"] {
        extend_family(&db, family, &mut fonts, &mut seen, &mut covered);
    }

    if fonts.is_empty() {
        warn!("no system sans fallback font found");
    }

    let mut mono_seen = HashSet::new();
    let mut mono_covered = [false; 4];
    for family in ["DejaVu Sans Mono", "Liberation Mono", "Courier New"] {
        extend_family(&db, family, &mut fonts, &mut mono_seen, &mut mono_covered);
    }

    debug!(fonts = fonts.len(), "loaded system fallback metadata");
    fonts
}

const STYLES: [(fontdb::Weight, fontdb::Style); 4] = [
    (fontdb::Weight::NORMAL, fontdb::Style::Normal),
    (fontdb::Weight::BOLD, fontdb::Style::Normal),
    (fontdb::Weight::NORMAL, fontdb::Style::Italic),
    (fontdb::Weight::BOLD, fontdb::Style::Italic),
];

fn extend_family(
    db: &fontdb::Database,
    family: &str,
    fonts: &mut Vec<CachedFont>,
    seen: &mut HashSet<(PathBuf, u32)>,
    covered: &mut [bool; 4],
) {
    for (index, (weight, style)) in STYLES.iter().copied().enumerate() {
        if covered[index] {
            continue;
        }
        let Some(font) = query_face(db, family, weight, style) else {
            continue;
        };
        if seen.insert((font.path.clone(), font.index)) {
            fonts.push(font);
        }
        covered[index] = true;
    }
}

fn query_face(
    db: &fontdb::Database,
    family: &str,
    weight: fontdb::Weight,
    style: fontdb::Style,
) -> Option<CachedFont> {
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(family)],
        weight,
        style,
        ..fontdb::Query::default()
    };
    let id = db.query(&query)?;
    let face = db.face(id)?;
    if face.weight != weight
        || (style == fontdb::Style::Normal && face.style != fontdb::Style::Normal)
        || (style != fontdb::Style::Normal && face.style == fontdb::Style::Normal)
    {
        return None;
    }
    let path = match &face.source {
        fontdb::Source::File(path) | fontdb::Source::SharedFile(path, _) => path.clone(),
        fontdb::Source::Binary(_) => return None,
    };
    let info = db.with_face_data(id, FontInfo::new).flatten()?;
    Some(CachedFont {
        path,
        index: face.index,
        info,
    })
}
