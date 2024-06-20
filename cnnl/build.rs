fn main() {
    use build_script_cfg::Cfg;
    use search_neuware_tools::find_neuware_home;
    use std::{env, path::PathBuf};

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=NEUWARE_HOME");

    let neuware = Cfg::new("detected_neuware");
    let Some(neuware_home) = find_neuware_home() else {
        return;
    };
    neuware.define();

    println!(
        "cargo:rustc-link-search=native={}",
        neuware_home.join("lib64").display()
    );
    println!("cargo:rustc-link-lib=dylib=cnnl");

    // Tell cargo to invalidate the built crate whenever the wrapper changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point to bindgen,
    // and lets you build up options for the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}", neuware_home.join("include").display()))
        // Only generate bindings for the functions in these namespaces.
        .allowlist_function("cnnl.*")
        .allowlist_item("cnnl.*")
        // Annotate the given type with the #[must_use] attribute.
        .must_use_type("cnnlStatus_t")
        // Generate rust style enums.
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        // Use core instead of std in the generated bindings.
        .use_core()
        // Tell cargo to invalidate the built crate whenever any of the included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
