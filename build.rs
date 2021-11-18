extern crate protoc_rust;

use protoc_rust::Customize;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("rust/wasp/src/")
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
    println!("cargo build-simple:\n{}", String::from_utf8(output.stdout).unwrap());
    let output = Command::new("sh").arg("-c").arg("cargo build-simple-release").output().expect("Command execution exception error prompt");
    println!("cargo build-simple-release:\n{}", String::from_utf8(output.stdout).unwrap());
}
