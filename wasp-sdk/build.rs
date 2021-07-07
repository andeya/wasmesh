extern crate protoc_rust;

use protoc_rust::Customize;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/")
        .inputs(&["message.proto"])
        .customize(Customize {
            carllerche_bytes_for_bytes: Some(true),
            // carllerche_bytes_for_string: Some(true),
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()
        .expect("protoc");
}
