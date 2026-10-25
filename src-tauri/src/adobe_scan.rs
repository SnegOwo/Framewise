use std::collections::HashSet;
use std::path::{Path, PathBuf};

use jwalk::WalkDir;
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
};
use winreg::RegKey;

use crate::models::DetectedApp;

const DRIVE_MAX_DEPTH: usize = 8;
const FREE_DEPTH: usize = 6;

const CANDIDATES: &[(&str, &str)] = &[
    ("photoshop.exe", "Photoshop.exe"),
    ("adobe premiere pro.exe", "Adobe Premiere Pro.exe"),
    ("afterfx.exe", "Adobe After Effects.exe"),
    ("adobe media encoder.exe", "Adobe Media Encoder.exe"),
    ("illustrator.exe", "Illustrator.exe"),
    ("adobe audition.exe", "Adobe Audition.exe"),
    ("audition.exe", "Adobe Audition.exe"),
    ("indesign.exe", "Adobe InDesign.exe"),
    ("adobe animate.exe", "Adobe Animate.exe"),
    ("animate.exe", "Adobe Animate.exe"),
    (
        "adobe character animator.exe",
        "Adobe Character Animator.exe",
    ),
    ("character animator.exe", "Adobe Character Animator.exe"),
    ("lightroom.exe", "Lightroom.exe"),
    ("lightroomclassic.exe", "Lightroom Classic.exe"),
    ("adobe bridge.exe", "Adobe Bridge.exe"),
    ("adobe premiere rush.exe", "Adobe Premiere Rush.exe"),
    ("adobe xd.exe", "Adobe XD.exe"),
    ("acrobat.exe", "Acrobat.exe"),
    ("substance 3d painter.exe", "Substance 3D Painter.exe"),
    ("substance 3d designer.exe", "Substance 3D Designer.exe"),
];

const SKIP_DIRS: &[&str] = &[
    "windows",
    "winsxs",
    "system32",
    "syswow64",
    "$recycle.bin",
    "system volume information",
    "recovery",
    "perflogs",
    "node_modules",
    ".git",
    ".svn",
    ".hg",
    "target",
    "cache",
    "caches",
    "temp",
    "tmp",
    "packages",
    "windowsapps",
    "appdata",
    "onedrive",
    "installer",
    "assembly",
    "driverstore",
    "drivers",
    "microsoft",
    "google",
    "mozilla",
    "steamapps",
    "epic games",
    "riot games",
    "battle.net",
    "docker",
    "wsl",
    "anaconda3",
    "miniconda3",
    ".cargo",
    ".rustup",
    ".gradle",
    "__pycache__",
    ".venv",
    "venv",
];

pub fn scan() -> Vec<DetectedApp> {
    let mut files: HashSet<PathBuf> = HashSet::new();
    let mut roots: Vec<(PathBuf, usize, usize, bool)> = Vec::new();

    for drive in fixed_drives() {
        roots.push((drive, DRIVE_MAX_DEPTH, FREE_DEPTH, false));
    }

    for base in adobe_bases() {
        if base.is_dir() {
            roots.push((base, 7, 7, true));
        }
    }

    for location in registry_roots() {
        if location.is_dir() {
            roots.push((location, 7, 7, true));
        }
    }

    for path in registry_app_paths() {
        if path.is_file() {
            files.insert(path);
        }
    }

    let mut seen_roots = HashSet::new();
    for (root, max_depth, free_depth, trusted) in roots {
        let key = root.to_string_lossy().to_lowercase();
        if !seen_roots.insert(key) {
            continue;
        }
        collect_from_root(&root, max_depth, free_depth, trusted, &mut files);
    }

    let mut out: Vec<DetectedApp> = files
        .into_iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy();
            let app = canonical(&name)?;
            Some(DetectedApp {
                app: app.to_string(),
                path: path.to_string_lossy().to_string(),
            })
        })
        .collect();
    out.sort_by(|a, b| {
        a.app
            .to_lowercase()
            .cmp(&b.app.to_lowercase())
            .then_with(|| a.path.to_lowercase().cmp(&b.path.to_lowercase()))
    });
    out.dedup_by(|a, b| a.path.eq_ignore_ascii_case(&b.path));
    out
}

