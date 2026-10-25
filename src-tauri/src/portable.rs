use std::path::PathBuf;

/// Folder that contains the running executable.
pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Portable mode is active when `framewise.portable` sits next to the exe.
pub fn is_portable() -> bool {
    exe_dir().join("framewise.portable").exists()
}

/// Where the database lives. Portable keeps everything beside the executable.
pub fn data_dir(fallback: PathBuf) -> PathBuf {
    if is_portable() {
        exe_dir().join("data")
    } else {
        fallback
    }
}

/// Keep WebView2's profile inside the portable folder so nothing is written
/// to AppData/LocalAppData. Harmless to call in the installed build.
pub fn prepare_webview_data() {
    if !is_portable() {
        return;
    }
    let dir = exe_dir().join("data").join("webview");
    let _ = std::fs::create_dir_all(&dir);
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &dir);
}
