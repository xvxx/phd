use crate::{Request, Result};
use gophermap::{GopherMenu, ItemType};
use std::{
    fs::{self, File},
    io::prelude::*,
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};
use threadpool::ThreadPool;

/// phd tries to be light on resources, so we only allow a low number
/// of simultaneous connections.
const MAX_WORKERS: usize = 10;

/// how many bytes of a file to read when trying to guess binary vs text?
const MAX_PEEK_SIZE: usize = 1024;

/// how many bytes to read() from the socket at a time.
const TCP_BUF_SIZE: usize = 1024;

/// Starts a Gopher server at the specified host, port, and root directory.
pub fn start(host: &str, port: u16, root: &str) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)?;
    let full_root_path = fs::canonicalize(&root)?.to_string_lossy().to_string();
    let pool = ThreadPool::new(MAX_WORKERS);

    println!("-> Listening on {} at {}", addr, full_root_path);
    for stream in listener.incoming() {
        let stream = stream?;
        println!("-> Connection from: {}", stream.peer_addr()?);
        let req = Request::from(host, port, root)?;
        pool.execute(move || {
            if let Err(e) = accept(stream, req) {
                eprintln!("-! {}", e);
            }
        });
    }
    Ok(())
}

/// Reads from the client and responds.
fn accept(stream: TcpStream, mut req: Request) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();
    if let Some(Ok(line)) = lines.next() {
        println!("-> Client sent: {:?}", line);
        req.selector = line;
        write_response(&stream, req)?;
    }
    Ok(())
}

/// Writes a response to a client based on a Request.
fn write_response<'a, W>(w: &'a W, mut req: Request) -> Result<()>
where
    &'a W: Write,
{
    let path = req.file_path();

    // check for dir.gph if we're looking for dir
    let mut gph_file = path.clone();
    gph_file.push_str(".gph");
    if Path::new(&gph_file).exists() {
        req.selector = req.selector.trim_end_matches('/').into();
        req.selector.push_str(".gph");
        return write_gophermap(w, req);
    } else {
        // check for index.gph if we're looking for dir
        let mut index = path.clone();
        ensure_trailing_slash(&mut index);
        index.push_str("index.gph");
        if Path::new(&index).exists() {
            ensure_trailing_slash(&mut req.selector);
            req.selector.push_str("index.gph");
            return write_gophermap(w, req);
        }
    }

    let md = fs::metadata(&path)?;
    if path.ends_with(".gph") {
        write_gophermap(w, req)
    } else if md.is_file() {
        write_file(w, req)
    } else if md.is_dir() {
        write_dir(w, req)
    } else {
        Ok(())
    }
}

/// Send a directory listing (menu) to the client based on a Request.
fn write_dir<'a, W>(w: &'a W, req: Request) -> Result<()>
where
    &'a W: Write,
{
    let path = req.file_path();

    let mut header = path.clone();
    header.push_str("/header.gph");
    if Path::new(&header).exists() {
        let mut sel = req.selector.clone();
        sel.push_str("/header.gph");
        write_gophermap(
            w,
            Request {
                selector: sel,
                ..req.clone()
            },
        )?;
    }
    let mut footer = path.clone();
    footer.push_str("/footer.gph");
    if Path::new(&footer).exists() {
        let mut sel = req.selector.clone();
        sel.push_str("/footer.gph");
        write_gophermap(
            w,
            Request {
                selector: sel,
                ..req.clone()
            },
        )?;
    }

    let mut menu = GopherMenu::with_write(w);
    let rel_path = req.relative_file_path();

    // sort directory entries
    let mut paths: Vec<_> = fs::read_dir(&path)?.filter_map(|r| r.ok()).collect();
    paths.sort_by_key(|dir| dir.path());

    for entry in paths {
        let file_name = entry.file_name();
        if file_name == "header.gph" || file_name == "footer.gph" {
            continue;
        }
        let mut path = rel_path.clone();
        ensure_trailing_slash(&mut path);
        path.push_str(&file_name.to_string_lossy());
        menu.write_entry(
            file_type(&entry),
            &file_name.to_string_lossy(),
            &path,
            &req.host,
            req.port,
        )?;
    }
    menu.end()?;
    Ok(())
}

/// Send a file to the client based on a Request.
fn write_file<'a, W>(mut w: &'a W, req: Request) -> Result<()>
where
    &'a W: Write,
{
    let path = req.file_path();
    let md = fs::metadata(&path)?;
    let mut f = fs::File::open(&path)?;
    let mut buf = [0; TCP_BUF_SIZE];
    let mut bytes = md.len();
    while bytes > 0 {
        let n = f.read(&mut buf[..])?;
        bytes -= n as u64;
        w.write_all(&buf[..n])?;
    }
    Ok(())
}

/// Send a gophermap (menu) to the client based on a Request.
fn write_gophermap<'a, W>(mut w: &'a W, req: Request) -> Result<()>
where
    &'a W: Write,
{
    let path = req.file_path();
    println!("write_gophermap: {:?}", path);
    let file = File::open(&path)?;
    println!("write_gophermap: {:?}", path);
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let mut line = line?.trim_end_matches("\r\n").to_string();
        match line.chars().filter(|&c| c == '\t').count() {
            0 => {
                if line.chars().nth(0) != Some('i') {
                    line.insert(0, 'i');
                }
                line.push_str(&format!("\t(null)\t{}\t{}", req.host, req.port))
            }
            1 => line.push_str(&format!("\t{}\t{}", req.host, req.port)),
            2 => line.push_str(&format!("\t{}", req.port)),
            _ => {}
        }
        line.push_str("\r\n");
        w.write_all(line.as_bytes())?;
    }
    Ok(())
}

/// Determine the gopher type for a DirEntry on disk.
fn file_type(dir: &fs::DirEntry) -> ItemType {
    let metadata = match dir.metadata() {
        Err(_) => return ItemType::Error,
        Ok(md) => md,
    };

    if metadata.is_file() {
        if let Ok(file) = fs::File::open(&dir.path()) {
            let mut buffer: Vec<u8> = vec![];
            let _ = file.take(MAX_PEEK_SIZE as u64).read_to_end(&mut buffer);
            if content_inspector::inspect(&buffer).is_binary() {
                ItemType::Binary
            } else {
                ItemType::File
            }
        } else {
            ItemType::Binary
        }
    } else if metadata.is_dir() {
        ItemType::Directory
    } else {
        ItemType::Error
    }
}

fn ensure_trailing_slash(s: &mut String) {
    if !s.ends_with('/') {
        s.push('/');
    }
}
