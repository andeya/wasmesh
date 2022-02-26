extern crate protoc_rust;

use protoc_rust::Customize;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("./src/")
        .include("./")
        .inputs(&["./proto.proto"])
        .customize(Customize {
            carllerche_bytes_for_bytes: Some(true),
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()
        .unwrap_or_else(|e| eprintln!("wasmesh-service build.sh error: {}", e));
}
