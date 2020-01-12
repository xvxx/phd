use crate::{color, Request, Result};
use gophermap::{GopherMenu, ItemType};
use std::{
    cmp::Ordering,
    fs::{self, DirEntry},
    io::{self, prelude::*, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    str,
};
use threadpool::ThreadPool;

/// phd tries to be light on resources, so we only allow a low number
/// of simultaneous connections.
const MAX_WORKERS: usize = 10;

/// how many bytes of a file to read when trying to guess binary vs text?
const MAX_PEEK_SIZE: usize = 1024;

/// Files not displayed in directory listings.
const IGNORED_FILES: [&str; 3] = ["header.gph", "footer.gph", ".reverse"];

/// Starts a Gopher server at the specified host, port, and root directory.
pub fn start(host: &str, port: u16, root: &str) -> Result<()> {
    let addr = format!("{}:{}", "0.0.0.0", port);
    let listener = TcpListener::bind(&addr)?;
    let full_root_path = fs::canonicalize(&root)?.to_string_lossy().to_string();
    let pool = ThreadPool::new(MAX_WORKERS);

    println!(
        "{}┬ Listening {}on {}{}{} at {}{}{}",
        color::Yellow,
        color::Reset,
        color::Yellow,
        addr,
        color::Reset,
        color::Blue,
        full_root_path,
        color::Reset
    );
    for stream in listener.incoming() {
        let stream = stream?;
        println!(
            "{}┌ Connection{} from {}{}",
            color::Green,
            color::Reset,
            color::Magenta,
            stream.peer_addr()?
        );
        let req = Request::from(host, port, root)?;
        pool.execute(move || {
            if let Err(e) = accept(stream, req) {
                eprintln!("{}└ {}{}", color::Red, e, color::Reset);
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
        println!(
            "{}│{} Client sent:\t{}{:?}{}",
            color::Green,
            color::Reset,
            color::Cyan,
            line,
            color::Reset
        );
        req.parse_request(&line);
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
    if fs_exists(&gph_file) {
        req.selector = req.selector.trim_end_matches('/').into();
        req.selector.push_str(".gph");
        return write_gophermap(w, req);
    } else {
        // check for index.gph if we're looking for dir
        let mut index = path.clone();
        index.push_str("/index.gph");
        if fs_exists(&index) {
            req.selector.push_str("/index.gph");
            return write_gophermap(w, req);
        }
    }

    let meta = match fs::metadata(&path) {
        Ok(meta) => meta,
        Err(_) => return write_not_found(w, req),
    };

    if path.ends_with(".gph") {
        write_gophermap(w, req)
    } else if meta.is_file() {
        write_file(w, req)
    } else if meta.is_dir() {
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
    if !fs_exists(&path) {
        return write_not_found(w, req);
    }

    let mut header = path.clone();
    header.push_str("/header.gph");
    if fs_exists(&header) {
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

    let mut menu = GopherMenu::with_write(w);
    let rel_path = req.relative_file_path();

    // show directory entries
    let mut reverse = path.to_string();
    reverse.push_str("/.reverse");
    let paths = sort_paths(&path, fs_exists(&reverse))?;
    for entry in paths {
        let file_name = entry.file_name();
        let f = file_name.to_string_lossy().to_string();
        if f.chars().nth(0) == Some('.') || IGNORED_FILES.contains(&f.as_ref()) {
            continue;
        }
        let mut path = rel_path.clone();
        path.push('/');
        path.push_str(&file_name.to_string_lossy());
        menu.write_entry(
            file_type(&entry),
            &file_name.to_string_lossy(),
            &path,
            &req.host,
            req.port,
        )?;
    }

    let mut footer = path.clone();
    footer.push_str("/footer.gph");
    if fs_exists(&footer) {
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

    menu.end()?;
    println!(
        "{}│{} Server reply:\t{}DIR {}{}{}",
        color::Green,
        color::Reset,
        color::Yellow,
        color::Bold,
        req.relative_file_path(),
        color::Reset,
    );
    Ok(())
}

/// Send a file to the client based on a Request.
fn write_file<'a, W>(mut w: &'a W, req: Request) -> Result<()>
where
    &'a W: Write,
{
    let path = req.file_path();
    let mut f = fs::File::open(&path)?;
    io::copy(&mut f, &mut w)?;
    println!(
        "{}│{} Server reply:\t{}FILE {}{}{}",
        color::Green,
        color::Reset,
        color::Yellow,
        color::Bold,
        req.relative_file_path(),
        color::Reset,
    );
    Ok(())
}

/// Send a gophermap (menu) to the client based on a Request.
fn write_gophermap<'a, W>(mut w: &'a W, req: Request) -> Result<()>
where
    &'a W: Write,
{
    let path = req.file_path();

    // Run the file and use its output as content if it's executable.
    let reader = if is_executable(&path) {
        shell(&path, &[&req.query, &req.host, &req.port.to_string()])?
    } else {
        fs::read_to_string(&path)?
    };

    for line in reader.lines() {
        w.write_all(gph_line_to_gopher(line, &req).as_bytes())?;
    }
    println!(
        "{}│{} Server reply:\t{}MAP {}{}{}",
        color::Green,
        color::Reset,
        color::Yellow,
        color::Bold,
        req.relative_file_path(),
        color::Reset,
    );
    Ok(())
}

/// Given a single line from a .gph file, convert it into a
/// Gopher-format line. Supports a basic format where lines without \t
/// get an `i` prefixed, and the Gophernicus format.
fn gph_line_to_gopher(line: &str, req: &Request) -> String {
    let mut line = line.trim_end_matches("\r").to_string();
    match line.chars().filter(|&c| c == '\t').count() {
        0 => {
            // Insert `i` prefix to any prefix-less lines without tabs.
            if line.chars().nth(0) != Some('i') {
                line.insert(0, 'i');
            }
            line.push_str(&format!("\t(null)\t{}\t{}", req.host, req.port))
        }
        // Auto-add host and port to lines with just a selector.
        1 => line.push_str(&format!("\t{}\t{}", req.host, req.port)),
        2 => line.push_str(&format!("\t{}", req.port)),
        _ => {}
    }
    line.push_str("\r\n");
    line
}

fn write_not_found<'a, W>(mut w: &'a W, req: Request) -> Result<()>
where
    &'a W: Write,
{
    let line = format!("3Not Found: {}\t/\tnone\t70\r\n", req.selector);
    println!(
        "{}│ Not found: {}{}{}",
        color::Red,
        color::Cyan,
        req.relative_file_path(),
        color::Reset,
    );
    w.write_all(line.as_bytes())?;
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
            ItemType::Error
        }
    } else if metadata.is_dir() {
        ItemType::Directory
    } else {
        ItemType::Error
    }
}

/// Does the file exist? Y'know.
fn fs_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Is the file at the given path executable?
fn is_executable(path: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        meta.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

/// Run a script and return its output.
fn shell(path: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(path).args(args).output()?;
    if output.status.success() {
        Ok(str::from_utf8(&output.stdout)?.to_string())
    } else {
        Ok(str::from_utf8(&output.stderr)?.to_string())
    }
}

/// Sort directory paths: dirs first, files 2nd, version #s respected.
fn sort_paths(dir_path: &str, reverse: bool) -> Result<Vec<DirEntry>> {
    let mut paths: Vec<_> = fs::read_dir(dir_path)?.filter_map(|r| r.ok()).collect();
    let is_dir = |entry: &fs::DirEntry| match entry.file_type() {
        Ok(t) => t.is_dir(),
        _ => false,
    };
    paths.sort_by(|a, b| {
        let a_is_dir = is_dir(a);
        let b_is_dir = is_dir(b);
        if a_is_dir && b_is_dir || !a_is_dir && !b_is_dir {
            let ord = alphanumeric_sort::compare_os_str(a.path().as_ref(), b.path().as_ref());
            if reverse {
                ord.reverse()
            } else {
                ord
            }
        } else if is_dir(a) {
            Ordering::Less
        } else if is_dir(b) {
            Ordering::Greater
        } else {
            Ordering::Equal // what
        }
    });
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! str_path {
        ($e:expr) => {
            $e.path()
                .to_str()
                .unwrap()
                .trim_start_matches("tests/sort/")
        };
    }

    #[test]
    fn test_sort_directory() {
        let paths = sort_paths("tests/sort", false).unwrap();
        assert_eq!(str_path!(paths[0]), "zzz");
        assert_eq!(str_path!(paths[1]), "phetch-v0.1.7-linux-armv7.tar.gz");
        assert_eq!(
            str_path!(paths[paths.len() - 1]),
            "phetch-v0.1.11-macos.zip"
        );
    }

    #[test]
    fn test_rsort_directory() {
        let paths = sort_paths("tests/sort", true).unwrap();
        assert_eq!(str_path!(paths[0]), "zzz");
        assert_eq!(str_path!(paths[1]), "phetch-v0.1.11-macos.zip");
        assert_eq!(
            str_path!(paths[paths.len() - 1]),
            "phetch-v0.1.7-linux-armv7.tar.gz"
        );
    }

    #[test]
    fn test_gph_line_to_gopher() {
        let req = Request::from("localhost", 70, ".").unwrap();

        assert_eq!(
            gph_line_to_gopher("regular line test", &req),
            "iregular line test	(null)	localhost	70\r\n"
        );
        assert_eq!(
            gph_line_to_gopher("1link test	/test	localhost	70", &req),
            "1link test	/test	localhost	70\r\n"
        );

        let line = "0short link test	/test";
        assert_eq!(
            gph_line_to_gopher(line, &req),
            "0short link test	/test	localhost	70\r\n"
        );
    }
}
