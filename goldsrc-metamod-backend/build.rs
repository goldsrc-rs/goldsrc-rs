fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

    if target_arch == "x86" && target_env == "msvc" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let exports_path = std::path::Path::new(&manifest_dir)
            .join("src")
            .join("exports.c");

        cc::Build::new().file(exports_path).compile("msvc_exports");
    }
}
