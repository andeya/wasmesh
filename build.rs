fn main() {
    use std::process::Command;
    let output = Command::new("sh").arg("-c").arg("cd examples/simple && cargo build").output().expect("Command execution exception error prompt");
    println!("cargo build examples/simple:\n{}", String::from_utf8(output.stdout).unwrap());
    let output = Command::new("sh").arg("-c").arg("cd examples/simple && cargo build --release").output().expect("Command execution exception error prompt");
    println!("cargo build examples/simple:\n{}", String::from_utf8(output.stdout).unwrap());
}
