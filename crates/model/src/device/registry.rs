use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::DeviceError;
use super::DeviceProfile;

/// Registry of known touchscreen device profiles.
///
/// Built-in profiles are compiled into the binary (`include_str!` of
/// `crates/model/devices/*.toml`); additional profiles load from a
/// directory via [`DeviceRegistry::load_dir`]. Lookups by name or by the
/// evdev `vendor:product` pair.
#[derive(Debug, Default)]
pub struct DeviceRegistry {
    /// All registered profiles - builtins first, then loaded ones.
    pub profiles: Vec<DeviceProfile>,
    /// Files that failed to load, with their errors.
    pub errors: Vec<(PathBuf, DeviceError)>,
}

impl DeviceRegistry {
    /// The compiled-in profiles of known touchscreens.
    pub fn builtin() -> Self {
        const SIS_HID_TOUCH: &str = include_str!("../../devices/sis-hid-touch.toml");
        const ILITEK_TOUCH: &str = include_str!("../../devices/ilitek-touch.toml");
        let mut registry = Self::default();
        for (name, toml) in [
            ("sis-hid-touch.toml", SIS_HID_TOUCH),
            ("ilitek-touch.toml", ILITEK_TOUCH),
        ] {
            match toml::from_str::<DeviceProfile>(toml) {
                Ok(profile) => registry.profiles.push(profile),
                Err(source) => registry.errors.push((
                    PathBuf::from(name),
                    DeviceError::Parse {
                        path: PathBuf::from(name),
                        source,
                    },
                )),
            }
        }
        registry
    }

    /// Loads every `*.toml` in `dir` and merges it with the builtins.
    pub fn with_dir(mut self, dir: &Path) -> Self {
        let mut files: Vec<PathBuf> = match fs::read_dir(dir) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|p| p.extension().map(|ext| ext == "toml").unwrap_or(false))
                .collect(),
            Err(error) => {
                self.errors.push((
                    dir.to_path_buf(),
                    DeviceError::Io {
                        path: dir.to_path_buf(),
                        source: error,
                    },
                ));
                return self;
            }
        };
        files.sort();
        for file in files {
            match fs::read_to_string(&file)
                .map_err(|source| DeviceError::Io {
                    path: file.clone(),
                    source,
                })
                .and_then(|content| {
                    toml::from_str::<DeviceProfile>(&content).map_err(|source| DeviceError::Parse {
                        path: file.clone(),
                        source,
                    })
                }) {
                Ok(profile) => self.profiles.push(profile),
                Err(error) => self.errors.push((file, error)),
            }
        }
        self
    }

    /// Finds a profile by exact or case-insensitive name match.
    pub fn find_by_name(&self, name: &str) -> Option<&DeviceProfile> {
        self.profiles.iter().find(|p| p.name == name).or_else(|| {
            self.profiles
                .iter()
                .find(|p| p.name.eq_ignore_ascii_case(name))
        })
    }

    /// Finds a profile by the evdev `vendor:product` pair.
    pub fn find_by_ids(&self, vendor: u16, product: u16) -> Option<&DeviceProfile> {
        self.profiles
            .iter()
            .find(|p| p.vendor == Some(vendor) && p.product == Some(product))
    }

    /// Finds a profile by registry name, or by loading it as a TOML file
    /// when no name matches.
    pub fn resolve(&self, name_or_path: &str) -> Option<DeviceProfile> {
        self.find_by_name(name_or_path).cloned().or_else(|| {
            let path = Path::new(name_or_path);
            fs::read_to_string(path)
                .ok()
                .and_then(|content| toml::from_str::<DeviceProfile>(&content).ok())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceRegistry;

    #[test]
    fn builtin_registry_contains_sis_touch() {
        let registry = DeviceRegistry::builtin();
        assert!(registry.errors.is_empty());
        let profile = registry
            .find_by_name("SiS HID Touch Controller")
            .expect("builtin profile");
        assert_eq!(profile.vendor, Some(0x457));
        assert_eq!(profile.product, Some(0x11dc));
        assert_eq!(profile.x_res.as_f64(), 7.0);
    }

    #[test]
    fn finds_by_vendor_product() {
        let registry = DeviceRegistry::builtin();
        assert!(registry.find_by_ids(0x457, 0x11dc).is_some());
        assert!(registry.find_by_ids(0x0, 0x0).is_none());
    }
}
