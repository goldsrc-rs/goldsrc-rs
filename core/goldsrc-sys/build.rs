use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("wasm32") {
        let dummy = r#"
            #[repr(C)]
            pub struct edict_t {
                pub v: entvars_t,
            }
            #[repr(C)]
            pub struct entvars_t {
                pub classname: usize,
                pub netname: usize,
                pub origin: [f32; 3],
                pub velocity: [f32; 3],
                pub health: f32,
                pub armorvalue: f32,
            }
        "#;
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        std::fs::write(out_path.join("bindings.rs"), dummy).unwrap();
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let hlsdk = repo_root.join("references").join("hlsdk");
    let metamod = repo_root
        .join("references")
        .join("metamod-r")
        .join("metamod")
        .join("extra")
        .join("example")
        .join("include")
        .join("metamod");

    // Check that references exist
    if !hlsdk.join("engine").join("eiface.h").exists() {
        panic!(
            "\n\nERROR: HLSDK not found at {}\n\
             Run the setup script first:\n\
               python3 scripts/setup.py\n\n",
            hlsdk.display()
        );
    }

    if !metamod.join("meta_api.h").exists() {
        panic!(
            "\n\nERROR: metamod-r not found at {}\n\
             Run the setup script first:\n\
               python3 scripts/setup.py\n\n",
            metamod.display()
        );
    }

    // Read local configuration (goldsrc.local.toml)
    let config_path = repo_root.join("goldsrc.local.toml");
    let config = if config_path.exists() {
        std::fs::read_to_string(&config_path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut includes: Vec<String> = Vec::new();

    // HLSDK includes
    for dir in &["public", "engine", "common", "dlls"] {
        includes.push(format!("-I{}", hlsdk.join(dir).display()));
    }

    // Metamod includes
    let metamod_parent = metamod.parent().unwrap().parent().unwrap();
    includes.push(format!("-I{}", metamod_parent.display()));
    includes.push(format!("-I{}", metamod.display()));

    // System includes from config (parse TOML array)
    // Simple parser: find all quoted strings in the include_paths array
    let mut in_include_paths = false;
    for line in config.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("include_paths") {
            in_include_paths = true;
        }

        if in_include_paths {
            // Check for array end
            if trimmed.contains(']') {
                in_include_paths = false;
            }

            // Extract quoted strings from the line
            for part in trimmed.split('"') {
                let part = part.trim().trim_end_matches(',').trim();
                if !part.is_empty() && !part.starts_with('[') && !part.starts_with(']') {
                    includes.push(format!("-I{}", part));
                }
            }
        }
    }

    let includes_ref: Vec<&str> = includes.iter().map(|s| s.as_str()).collect();

    let bindings = bindgen::Builder::default()
        .header(manifest_dir.join("wrapper.h").to_str().unwrap())
        .clang_args(&includes_ref)
        .allowlist_function(".*")
        .allowlist_type(".*")
        .allowlist_var(".*")
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