fn collect_from_root(
    root: &Path,
    max_depth: usize,
    free_depth: usize,
    trusted: bool,
    files: &mut HashSet<PathBuf>,
) {
    let walker = WalkDir::new(root).max_depth(max_depth).process_read_dir(
        move |depth, path, _state, children| {
            let relevant_ancestor = trusted
                || depth.map(|value| value <= free_depth).unwrap_or(true)
                || path
                    .components()
                    .any(|part| relevant_dir(&part.as_os_str().to_string_lossy()));
            children.retain(|entry| {
                let Ok(entry) = entry else {
                    return false;
                };
                let name = entry.file_name().to_string_lossy();
                let lower = name.to_ascii_lowercase();
                if entry.file_type().is_dir() {
                    if skip_dir(&lower) {
                        return false;
                    }
                    trusted || relevant_ancestor || relevant_dir(&lower)
                } else {
                    entry.file_type().is_file() && canonical(&name).is_some()
                }
            });
        },
    );

    for entry in walker.into_iter().flatten() {
        if entry.file_type().is_file() {
            files.insert(entry.path());
        }
    }
}

fn canonical(name: &str) -> Option<&'static str> {
    CANDIDATES
        .iter()
        .find(|(exe, _)| name.eq_ignore_ascii_case(exe))
        .map(|(_, app)| *app)
}

fn relevant_dir(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("adobe")
        || [
            "photoshop",
            "premiere",
            "after effects",
            "afterfx",
            "illustrator",
            "lightroom",
            "audition",
            "indesign",
            "animate",
            "media encoder",
            "character animator",
            "bridge",
            "acrobat",
            "creative cloud",
            "substance",
        ]
        .iter()
        .any(|part| lower.contains(part))
}

fn skip_dir(lower: &str) -> bool {
    SKIP_DIRS.contains(&lower)
}

fn adobe_bases() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for key in [
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramData",
        "LOCALAPPDATA",
        "APPDATA",
    ] {
        if let Some(base) = std::env::var_os(key) {
            out.push(PathBuf::from(base).join("Adobe"));
        }
    }
    out
}

fn registry_roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
            let Ok(base) = RegKey::predef(hive).open_subkey_with_flags(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
                KEY_READ | view,
            ) else {
                continue;
            };
            for name in base.enum_keys().flatten() {
                let Ok(sub) = base.open_subkey_with_flags(&name, KEY_READ | view) else {
                    continue;
                };
                let display: String = sub.get_value("DisplayName").unwrap_or_default();
                if !display.to_lowercase().contains("adobe") {
                    continue;
                }
                if let Ok(location) = sub.get_value::<String, _>("InstallLocation") {
                    let location = location.trim();
                    if !location.is_empty() {
                        out.push(PathBuf::from(location));
                    }
                }
            }
        }
    }
    out
}

fn registry_app_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
            let Ok(base) = RegKey::predef(hive).open_subkey_with_flags(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
                KEY_READ | view,
            ) else {
                continue;
            };
            for name in base.enum_keys().flatten() {
                if canonical(&name).is_none() {
                    continue;
                }
                let Ok(sub) = base.open_subkey_with_flags(&name, KEY_READ | view) else {
                    continue;
                };
                if let Ok(path) = sub.get_value::<String, _>("") {
                    let trimmed = path.trim().trim_matches('"');
                    if !trimmed.is_empty() {
                        out.push(PathBuf::from(trimmed));
                    }
                }
            }
        }
    }
    out
}

fn fixed_drives() -> Vec<PathBuf> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};

    let mask = unsafe { GetLogicalDrives() };
    let mut out = Vec::new();
    for letter in b'A'..=b'Z' {
        let bit = 1u32 << (letter - b'A');
        if mask & bit == 0 {
            continue;
        }
        let root = format!("{}:\\", letter as char);
        let wide: Vec<u16> = root.encode_utf16().chain(Some(0)).collect();
        if unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr())) } == 3 {
            out.push(PathBuf::from(root));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_main_adobe_executables() {
        assert_eq!(canonical("Photoshop.exe"), Some("Photoshop.exe"));
        assert_eq!(
            canonical("ADOBE PREMIERE PRO.EXE"),
            Some("Adobe Premiere Pro.exe")
        );
        assert_eq!(canonical("AfterFX.exe"), Some("Adobe After Effects.exe"));
        assert_eq!(
            canonical("Adobe Media Encoder.exe"),
            Some("Adobe Media Encoder.exe")
        );
        assert_eq!(canonical("notepad.exe"), None);
    }

    #[test]
    fn skips_noise_directories() {
        assert!(skip_dir("node_modules"));
        assert!(skip_dir("windows"));
        assert!(!skip_dir("adobe"));
        assert!(relevant_dir("Adobe Photoshop 2026"));
        assert!(relevant_dir("Premiere Pro"));
    }
}
