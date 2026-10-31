//! Localization loader - Fluent assets shared with `crates/gtk` via
//! the workspace-root `i18n/` directory (I18N.md).

use i18n_embed::DesktopLanguageRequester;
use i18n_embed::LanguageRequester;
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed::fluent::fluent_language_loader;
use rust_embed::RustEmbed;
use std::sync::LazyLock;

/// Compile-time embedded localization assets (workspace `i18n/`).
#[derive(RustEmbed)]
#[folder = "../../i18n/"]
struct Localizations;

static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    let requested = DesktopLanguageRequester::new().requested_languages();
    if i18n_embed::select(&loader, &Localizations, &requested).is_err() {
        // fallback_language ("en") stays loaded - a missing locale is
        // not a failure the TUI should surface.
    }
    loader
});

/// The shared fluent language loader - passed to `fl!`.
pub fn language_loader() -> &'static FluentLanguageLoader {
    &LANGUAGE_LOADER
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    /// Top-level message IDs of an `.ftl` file - `id` starts a line
    /// directly at column 0; multiline values/selects are indented.
    fn message_ids(path: &Path) -> BTreeSet<String> {
        let contents = std::fs::read_to_string(path).expect("ftl file must exist");
        contents
            .lines()
            .filter(|line| {
                line.as_bytes()
                    .first()
                    .is_some_and(|byte| byte.is_ascii_alphabetic())
            })
            .filter_map(|line| line.split('=').next().map(str::trim).map(String::from))
            .collect()
    }

    /// Every localized surface (gtk, tui) must translate each en ID
    /// into de - a missing key silently falls back, hiding drift.
    #[test]
    fn en_and_de_message_ids_match() {
        let assets = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../i18n"));
        for surface in ["tangible-objects-gtk", "tangible-objects-tui"] {
            let en = message_ids(&assets.join(format!("en/{surface}.ftl")));
            let de = message_ids(&assets.join(format!("de/{surface}.ftl")));
            let only_en: Vec<_> = en.difference(&de).collect();
            let only_de: Vec<_> = de.difference(&en).collect();
            assert!(
                only_en.is_empty() && only_de.is_empty(),
                "{surface}: en-only {only_en:?}, de-only {only_de:?}"
            );
        }
    }
}
