#![allow(unused_must_use)]

extern crate async_std;

use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() {
    println!("-> Listening on localhost:7070");
    let fut = listen("localhost:7070");
    if let Err(e) = task::block_on(fut) {
        eprintln!("{}", e);
    }
}

async fn listen(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("-> Connection from: {}", stream.peer_addr()?);
        task::spawn(client_loop(stream));
    }
    Ok(())
}

async fn client_loop(stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next().await {
        let line = line?;
        println!("-> client sent: {:?}", line);
    }
    Ok(())
}
