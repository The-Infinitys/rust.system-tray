
use std::env;
use std::path::PathBuf;

fn main() {
    let dst = cmake::build("lib");

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=qt6-bind");
    println!("cargo:rustc-link-lib=Qt6Widgets");
    println!("cargo:rustc-link-lib=Qt6Gui");
    println!("cargo:rustc-link-lib=Qt6Core");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rerun-inf-changed=lib/**");

    let bindings = bindgen::Builder::default()
        .header("lib/src/lib.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
