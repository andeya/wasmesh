use std::io::ErrorKind;
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::instance::local_instance_ref;
use crate::proto::resize_with_capacity;

pub(crate) async fn serve(addr: SocketAddr) -> anyhow::Result<()> {
    // Bind the listener to the address
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on rpc://{}", addr);

    loop {
        // The second item contains the IP and port of the new connection.
        match listener.accept().await {
            Ok((stream, _)) => tokio::spawn(async move {
                process(stream).await;
            }),
            Err(e) => {
                eprintln!("SERVER error: {}", e);
                break
            }
        };
    }
    return Ok(());
}

const MAX_PKG_LEN: i32 = (1 << 20) * 128;

async fn process(mut stream: TcpStream) {
    #[cfg(debug_assertions)] {
        let remote_addr = stream.peer_addr().unwrap();
        println!("RPC remote_addr = {:?}", remote_addr.to_string());
    }
    let (mut reader, mut writer) = stream.split();
    // let mut reader = BufReader::new(reader);
    let mut req_vec = vec![];
    loop {
        let len = reader.read_i32_le().await;
        if let Err(e) = len {
            if let ErrorKind::UnexpectedEof = e.kind() {
                tokio::task::yield_now().await;
                continue
            }
            eprintln!("failed to read len(i32): {}", e);
            return;
        }
        let len = len.unwrap();
        #[cfg(debug_assertions)] {
            println!("receive request pack len={}", len);
        }
        if len < 0 || len > MAX_PKG_LEN {
            eprintln!("length exceeds the limit of (0,128MB]: {:.3}MB", len as f64 / (1 << 20) as f64);
            return;
        }
        resize_with_capacity(&mut req_vec, len as usize);
        #[cfg(debug_assertions)] {
            println!("receiving request payload, len={}...", req_vec.len());
        }
        if let Err(e) = reader.read_exact(&mut req_vec).await {
            eprintln!("failed to read request: size={:.3}MB, error={}", len as f64 / (1 << 20) as f64, e);
            return;
        }
        #[cfg(debug_assertions)] {
            println!("RPC handling...");
        }
        // tokio::spawn(async move {
        match handle(&req_vec).await {
            Ok(resp_vec) => {
                if let Err(e) = writer.write_i32_le(resp_vec.len() as i32).await {
                    eprintln!("failed to write response size, error={}", e);
                    return;
                }
                if let Err(e) = writer.write_all(resp_vec.as_slice()).await {
                    eprintln!("failed to write response payload, error={}", e);
                    return;
                }
                if let Err(e) = writer.flush().await {
                    eprintln!("failed to flush response, error={}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("failed to handle: {}", e);
            }
        }
    }
}

async fn handle(req_vec: &Vec<u8>) -> Result<Vec<u8>, String> {
    let (thread_id, ins) = local_instance_ref();
    let ctx_id = ins.gen_ctx_id(thread_id);

    #[cfg(debug_assertions)]
    println!("thread_id:{}, ctx_id:{}", thread_id, ctx_id);

    let buffer_len = ins.use_mut_buffer(ctx_id, req_vec.len(), |buffer: &mut Vec<u8>| {
        buffer.copy_from_slice(req_vec.as_slice());
        req_vec.len()
    });

    ins.call_guest_handler(ctx_id, buffer_len as i32);

    Ok(ins.take_buffer(ctx_id).unwrap_or(vec![]))
}
