use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();

    // Only handle GPM linking for Linux targets
    if target.contains("linux") {
        // Check if libgpm is available on the system
        let gpm_available = is_libgpm_available();

        // If we're cross-compiling OR GPM is not available, create a stub library
        if host != target || !gpm_available {
            create_stub_libgpm();
        }
        // The actual libgpm.so will be required at runtime on Linux (if available)
    }
}

/// Check if libgpm is available on the system
fn is_libgpm_available() -> bool {
    // Try to compile and link a simple program that uses libgpm
    let test_code = r#"
        int main() {
            extern int Gpm_Open(void*, int);
            return 0;
        }
    "#;

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let test_c = out_dir.join("test_gpm.c");
    let test_out = out_dir.join("test_gpm");

    // Write test code
    if std::fs::write(&test_c, test_code).is_err() {
        return false;
    }

    // Try to compile and link with -lgpm
    let output = std::process::Command::new("cc")
        .args(["-o"])
        .arg(&test_out)
        .arg(&test_c)
        .arg("-lgpm")
        .output();

    // Clean up test files
    let _ = std::fs::remove_file(test_c);
    let _ = std::fs::remove_file(test_out);

    // Check if compilation succeeded
    output.is_ok_and(|o| o.status.success())
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
