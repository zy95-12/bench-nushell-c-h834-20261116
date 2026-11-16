use std::path::PathBuf;

use crate::AbsolutePathBuf;

pub fn home_dir() -> Option<AbsolutePathBuf> {
    dirs::home_dir().and_then(|home| AbsolutePathBuf::try_from(home).ok())
}

/// Return the data directory for the current platform or XDG_DATA_HOME if specified.
pub fn data_dir() -> Option<AbsolutePathBuf> {
    configurable_dir_path("XDG_DATA_HOME", dirs::data_dir)
}

/// Return the cache directory for the current platform or XDG_CACHE_HOME if specified.
pub fn cache_dir() -> Option<AbsolutePathBuf> {
    configurable_dir_path("XDG_CACHE_HOME", dirs::cache_dir)
}

/// Return the nushell config directory.
pub fn nu_config_dir() -> Option<AbsolutePathBuf> {
    configurable_dir_path("XDG_CONFIG_HOME", dirs::config_dir).map(|mut p| {
        p.push("nushell");
        p
    })
}

fn configurable_dir_path(
    name: &'static str,
    dir: impl FnOnce() -> Option<PathBuf>,
) -> Option<AbsolutePathBuf> {
    std::env::var(name)
        .ok()
        .and_then(|path| AbsolutePathBuf::try_from(path).ok())
        .or_else(|| dir().and_then(|path| AbsolutePathBuf::try_from(path).ok()))
        .map(|path| path.canonicalize().map(Into::into).unwrap_or(path))
}

/// Whether a path names a Windows device rather than an ordinary file.
pub fn is_windows_device_path(path: impl AsRef<std::path::Path>) -> bool {
    if !cfg!(windows) {
        return false;
    }
    let path = path.as_ref().to_string_lossy();
    if path.starts_with(r"\\.\") {
        return true;
    }
    let name = path.rsplit(['/', '\\']).next().unwrap_or_default();
    let name = name
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches([' ', ':']);
    let name = name.to_ascii_uppercase();
    matches!(
        name.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || ["COM", "LPT"].iter().any(|prefix| {
        name.strip_prefix(prefix).is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    })
}

#[cfg(test)]
mod device_path_tests {
    use super::is_windows_device_path;

    #[test]
    fn platform_device_names() {
        for name in ["CON", "con", "NUL", "COM1", "LPT9", r"\\.\CON", "NUL.txt"] {
            assert_eq!(is_windows_device_path(name), cfg!(windows), "{name}");
        }
        for name in ["ordinary.txt", "COM10", "COM0", "console", "/tmp/file"] {
            assert!(!is_windows_device_path(name), "{name}");
        }
    }
}
