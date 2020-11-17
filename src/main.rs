use phd;
use std::process;

const DEFAULT_BIND: &str = "[::]:7070";
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 7070;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut args = args.iter();
    let mut root = ".";
    let mut addr = DEFAULT_BIND;
    let mut host = DEFAULT_HOST;
    let mut port = DEFAULT_PORT;
    let mut render = "";

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "--version" | "-v" | "-version" => return print_version(),
            "--help" | "-help" => return print_help(),
            "--no-color" | "-no-color" => phd::color::hide_colors(),
            "--render" | "-render" | "-r" => {
                if let Some(path) = args.next() {
                    render = path;
                } else {
                    render = "/";
                }
            }
            "--bind" | "-b" | "-bind" => {
                if let Some(a) = args.next() {
                    addr = a
                }
            }
            "--port" | "-p" | "-port" => {
                if let Some(p) = args.next() {
                    port = p
                        .parse()
                        .map_err(|_| {
                            eprintln!("bad port: {}", p);
                            process::exit(1)
                        })
                        .unwrap();
                }
            }
            "-h" => {
                if let Some(h) = args.next() {
                    host = &h;
                } else {
                    return print_help();
                }
            }
            "--host" | "-host" => {
                if let Some(h) = args.next() {
                    host = &h;
                }
            }
            _ => {
                if let Some('-') = arg.chars().nth(0) {
                    eprintln!("unknown flag: {}", arg);
                    process::exit(1);
                } else {
                    root = &arg;
                }
            }
        }
    }

    // https://no-color.org/
    if std::env::var("NO_COLOR").is_ok() {
        phd::color::hide_colors()
    }

    // If port was given and socket wasn't, bind to that port.
    let bind = if port != DEFAULT_PORT && addr == DEFAULT_BIND {
        format!("[::]:{}", port).parse().unwrap()
    } else {
        addr.parse().unwrap()
    };

    if !render.is_empty() {
        return match phd::server::render(host, port, root, &render) {
            Ok(out) => print!("{}", out),
            Err(e) => eprintln!("{}", e),
        };
    }

    if let Err(e) = phd::server::start(bind, host, port, root) {
        eprintln!("{}", e);
    }
}

fn print_help() {
    println!(
        "Usage:

    phd [options] <root directory>

Options:

    -r, --render SELECTOR  Render and print SELECTOR to stdout only.
    -h, --host HOST        Hostname for links. [Default: {host}]
    -p, --port PORT        Port for links. [Default: {port}]
    -b, --bind ADDRESS     Socket address to bind to. [Default: {bind}]
    --no-color             Don't show colors in log messages.

Other flags:

    -h, --help      Print this screen.
    -v, --version   Print phd version.

Examples:

    phd ./path/to/site  # Serve directory over port 7070.
    phd -p 70 docs      # Serve 'docs' directory on port 70
    phd -h gopher.com   # Serve current directory over port 7070
                        # using hostname 'gopher.com'
    phd -r / ./site     # Render local gopher site to stdout.
",
        host = DEFAULT_HOST,
        port = DEFAULT_PORT,
        bind = DEFAULT_BIND,
    );
}

fn print_version() {
    println!("phd v{}", env!("CARGO_PKG_VERSION"));
}
