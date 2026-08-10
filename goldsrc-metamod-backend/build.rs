fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

    if target_arch == "x86" && target_env == "msvc" {
        cc::Build::new()
            .file("src/exports.c")
            .compile("msvc_exports");
    }
}
