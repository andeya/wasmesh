extern crate capnpc;

// required https://capnproto.org/install.html

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("src")
        .output_path("rust/src")
        .file("message.capnp")
        .run()
        .expect("capnpc: schema compiler command");

    use std::process::Command;
    let output = Command::new("sh").arg("-c").arg("cd rust/examples/simple && cargo build").output().expect("Command execution exception error prompt");
    println!("cargo build rust/examples/simple:\n{}", String::from_utf8(output.stdout).unwrap());
    let output = Command::new("sh").arg("-c").arg("cd rust/examples/simple && cargo build --release").output().expect("Command execution exception error prompt");
    println!("cargo build rust/examples/simple:\n{}", String::from_utf8(output.stdout).unwrap());
}
