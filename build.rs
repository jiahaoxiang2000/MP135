// filepath: /Users/xiangjiahao/embed/MP135/build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("--target=arm-unknown-linux-gnueabihf")
        .clang_arg("-I/opt/homebrew/opt/arm-unknown-linux-gnueabihf/toolchain/arm-unknown-linux-gnueabihf/sysroot/usr/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}