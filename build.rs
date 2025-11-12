fn main() {
    // GPM support is now handled via dlopen at runtime
    // No build-time linking required - works for both native and cross-compilation
    println!("cargo:rerun-if-changed=build.rs");
}
