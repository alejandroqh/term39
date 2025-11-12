use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();

    // Only handle GPM linking for Linux targets
    if target.contains("linux") {
        // If we're cross-compiling (host != target), create a stub library
        if host != target {
            create_stub_libgpm();
        }
        // The actual libgpm.so will be required at runtime on Linux
    }
}

fn create_stub_libgpm() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let stub_dir = out_dir.join("stub_libs");
    std::fs::create_dir_all(&stub_dir).ok();

    // Create a stub libgpm.so with empty implementations
    let stub_code = r#"
        int Gpm_Open(void* conn, int flag) { return -1; }
        int Gpm_Close(void) { return 0; }
        int Gpm_GetEvent(void* event) { return -1; }
        int Gpm_Getc(void* f) { return -1; }
    "#;

    let stub_c = stub_dir.join("gpm_stub.c");
    std::fs::write(&stub_c, stub_code).unwrap();

    let rust_target = env::var("TARGET").unwrap();
    let stub_lib = stub_dir.join("libgpm.so");

    println!("cargo:rustc-link-search=native={}", stub_dir.display());
    println!("cargo:rerun-if-changed=build.rs");

    // Convert Rust target to Zig target format
    // Rust: x86_64-unknown-linux-gnu -> Zig: x86_64-linux-gnu
    let zig_target = rust_target.replace("unknown-", "");

    // Try to use zig cc to compile the stub for the target platform
    let mut tried_zig = false;
    let output = std::process::Command::new("zig")
        .args(["cc", "-target", &zig_target, "-shared", "-fPIC", "-o"])
        .arg(&stub_lib)
        .arg(&stub_c)
        .output();

    if output.is_ok_and(|o| o.status.success()) {
        tried_zig = true;
    }

    // If zig compilation didn't work, try gcc
    if !tried_zig {
        let _ = std::process::Command::new("gcc")
            .args(["-shared", "-fPIC", "-o"])
            .arg(&stub_lib)
            .arg(&stub_c)
            .output();
    }

    // Check if the stub library was created
    if !stub_lib.exists() {
        eprintln!(
            "Warning: Could not create stub libgpm.so. The library must be present at runtime on Linux."
        );
    }
}
