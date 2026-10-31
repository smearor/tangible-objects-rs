//! Localization loader - Fluent assets shared with `crates/tui` via
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
        // not a failure the GUI should surface.
    }
    loader
});

/// The shared fluent language loader - passed to `fl!`.
pub fn language_loader() -> &'static FluentLanguageLoader {
    &LANGUAGE_LOADER
}
