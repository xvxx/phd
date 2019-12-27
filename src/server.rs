use std::{
    fs,
    io::prelude::*,
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    path::PathBuf,
};
use threadpool::ThreadPool;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const MAX_PEEK_SIZE: usize = 1024;

#[derive(Default)]
pub struct Request {
    selector: String, // client info
    root: String,     // server info
    host: String,
    port: String,
}

impl Request {
    pub fn new() -> Request {
        Default::default()
    }

    pub fn root_path_string(&self) -> Result<String> {
        Ok(fs::canonicalize(&self.root)?.to_string_lossy().to_string())
    }

    pub fn path(&self) -> Result<PathBuf> {
        let mut path = fs::canonicalize(&self.root)?;
        path.push(self.selector.replace("..", ".").trim_start_matches('/'));
        Ok(path)
    }

    pub fn path_string(&self) -> Result<String> {
        let path = self.path()?;
        Ok(path.to_string_lossy().to_string())
    }
}

pub fn start(addr: impl ToSocketAddrs, root: &str) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    let pool = ThreadPool::new(4);
    let local_addr = listener.local_addr()?;
    let host = local_addr.ip();
    let port = local_addr.port();
    for stream in listener.incoming() {
        let stream = stream?;
        println!("-> Connection from: {}", stream.peer_addr()?);
        let req = Request {
            root: root.to_string(),
            host: host.to_string(),
            port: port.to_string(),
            ..Default::default()
        };
        pool.execute(|| {
            if let Err(e) = client_loop(stream, req) {
                eprintln!("-> {}", e);
            }
        });
    }
    Ok(())
}

fn client_loop(stream: TcpStream, mut req: Request) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();
    if let Some(Ok(line)) = lines.next() {
        println!("-> client sent: {:?}", line);
        req.selector = line;
        respond(stream, req)?;
    }
    Ok(())
}

fn respond(stream: TcpStream, req: Request) -> Result<()> {
    let md = fs::metadata(req.path()?)?;
    if md.is_file() {
        send_text(stream, req)
    } else if md.is_dir() {
        send_dir(stream, req)
    } else {
        Ok(())
    }
}

fn send_dir(mut stream: TcpStream, req: Request) -> Result<()> {
    let mut response = String::new();
    let mut dir = fs::read_dir(req.path()?)?;
    let mut path = req.path_string()?.replace(&req.root_path_string()?, "");
    if !path.ends_with('/') {
        path.push('/');
    }
    while let Some(Ok(entry)) = dir.next() {
        let file_type = file_type(&entry);
        let f = entry.file_name();
        let file_name = f.to_string_lossy();
        response.push_str(&format!(
            "{}{}\t{}{}\tlocalhost\t7070\r\n",
            file_type, file_name, path, file_name,
        ));
    }
    stream.write_all(response.as_bytes())?;
    stream.write_all(b"\r\n.\r\n")?; // end gopher response
    Ok(())
}

fn send_text(mut stream: TcpStream, req: Request) -> Result<()> {
    let path = req.path()?;
    let md = fs::metadata(&path)?;
    let mut f = fs::File::open(&path)?;
    let mut buf = [0; 1024];
    let mut bytes = md.len();
    while bytes > 0 {
        let n = f.read(&mut buf[..])?;
        bytes -= n as u64;
        stream.write_all(&buf)?;
    }
    stream.write_all(b"\r\n.\r\n")?; // end gopher response
    Ok(())
}

fn file_type(dir: &fs::DirEntry) -> char {
    if let Ok(metadata) = dir.metadata() {
        if metadata.is_file() {
            if let Ok(file) = fs::File::open(&dir.path()) {
                let mut buffer: Vec<u8> = vec![];
                let _ = file.take(MAX_PEEK_SIZE as u64).read_to_end(&mut buffer);
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
