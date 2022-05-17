#![windows_subsystem = "windows"]

use ipp::prelude::*;
use std::io::{Read, Write};

fn print(file: Vec<u8>, user: &str) -> Result<(), Box<dyn std::error::Error>> {
    let uri: Uri = std::env::var("IPP_PRINTER")
        .expect("You haven't provided a printer through the IPP_PRINTER env var")
        .parse()?;

    let payload = IppPayload::new(std::io::Cursor::new(file));

    let job = IppOperationBuilder::print_job(uri.clone(), payload)
        .user_name(user)
        .attribute(IppAttribute::new(
            "media",
            IppValue::Array(vec![
                IppValue::Keyword("manual".to_string()),
                IppValue::Keyword("iso_dl_110x220mm".to_string()),
            ]),
        ))
        .build();

    let client = IppClient::builder(uri).ignore_tls_errors(true).build();
    let _ = futures::executor::block_on(client.send(job))?;
    Ok(())
}
fn main() {
    // -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let sock = std::net::TcpListener::bind(
        std::env::var("TCP_PDF_IPP_BIND").expect("Missing TCP_PDF_IPP_BIND env var"),
    )
    .expect("Could not listen on specified port");
    for stream in sock.incoming() {
        if let Ok(mut stream) = stream {
            let mut buf: Vec<u8> = vec![];
            stream.read_to_end(&mut buf).ok();
            if let Err(e) = print(buf, stream.peer_addr().unwrap().to_string().as_str()) {
                stream.write(format!("{:?}", e).as_bytes()).ok();
            } else {
                stream.write("Ok".as_bytes()).ok();
            };
        }
    }
}
