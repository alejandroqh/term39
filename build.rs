fn main() {
    // GPM support is now handled via dlopen at runtime
    // No build-time linking required - works for both native and cross-compilation
    println!("cargo:rerun-if-changed=build.rs");

    // Embed icon and metadata into Windows executable
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();

        // Only set icon if assets folder exists (not present when installed via cargo install)
        let icon_path = "assets/term39.ico";
        if std::path::Path::new(icon_path).exists() {
            res.set_icon(icon_path);
        }

        res.set("ProductName", "term39");
        res.set(
            "FileDescription",
            "A modern, retro-styled terminal multiplexer with a classic MS-DOS aesthetic",
        );
        res.set("LegalCopyright", "Copyright (c) 2025 Alejandro Quintanar");
        res.compile().expect("Failed to compile Windows resources");
    }
}
