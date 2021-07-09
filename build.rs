extern crate protoc_rust;

use protoc_rust::Customize;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("rust/src/")
        .inputs(&["message.proto"])
        .customize(Customize {
            carllerche_bytes_for_bytes: Some(true),
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()
        .expect("protoc");
    use std::process::Command;
    let output = Command::new("sh").arg("-c").arg("cd rust/examples/simple && cargo build").output().expect("Command execution exception error prompt");
    println!("cargo build rust/examples/simple:\n{}", String::from_utf8(output.stdout).unwrap());
    let output = Command::new("sh").arg("-c").arg("cd rust/examples/simple && cargo build --release").output().expect("Command execution exception error prompt");
    println!("cargo build rust/examples/simple:\n{}", String::from_utf8(output.stdout).unwrap());
}
