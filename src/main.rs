#[macro_use]
extern crate lazy_static;


mod server;
mod wasi;

lazy_static! {static ref SERVER: Result<server::Server,  String> = server::Server::new().map_err(|e|format!("{}",e));}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SERVER.as_ref()
          .map_err(|e| e.as_str())?
        .serve(([127, 0, 0, 1], 8080).into())
        .await;
    Ok(())
}

#[test]
fn test_wasi() -> Result<(), Box<dyn std::error::Error>> {
    main()
}
