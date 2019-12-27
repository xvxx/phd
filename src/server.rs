use async_std::{
    fs,
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    path::PathBuf,
    prelude::*,
    task,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const MAX_PEEK_SIZE: usize = 1024;

#[derive(Default)]
pub struct Request {
    selector: String, // client info

    root: String, // server info
    host: String,
    port: String,
}

impl Request {
    pub fn new() -> Request {
        Default::default()
    }

    pub async fn path(&self) -> Result<PathBuf> {
        let mut path = fs::canonicalize(&self.root).await?;
        path.push(self.selector.replace("..", ".").trim_start_matches('/'));
        Ok(path)
    }

    pub async fn path_string(&self) -> Result<String> {
        let path = self.path().await?;
        Ok(path.to_string_lossy().to_string())
    }
}

pub fn start(addr: impl ToSocketAddrs, root: &str) -> Result<()> {
    task::block_on(async {
        let listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();
        let local_addr = listener.local_addr()?;
        let host = local_addr.ip();
        let port = local_addr.port();
        let root = fs::canonicalize(&root).await?.to_string_lossy().to_string();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            println!("-> Connection from: {}", stream.peer_addr()?);
            let req = Request {
                root: root.to_string(),
                host: host.to_string(),
                port: port.to_string(),
                ..Default::default()
            };
            task::spawn(async {
                if let Err(e) = client_loop(stream, req).await {
                    eprintln!("-> {}", e);
                }
            });
        }
        Ok(())
    })
}

async fn client_loop(mut stream: TcpStream, mut req: Request) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();

    if let Some(Ok(line)) = lines.next().await {
        println!("-> {} sent: {:?}", stream.peer_addr()?, line);
        req.selector = line;
        respond(&mut stream, req).await?;
    }
    Ok(())
}

async fn respond(stream: &mut TcpStream, req: Request) -> Result<()> {
    let mut path = fs::canonicalize(&req.root).await?;
    path.push(req.selector.replace("..", ".").trim_start_matches('/'));
    println!("path {:?}", path);

    let md = fs::metadata(path.clone()).await?;
    if md.is_file() {
        return send_text(stream, req).await;
    } else if md.is_dir() {
        return send_dir(stream, req).await;
    } else {
        Ok(())
    }
}

async fn send_dir(stream: &mut TcpStream, req: Request) -> Result<()> {
    let mut response = String::new();
    let mut dir = fs::read_dir(req.path().await?).await?;
    let mut path = req.path_string().await?.replace(&req.root, "");
    if !path.ends_with('/') {
        path.push('/');
    }
    while let Some(Ok(entry)) = dir.next().await {
        let file_type = file_type(&entry).await;
        let f = entry.file_name();
        let file_name = f.to_string_lossy();
        response.push_str(&format!(
            "{}{}\t{}{}\tlocalhost\t7070\r\n",
            file_type, file_name, path, file_name,
        ));
    }
    stream.write_all(response.as_bytes()).await?;
    stream.write_all(b"\r\n.\r\n").await?; // end gopher response
    Ok(())
}

async fn send_text(stream: &mut TcpStream, req: Request) -> Result<()> {
    let path = req.path().await?;
    let md = fs::metadata(&path).await?;
    let mut f = fs::File::open(&path).await?;
    let mut buf = [0; 1024];
    let mut bytes = md.len();
    while bytes > 0 {
        let n = f.read(&mut buf[..]).await?;
        bytes -= n as u64;
        stream.write_all(&buf).await?;
    }
    stream.write_all(b"\r\n.\r\n").await?; // end gopher response
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
