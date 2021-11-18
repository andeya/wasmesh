extern crate protoc_rust;

use protoc_rust::Customize;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("service/rust/wasp/src/")
        .inputs(&["message.proto"])
        .customize(Customize {
            carllerche_bytes_for_bytes: Some(true),
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()
        .expect("protoc");
    use std::process::Command;
    let output = Command::new("sh").arg("-c").arg("cargo build-simple").output().expect("Command execution exception error prompt");
    if !output.status.success() {
        eprintln!("cargo build-simple:\n{}", String::from_utf8(output.stderr).unwrap());
    }
    let output = Command::new("sh").arg("-c").arg("cargo build-simple-release").output().expect("Command execution exception error prompt");
    if !output.status.success() {
        eprintln!("cargo build-simple-release:\n{}", String::from_utf8(output.stderr).unwrap());
    }
}
