// Embed the app icon into the Windows executable (Explorer, taskbar, shortcuts).
fn main() {
    println!("cargo:rerun-if-changed=assets/icon/icon.ico");
    theme_covers();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/icon/icon.ico")
            .set_manifest(MANIFEST)
            .compile()
            .expect("failed to embed Windows icon");
        copy_webview2_loader();
    }
}

// Ensures WebView2Loader.dll is copied alongside the built executable so bitchord.exe
// can be executed directly without "WebView2Loader.dll was not found" errors.
fn copy_webview2_loader() {
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(d) => std::path::PathBuf::from(d),
        Err(_) => return,
    };
    let target_dir = match out_dir.ancestors().nth(3) {
        Some(d) => d.to_path_buf(),
        None => return,
    };
    if let Ok(entries) = std::fs::read_dir(target_dir.join("build")) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("webview2-com-sys")) {
                let arch_candidates = ["out/x64/WebView2Loader.dll", "out/x86/WebView2Loader.dll", "out/arm64/WebView2Loader.dll"];
                for rel in arch_candidates {
                    let dll = p.join(rel);
                    if dll.exists() {
                        let _ = std::fs::copy(&dll, target_dir.join("WebView2Loader.dll"));
                        return;
                    }
                }
            }
        }
    }
}

// The segment heap returns freed memory to the OS far more readily than the default
// NT heap, which keeps the resident footprint down after bursts (images, JSON).
const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <heapType xmlns="http://schemas.microsoft.com/SMI/2020/WindowsSettings">SegmentHeap</heapType>
    </windowsSettings>
  </application>
</assembly>
"#;

// Embed every theme preset's cover thumbnail (assets/themes/covers/<id>.jpg) as a
// table the settings page looks up by preset id.
fn theme_covers() {
    let dir = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets/themes/covers");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "jpg")).collect())
        .unwrap_or_default();
    entries.sort();
    let mut out = String::from("&[\n");
    for p in entries {
        let id = p.file_stem().unwrap().to_string_lossy();
        out += &format!("    ({id:?}, include_bytes!({:?}).as_slice()),\n", p.display().to_string());
    }
    out += "]\n";
    std::fs::write(std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("theme_covers.rs"), out).unwrap();
}
