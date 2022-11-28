//! A simple multi-threaded Gopher server.

use crate::{color, gopher, Request, Result};
use std::{
    cmp::Ordering,
    fs::{self, DirEntry},
    io::{self, prelude::*, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    str,
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
};
use threadpool::ThreadPool;

/// phd tries to be light on resources, so we only allow a low number
/// of simultaneous connections.
const MAX_WORKERS: usize = 10;

/// how many bytes of a file to read when trying to guess binary vs text?
const MAX_PEEK_SIZE: usize = 1024;

/// Files not displayed in directory listings.
const IGNORED_FILES: [&str; 3] = ["header.gph", "footer.gph", ".reverse"];

/// Whether to print info!() messages to stdout.
/// Defaults to true.
static SHOW_INFO: AtomicBool = AtomicBool::new(true);

/// Hide info! messages.
fn hide_info() {
    SHOW_INFO.swap(false, AtomicOrdering::Relaxed);
}

/// Print status message to the server's stdout.
macro_rules! info {
    ($e:expr) => {
        if SHOW_INFO.load(AtomicOrdering::Relaxed) {
            println!("{}", $e);
        }
    };
    ($fmt:expr, $($args:expr),*) => {
        info!(format!($fmt, $($args),*));
    };
    ($fmt:expr, $($args:expr,)*) => {
        info!(format!($fmt, $($args,)*));
    };
}

/// Starts a Gopher server at the specified host, port, and root directory.
pub fn start(bind: SocketAddr, host: &str, port: u16, root: &str) -> Result<()> {
    let listener = TcpListener::bind(&bind)?;
    let full_root_path = fs::canonicalize(&root)?.to_string_lossy().to_string();
    let pool = ThreadPool::new(MAX_WORKERS);

    info!(
        "{}» Listening {}on {}{}{} at {}{}{}",
        color::Yellow,
        color::Reset,
        color::Yellow,
        bind,
        color::Reset,
        color::Blue,
        full_root_path,
        color::Reset
    );
    for stream in listener.incoming() {
        let stream = stream?;
        info!(
            "{}┌ Connection{} from {}{}",
            color::Green,
            color::Reset,
            color::Magenta,
            stream.peer_addr()?
        );
        let req = Request::from(host, port, root)?;
        pool.execute(move || {
            if let Err(e) = accept(stream, req) {
                info!("{}└ {}{}", color::Red, e, color::Reset);
            }
        });
    }
    Ok(())
}

/// Reads from the client and responds.
fn accept(mut stream: TcpStream, mut req: Request) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();
    if let Some(Ok(line)) = lines.next() {
        info!(
            "{}│{} Client sent:\t{}{:?}{}",
            color::Green,
            color::Reset,
            color::Cyan,
            line,
            color::Reset
        );
        req.parse_request(&line);
        write_response(&mut stream, req)?;
    }
    Ok(())
}

/// Render a response to a String.
pub fn render(host: &str, port: u16, root: &str, selector: &str) -> Result<String> {
    hide_info();
    let mut req = Request::from(host, port, root)?;
    req.parse_request(&selector);
    let mut out = vec![];
    write_response(&mut out, req)?;
    Ok(String::from_utf8_lossy(&out).into())
}

