fn main() {
    // GPM support is now handled via dlopen at runtime
    // No build-time linking required - works for both native and cross-compilation
    println!("cargo:rerun-if-changed=build.rs");

    // Embed icon and metadata into Windows executable
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/term39.ico");
        res.set("ProductName", "term39");
        res.set(
            "FileDescription",
            "A modern, retro-styled terminal multiplexer with a classic MS-DOS aesthetic",
        );
        res.set("LegalCopyright", "Copyright (c) 2025 Alejandro Quintanar");
        res.compile().expect("Failed to compile Windows resources");
    }
}
