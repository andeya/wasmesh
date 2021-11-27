extern crate protoc_rust;

use std::process::Command;

use protoc_rust::Customize;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("../service/rust/wasmesh/src/")
        .include("../")
        .inputs(&["../message.proto"])
        .customize(Customize {
            carllerche_bytes_for_bytes: Some(true),
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()
        .expect("protoc");
    let arg = "cargo build --target wasm32-wasi --package simple --target-dir ../service/rust/examples/target";
    let output = Command::new("sh").arg("-c").arg(&arg).output().expect("Command execution exception error prompt");
    if !output.status.success() {
        eprintln!("$ {}:\n{}", arg, String::from_utf8(output.stderr).unwrap());
    }
    let arg = format!("{} --release", arg);
    let output = Command::new("sh").arg("-c").arg(&arg).output().expect("Command execution exception error prompt");
    if !output.status.success() {
        eprintln!("$ {}:\n{}", arg, String::from_utf8(output.stderr).unwrap());
    }
}
