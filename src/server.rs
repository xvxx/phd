use async_std::{
    fs,
    io::BufReader,
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};
use content_inspector::{inspect, ContentType};
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const MAX_PEEK_SIZE: usize = 1024;

pub fn start(addr: &str, root: &str) -> Result<()> {
    task::block_on(async {
        let listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            println!("-> Connection from: {}", stream.peer_addr()?);
            let root = root.to_string();
            task::spawn(client_loop(stream, root));
        }
        Ok(())
    })
}

async fn client_loop(mut stream: TcpStream, root: String) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();

    if let Some(Ok(line)) = lines.next().await {
        println!("-> client sent: {:?}", line);
        respond(&mut stream, &line, &root).await?;
    }
    Ok(())
}

async fn respond(stream: &mut TcpStream, selector: &str, root: &str) -> Result<()> {
    let mut path = PathBuf::from(root);
    path.push(selector.replace("..", "."));

    let md = fs::metadata(path.clone()).await?;
    if md.is_file() {
        let mut f = fs::File::open(path).await?;
        let mut buf = [0; 1024];
        let mut bytes = md.len();
        while bytes > 0 {
            let n = f.read(&mut buf[..]).await?;
            bytes -= n as u64;
            stream.write_all(&buf).await?;
        }
        return Ok(());
    }

    let mut response = String::new();
    let mut dir = fs::read_dir(path.clone()).await?;

    while let Some(Ok(entry)) = dir.next().await {
        let file_type = file_type(&entry).await;
        response.push_str(&format!(
            "{}{}\t{}\tlocalhost\t7070\r\n",
            file_type,
            entry.file_name().into_string().unwrap(),
            entry.path().to_string_lossy(),
        ));
    }
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

async fn file_type(dir: &fs::DirEntry) -> char {
    if let Ok(metadata) = dir.metadata().await {
        if metadata.is_file() {
            if let Ok(file) = fs::File::open(&dir.path()).await {
                let mut buffer: Vec<u8> = vec![];
                let _ = file
                    .take(MAX_PEEK_SIZE as u64)
                    .read_to_end(&mut buffer)
                    .await;
                if content_inspector::inspect(&buffer).is_binary() {
                    '9'
                } else {
                    '0'
                }
            } else {
                '9'
            }
        } else if metadata.is_dir() {
            '1'
        } else {
            '3'
        }
    } else {
        '3'
    }
}