/// Writes a response to a client based on a Request.
fn write_response<W>(w: &mut W, mut req: Request) -> Result<()>
where
    W: Write,
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
fn write_dir<W>(w: &mut W, req: Request) -> Result<()>
where
    W: Write,
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

    let rel_path = req.relative_file_path();

    // show directory entries
    let reverse = format!("{}/.reverse", path);
    let paths = sort_paths(&path, fs_exists(&reverse))?;
    for entry in paths {
        let file_name = entry.file_name();
        let f = file_name.to_string_lossy().to_string();
        if f.chars().nth(0) == Some('.') || IGNORED_FILES.contains(&f.as_ref()) {
            continue;
        }
        let path = format!(
            "{}/{}",
            rel_path.trim_end_matches('/'),
            file_name.to_string_lossy()
        );
        write!(
            w,
            "{}{}\t{}\t{}\t{}\r\n",
            file_type(&entry).to_char(),
            &file_name.to_string_lossy(),
            &path,
            &req.host,
            req.port,
        )?;
    }

    let footer = format!("{}/footer.gph", path.trim_end_matches('/'));
    if fs_exists(&footer) {
        let sel = format!("{}/footer.gph", req.selector);
        write_gophermap(
            w,
            Request {
                selector: sel,
                ..req.clone()
            },
        )?;
    }

    write!(w, ".\r\n");

    info!(
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
fn write_file<W>(w: &mut W, req: Request) -> Result<()>
where
    W: Write,
{
    let path = req.file_path();
    let mut f = fs::File::open(&path)?;
    io::copy(&mut f, w)?;
    info!(
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
fn write_gophermap<W>(w: &mut W, req: Request) -> Result<()>
where
    W: Write,
{
    let path = req.file_path();

    // Run the file and use its output as content if it's executable.
    let reader = if is_executable(&path) {
        shell(&path, &[&req.query, &req.host, &req.port.to_string()])?
    } else {
        fs::read_to_string(&path)?
    };

    for line in reader.lines() {
        write!(w, "{}", gph_line_to_gopher(line, &req))?;
    }
    info!(
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
/// get an `i` prefixed, and the geomyidae format.
fn gph_line_to_gopher(line: &str, req: &Request) -> String {
    if line.starts_with('#') {
        return "".to_string();
    }

    let mut line = line.trim_end_matches('\r').to_string();
    if line.starts_with('[') && line.ends_with(']') && line.contains('|') {
        // [1|name|sel|server|port]
        let port = req.port.to_string();
        line = line
            .replacen('|', "", 1)
            .trim_start_matches('[')
            .trim_end_matches(']')
            .replace("\\|", "__P_ESC_PIPE") // cheap hack
            .replace('|', "\t")
            .replace("__P_ESC_PIPE", "\\|")
            .replace("\tserver\t", format!("\t{}\t", req.host).as_ref())
            .replace("\tport", format!("\t{}", port).as_ref());
        let tabs = line.matches('\t').count();
        if tabs < 1 {
            line.push('\t');
            line.push_str("(null)");
        }
        // if a link is missing host + port, assume it's this server.
        // if it's just missing the port, assume port 70
        if tabs < 2 {
            line.push('\t');
            line.push_str(&req.host);
            line.push('\t');
            line.push_str(&port);
        } else if tabs < 3 {
            line.push('\t');
            line.push_str("70");
        }
    } else {
        match line.matches('\t').count() {
            0 => {
                // Always insert `i` prefix to any lines without tabs.
                line.insert(0, 'i');
                line.push_str(&format!("\t(null)\t{}\t{}", req.host, req.port))
            }
            // Auto-add host and port to lines with just a selector.
            1 => line.push_str(&format!("\t{}\t{}", req.host, req.port)),
            2 => line.push_str(&format!("\t{}", req.port)),
            _ => {}
        }
    }
    line.push_str("\r\n");
    line
}

fn write_not_found<W>(w: &mut W, req: Request) -> Result<()>
where
    W: Write,
{
    let line = format!("3Not Found: {}\t/\tnone\t70\r\n", req.selector);
    info!(
        "{}│ Not found: {}{}{}",
        color::Red,
        color::Cyan,
        req.relative_file_path(),
        color::Reset,
    );
    write!(w, "{}", line)?;
    Ok(())
}

/// Determine the gopher type for a DirEntry on disk.
fn file_type(dir: &fs::DirEntry) -> gopher::Type {
    let metadata = match dir.metadata() {
        Err(_) => return gopher::Type::Error,
        Ok(md) => md,
    };

    if metadata.is_file() {
        if let Ok(file) = fs::File::open(&dir.path()) {
            let mut buffer: Vec<u8> = vec![];
            let _ = file.take(MAX_PEEK_SIZE as u64).read_to_end(&mut buffer);
            if content_inspector::inspect(&buffer).is_binary() {
                gopher::Type::Binary
            } else {
                gopher::Type::Text
            }
        } else {
            gopher::Type::Error
        }
    } else if metadata.is_dir() {
        gopher::Type::Menu
    } else {
        gopher::Type::Error
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
            let ord = alphanumeric_sort::compare_os_str::<&Path, &Path>(
                a.path().as_ref(),
                b.path().as_ref(),
            );
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

    #[test]
    fn test_gph_geomyidae() {
        let req = Request::from("localhost", 7070, ".").unwrap();

        assert_eq!(
            gph_line_to_gopher("[1|phkt.io|/|phkt.io]", &req),
            "1phkt.io	/	phkt.io	70\r\n"
        );
        assert_eq!(gph_line_to_gopher("#[1|phkt.io|/|phkt.io]", &req), "");
        assert_eq!(
            gph_line_to_gopher("[1|sdf6000|/not-real|sdf.org|6000]", &req),
            "1sdf6000	/not-real	sdf.org	6000\r\n"
        );
        assert_eq!(
            gph_line_to_gopher("[1|R-36|/]", &req),
            "1R-36	/	localhost	7070\r\n"
        );
        assert_eq!(
            gph_line_to_gopher("[1|R-36|/|server|port]", &req),
            "1R-36	/	localhost	7070\r\n"
        );
        assert_eq!(
            gph_line_to_gopher("[0|file - comment|/file.dat|server|port]", &req),
            "0file - comment	/file.dat	localhost	7070\r\n"
        );
        assert_eq!(
            gph_line_to_gopher(
                "[0|some \\| escape and [ special characters ] test|error|server|port]",
                &req
            ),
            "0some \\| escape and [ special characters ] test	error	localhost	7070\r\n"
        );
        assert_eq!(
            gph_line_to_gopher("[|empty type||server|port]", &req),
            "empty type\t\tlocalhost\t7070\r\n",
        );
    }
}
